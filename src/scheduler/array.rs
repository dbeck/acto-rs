
use std::sync::atomic::{AtomicPtr, Ordering};
use super::super::common::{Task};
use super::{wrap, CountingReporter};
use std::ptr;

pub struct TaskArray {
  l2: Vec<AtomicPtr<wrap::TaskWrap>>,
}

impl TaskArray {
  pub fn store(&mut self, idx: usize, task: Box<Task+Send>) {
    let wrap = Box::new(wrap::new(task));
    // TODO : check idx
    unsafe {
      let l2_atomic_ptr = self.l2.get_unchecked_mut(idx);
      l2_atomic_ptr.store(Box::into_raw(wrap), Ordering::SeqCst);
    }
  }

  pub fn execute(&mut self, l2_max_idx: usize, id: usize) -> u64 {
    let l2_slice = self.l2.as_mut_slice();
    let mut skip = id;
    let mut l2idx = 0;
    let mut exec_count = 0;
    loop {
      if l2idx > l2_max_idx { break; }
      let wrk = l2_slice[l2idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::SeqCst);
      if wrk.is_null() == false {
        let mut reporter = CountingReporter{ count: 0 };
        unsafe { (*wrk).execute(&mut reporter); }
        l2_slice[l2idx].store(wrk, Ordering::SeqCst);
        exec_count += 1;
      } else {
        l2idx += skip;
        skip += id;
      }
      l2idx += 1;
    }
    exec_count
  }
}

pub fn new() -> TaskArray {
  let mut bucket = Vec::with_capacity(4096);
  for _i in 0..(4096) {
    bucket.push(AtomicPtr::default());
  }
  TaskArray{ l2: bucket }
}

pub fn max_idx() -> usize {
  4095
}
