
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
use super::super::{Task, Error};
use super::{array, task_id, CountingReporter};
use std::ptr;
use time;
use libc;

pub struct SchedulerData {
  max_id:   AtomicUsize,
  l1:       Vec<AtomicPtr<array::TaskArray>>,
  stop:     AtomicBool,
  time_us:  AtomicUsize,
}

impl SchedulerData {
  fn add_l2_bucket(&mut self, idx: usize) {
    let array = Box::new(array::new());
    let l1_slice = self.l1.as_mut_slice();
    l1_slice[idx].store(Box::into_raw(array), Ordering::SeqCst);
  }

  fn new() -> SchedulerData {
    let l1_size = initial_capacity();
    let mut data = SchedulerData{
      max_id:   AtomicUsize::new(0),
      l1:       Vec::with_capacity(l1_size),
      stop:     AtomicBool::new(false),
      time_us:  AtomicUsize::new((time::precise_time_ns()/1000) as usize),
    };

    // fill the l1 bucket
    for _i in 0..l1_size {
      data.l1.push(AtomicPtr::default());
    }

    // add an initial l2 bucket
    data.add_l2_bucket(0);
    data
  }

  pub fn add_task(&mut self, task: Box<Task+Send>) -> task_id::TaskId {
    let ret = task_id::new(self.max_id.fetch_add(1, Ordering::SeqCst));
    let (l1, l2) = array::position(ret.id());
    if l2 == 0 {
      // make sure the next bucket exists when needed
      self.add_l2_bucket(l1+1);
    }
    unsafe {
      let l1_ptr = self.l1.get_unchecked_mut(l1).load(Ordering::SeqCst);
      if l1_ptr.is_null() == false {
        (*l1_ptr).store(l2, task, ret.id());
      }
    }
    ret
  }

  pub fn ticker(&mut self) {
    loop {
      if self.stop.load(Ordering::SeqCst) {
        break;
      }
      unsafe { libc::usleep(10); }
      self.time_us.store((time::precise_time_ns()/1000) as usize, Ordering::Release);
    }
  }

  pub fn entry(&mut self, id: usize) {
    let mut exec_count : u64 = 0;
    let start = time::precise_time_ns();
    loop {
      if self.stop.load(Ordering::SeqCst) {
        break;
      }
      let mut reporter = CountingReporter{ count: 0 };
      let (l1, l2) = array::position(self.max_id.load(Ordering::SeqCst));
      let l1_slice = self.l1.as_mut_slice();
      for l1_idx in 0..(l1+1) {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::SeqCst);
        let mut l2_max_idx = array::max_idx();
        if l1_idx == l1 {
          l2_max_idx = l2;
        }
        unsafe {
          exec_count += (*l1_ptr).execute(l2_max_idx, id, &mut reporter, &self.time_us);
        }
      }
    }
    let end = time::precise_time_ns();
    let diff = end - start;
    println!("thread: #{} exiting. #exec: {} exec-time: {} ns {}/s",
      id, exec_count, diff/exec_count, 1_000_000_000/diff);
  }

  pub fn notify(&mut self, id: &task_id::TaskId) -> Result<usize, Error> {
    if self.stop.load(Ordering::SeqCst) {
      return Result::Err(Error::Stopping);
    }
    let max = self.max_id.load(Ordering::SeqCst);
    if id.id() >= max {
      return Result::Err(Error::NonExistent);
    }
    let (l1, l2) = array::position(id.id());
    let l1_slice = self.l1.as_mut_slice();
    let l1_ptr = l1_slice[l1].load(Ordering::SeqCst);
    if l1_ptr.is_null() {
      return Result::Err(Error::NonExistent);
    }
    unsafe { (*l1_ptr).notify(l2) }
  }

  pub fn stop(&mut self) {
    self.stop.store(true, Ordering::SeqCst);
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
