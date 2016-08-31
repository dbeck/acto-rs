
use std::collections::{HashMap};
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
use super::super::{Task, Error};
use super::{array, task_id};
use super::observer::{CountingReporter};
use parking_lot::{Mutex};
use std::ptr;
use time;
use libc;

pub struct SchedulerData {
  max_id:   AtomicUsize,
  l1:       Vec<AtomicPtr<array::TaskArray>>,
  stop:     AtomicBool,
  time_us:  AtomicUsize,
  ids:      Mutex<HashMap<String, usize>>,
}

impl SchedulerData {
  fn add_l2_bucket(&mut self, idx: usize) {
    let array = Box::new(array::new());
    let l1_slice = self.l1.as_mut_slice();
    l1_slice[idx].store(Box::into_raw(array), Ordering::Release);
  }

  fn new() -> SchedulerData {
    let l1_size = initial_capacity();
    let mut data = SchedulerData{
      max_id:   AtomicUsize::new(0),
      l1:       Vec::with_capacity(l1_size),
      stop:     AtomicBool::new(false),
      time_us:  AtomicUsize::new((time::precise_time_ns()/1000) as usize),
      ids:      Mutex::new(HashMap::new()),
    };

    // fill the l1 bucket
    for _i in 0..l1_size {
      data.l1.push(AtomicPtr::default());
    }

    // add an initial l2 bucket
    data.add_l2_bucket(0);
    data
  }

  pub fn add_task(&mut self, task: Box<Task+Send>) -> Result<task_id::TaskId, Error> {
    let ret_id : usize;
    // check if name exists, and register if not
    {
      let mut guard = self.ids.lock();
      if guard.contains_key(task.name()) {
        return Result::Err(Error::AlreadyExists);
      } else {
        ret_id = self.max_id.fetch_add(1, Ordering::SeqCst);
        guard.insert(task.name().clone(), ret_id);
      }
    }
    let (l1, l2) = array::position(ret_id);
    if l2 == 0 {
      // make sure the next bucket exists when needed
      self.add_l2_bucket(l1+1);
    }
    unsafe {
      let l1_ptr = self.l1.get_unchecked_mut(l1).load(Ordering::Acquire);
      if l1_ptr.is_null() == false {
        (*l1_ptr).store(l2, task, ret_id);
      }
    }
    Result::Ok(task_id::new(ret_id))
  }

  pub fn ticker(&mut self) {
    loop {
      unsafe { libc::usleep(10); }
      self.time_us.store((time::precise_time_ns()/1000) as usize, Ordering::Release);
      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
    }
  }

  pub fn entry(&mut self, id: usize) {
    let mut exec_count : u64 = 0;
    let start = time::precise_time_ns();
    loop {
      let mut reporter = CountingReporter::new();
      let (l1, l2) = array::position(self.max_id.load(Ordering::Acquire));
      let l1_slice = self.l1.as_mut_slice();
      for l1_idx in 0..(l1+1) {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::Acquire);
        let mut l2_max_idx = array::max_idx();
        if l1_idx == l1 {
          l2_max_idx = l2;
        }
        unsafe {
          exec_count += (*l1_ptr).eval(l2_max_idx, id, &mut reporter, &self.time_us);
        }
      }
      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
    }
    let end = time::precise_time_ns();
    let diff = end - start;
    println!("thread: #{} exiting. #exec: {} exec-time: {} ns {}k/s",
      id, exec_count, diff/exec_count, exec_count*1_000*1_000/diff);
  }

  pub fn notify(&mut self, id: &task_id::TaskId) -> Result<usize, Error> {
    if self.stop.load(Ordering::Acquire) {
      return Result::Err(Error::Stopping);
    }
    let max = self.max_id.load(Ordering::Acquire);
    if id.id() >= max {
      return Result::Err(Error::NonExistent);
    }
    let (l1, l2) = array::position(id.id());
    let l1_slice = self.l1.as_mut_slice();
    let l1_ptr = l1_slice[l1].load(Ordering::Acquire);
    if l1_ptr.is_null() {
      return Result::Err(Error::NonExistent);
    }
    unsafe { (*l1_ptr).notify(l2) }
  }

  pub fn stop(&mut self) {
    self.stop.store(true, Ordering::Release);
  }
}

pub fn new() -> SchedulerData {
  SchedulerData::new()
}

pub fn initial_capacity() -> usize {
  65536
}

impl Drop for SchedulerData {
  fn drop(&mut self) {
    let len = self.l1.len();
    let l1_slice = self.l1.as_mut_slice();
    for i in 0..len {
      let l1_atomic_ptr = &mut l1_slice[i];
      let ptr = l1_atomic_ptr.swap(ptr::null_mut::<array::TaskArray>(), Ordering::SeqCst);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      } else {
        break;
      }
    }
  }
}
