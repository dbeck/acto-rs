
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
    // TODO : check idx
    unsafe {
      let l2_atomic_ptr = self.l2.get_unchecked_mut(idx);
      l2_atomic_ptr.store(Box::into_raw(wrap), Ordering::SeqCst);
    }
  }

  pub fn eval(&mut self,
                 l2_max_idx: usize,
                 id: usize,
                 observer: &mut Observer,
                 time_us: &AtomicUsize) -> u64 {
    let l2_slice = self.l2.as_mut_slice();
    let mut skip = id;
    let mut l2idx = 0;
    let mut exec_count = 0;
    loop {
      if l2idx > l2_max_idx { break; }
      let wrk = l2_slice[l2idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::SeqCst);
      if wrk.is_null() == false {
        unsafe { (*wrk).eval(observer, &time_us); }
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

  pub fn notify(&mut self, l2_idx: usize) -> Result<usize, Error> {
    let l2_slice = self.l2.as_mut_slice();
    let wrk = l2_slice[l2_idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::SeqCst);
    if wrk.is_null() == false {
      let ret = unsafe { (*wrk).notify() };
      l2_slice[l2_idx].store(wrk, Ordering::SeqCst);
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
      let ptr = l2_atomic_ptr.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::SeqCst);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      }
    }
  }
}
