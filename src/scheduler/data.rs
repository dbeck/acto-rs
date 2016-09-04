
use std::collections::{HashMap};
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
use super::super::{Task, Error};
use super::{array, task_id};
use super::observer::{TaskObserver};
use parking_lot::{Mutex};
use std::ptr;
use std::time::{Duration, Instant};
use libc;

pub struct SchedulerData {
  start:    Instant,
  max_id:   AtomicUsize,
  l1:       Vec<AtomicPtr<array::TaskArray>>,
  stop:     AtomicBool,
  time_us:  AtomicUsize,
  ids:      Mutex<HashMap<String, usize>>,
}

impl SchedulerData {
  fn add_l2_bucket(&mut self, idx: usize) {
    let array = Box::new(array::new());
    let len = self.l1.len();
    if idx >= len-1 {
      // extend slice
      for _i in 0..initial_capacity() {
        self.l1.push(AtomicPtr::default());
      }
    }
    let l1_slice = self.l1.as_mut_slice();
    l1_slice[idx].store(Box::into_raw(array), Ordering::Release);
  }

  fn new() -> SchedulerData {
    let l1_size = initial_capacity();
    let mut data = SchedulerData{
      start:    Instant::now(),
      max_id:   AtomicUsize::new(0),
      l1:       Vec::with_capacity(l1_size),
      stop:     AtomicBool::new(false),
      time_us:  AtomicUsize::new(0),
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
    let input_count = task.input_count();
    let mut input_task_ids : Vec<Option<usize>> = Vec::with_capacity(input_count);
    // check if name exists, and register if not
    {
      let mut ids = self.ids.lock();
      if ids.contains_key(task.name()) {
        return Result::Err(Error::AlreadyExists);
      } else {
        ret_id = self.max_id.fetch_add(1, Ordering::AcqRel);
        ids.insert(task.name().clone(), ret_id);
        for i in 0..input_count {
          match task.input_id(i) {
            Some(ref ch_id) => {
              match ids.get(ch_id.task_name()) {
                Some(&id)  => input_task_ids.push(Some(id)),
                None       => input_task_ids.push(None)
              }
            },
            _ => input_task_ids.push(None)
          }
        }
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
        (*l1_ptr).store(l2, task, ret_id, input_task_ids);
      }
    }
    Result::Ok(task_id::new(ret_id))
  }

  pub fn ticker(&mut self) {
    loop {
      unsafe { libc::usleep(10); }
      let diff = self.start.elapsed();
      let diff = diff.as_secs() as usize * 1000_000 + diff.subsec_nanos() as usize / 1000;
      self.time_us.store(diff, Ordering::Release);
      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
    }
  }

  pub fn entry(&mut self, id: usize) {
    let mut exec_count : u64 = 0;
    let start = Instant::now();
    let l2_max = array::max_idx();
    loop {
      let max_id = self.max_id.load(Ordering::Acquire);
      let mut reporter = TaskObserver::new(max_id);
      let (l1, l2) = array::position(max_id);
      let l1_slice = self.l1.as_mut_slice();

      // go through all fully filled l2 buckets
      let mut l2_max_idx = l2_max;
      for l1_idx in 0..l1 {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::Acquire);
        unsafe {
          (*l1_ptr).eval(l2_max_idx, id, &mut reporter, &self.time_us);
        }
      }

      // take care of the last, partially filled bucket
      l2_max_idx = l2;
      for l1_idx in l1..(l1+1) {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::Acquire);
        unsafe {
          (*l1_ptr).eval(l2_max_idx, id, &mut reporter, &self.time_us);
        }
      }

      exec_count += reporter.exec_count();

      {
        // process msg wait dependencies
      }
      
      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
    }
    let diff = start.elapsed();
    let diff = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
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
      let ptr = l1_atomic_ptr.swap(ptr::null_mut::<array::TaskArray>(), Ordering::AcqRel);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      } else {
        break;
      }
    }
  }
}
