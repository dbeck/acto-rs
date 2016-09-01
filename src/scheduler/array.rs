
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, Error, Observer};
use super::{wrap};
use std::ptr;

pub struct TaskArray {
  l2: Vec<AtomicPtr<wrap::TaskWrap>>,
}

impl TaskArray {
  pub fn store(&mut self, idx: usize, task: Box<Task+Send>, id: usize) {
    let wrap = Box::new(wrap::new(task, id));
    let slice = self.l2.as_mut_slice();
    let old = slice[idx].swap(Box::into_raw(wrap), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
  }

  pub fn eval(&mut self,
                 l2_max_idx: usize,
                 id: usize,
                 observer: &mut Observer,
                 time_us: &AtomicUsize) {
    let l2_slice = self.l2.as_mut_slice();
    let mut skip = id;
    let mut l2idx = 0;
    loop {
      if l2idx > l2_max_idx { break; }
      let wrk = l2_slice[l2idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if wrk.is_null() == false {
        unsafe { (*wrk).eval(observer, &time_us); }
        l2_slice[l2idx].store(wrk, Ordering::Release);
      } else {
        l2idx += skip;
        skip += id;
      }
      l2idx += 1;
    }
  }

  pub fn notify(&mut self, l2_idx: usize) -> Result<usize, Error> {
    let l2_slice = self.l2.as_mut_slice();
    let wrk = l2_slice[l2_idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
    if wrk.is_null() == false {
      let ret = unsafe { (*wrk).notify() };
      l2_slice[l2_idx].store(wrk, Ordering::Release);
      Result::Ok(ret)
    } else {
      Result::Err(Error::Busy)
    }
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

pub fn position(idx: usize) -> (usize, usize) {
  // note: this depends on max_idx !!!
  (idx>>12, idx&0xfff)
}

impl Drop for TaskArray {
  fn drop(&mut self) {
    let l2_slice = self.l2.as_mut_slice();
    for i in 0..(1+max_idx()) {
      let l2_atomic_ptr = &mut l2_slice[i];
      let ptr = l2_atomic_ptr.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      }
    }
  }
}
