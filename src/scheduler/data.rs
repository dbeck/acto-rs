
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
use super::super::{Task};
use super::{array, task_id};
use time;

pub struct SchedulerData {
  max_id:   AtomicUsize,
  l1:       Vec<AtomicPtr<array::TaskArray>>,
  stop:     AtomicBool,
}

impl SchedulerData {
  fn add_l2_bucket(&mut self, idx: usize) {
    let array = Box::new(array::new());
    let l1_slice = self.l1.as_mut_slice();
    l1_slice[idx].store(Box::into_raw(array), Ordering::SeqCst);
  }

  fn position(&self, idx: usize) -> (usize, usize) {
    (idx>>12, idx&0xfff)
  }

  fn new() -> SchedulerData {
    let mut data = SchedulerData{
      max_id:   AtomicUsize::new(0),
      l1:       Vec::with_capacity(65536),
      stop:     AtomicBool::new(false),
    };

    // fill the l1 bucket
    for _i in 0..65536 {
      data.l1.push(AtomicPtr::default());
    }

    // add an initial l2 bucket
    data.add_l2_bucket(0);
    data
  }

  pub fn add_task(&mut self, task: Box<Task+Send>) -> task_id::TaskId {
    let ret = task_id::new(self.max_id.fetch_add(1, Ordering::SeqCst));
    let (l1, l2) = self.position(ret.id());
    if l2 == 0 {
      // make sure the next bucket exists when needed
      self.add_l2_bucket(l1+1);
    }
    //if l1 == 65534 {
      // TODO:
    //}
    unsafe {
      let l1_ptr = self.l1.get_unchecked_mut(l1).load(Ordering::SeqCst);
      if l1_ptr.is_null() == false {
        (*l1_ptr).store(l2, task);
      }
    }
    ret
  }

  pub fn entry(&mut self, id: usize) {
    let mut exec_count : u64 = 0;
    let start = time::precise_time_ns();
    loop {
      if self.stop.load(Ordering::SeqCst) {
        break;
      }
      let (l1, l2) = self.position(self.max_id.load(Ordering::SeqCst));
      let l1_slice = self.l1.as_mut_slice();
      for l1_idx in 0..(l1+1) {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::SeqCst);
        let mut l2_max_idx = array::max_idx();
        if l1_idx == l1 {
          l2_max_idx = l2;
        }
        unsafe { exec_count += (*l1_ptr).execute(l2_max_idx, id); }
      }
    }
    let end = time::precise_time_ns();
    let diff = end - start;
    println!("thread: #{} exiting. #exec: {} exec-time: {} ns", id, exec_count, diff/exec_count);
  }

  pub fn stop(&mut self) {
    self.stop.store(true, Ordering::SeqCst);
  }
}

pub fn new() -> SchedulerData {
  SchedulerData::new()
}
