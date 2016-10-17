
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, ChannelId, TaskId};
use super::prv::{Private};
use super::{wrap};
use std::ptr;

struct ExecFlags (AtomicUsize);

pub struct TaskPage {
  data:    Vec<(AtomicPtr<wrap::TaskWrap>, ExecFlags)>,
  id:      usize,
}

pub fn max_idx() -> usize {
  // note: this must be aligned with position(idx)
  4095
}

pub fn position(idx: usize) -> (usize, usize) {
  // note: this depends on max_idx !!!
  (idx>>12, idx&0xfff)
}

impl TaskPage {
  pub fn store(&mut self,
               idx: usize,
               task: Box<Task+Send>)
  {
    let wrap = Box::new(wrap::new(task));
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let old = data_ref.0.swap(Box::into_raw(wrap), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
    // zero the flags too
    (data_ref.1).0.store(0, Ordering::Release);
  }

  pub fn register_dependents(&mut self,
                             idx: usize,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let atomic_flags = &mut data_ref.1;
    // TODO
    let flags = atomic_flags.load(Ordering::Acquire); 
  }

  pub fn set_dependents_flag(&mut self, idx: usize, multi_deps: bool) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let flags = &mut data_ref.1;
    if multi_deps {
      flags.0.fetch_or(3, Ordering::Release);
    } else {
      let old = flags.0.fetch_or(1, Ordering::AcqRel);
      if old&1 == 1 {
        flags.0.fetch_or(3, Ordering::Release);
      }
    }
  }

  pub fn set_conditional_exec_flag(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let flags = &mut data_ref.1;
    flags.0.fetch_or(32, Ordering::Release);
  }

  pub fn notify(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let flags = &(data_ref.1).0.load(Ordering::Acquire);
    // clear next_exec_time and set notified flag
    let new_flags = (flags&63) | 4;
    let mut_flags = &mut data_ref.1;
    mut_flags.0.store(new_flags, Ordering::Release);
  }

  #[allow(dead_code)]
  pub fn trigger(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let flags = &(data_ref.1).0.load(Ordering::Acquire);
    // clear next_exec_time and set triggered flag
    let new_flags = (flags&63) | 8;
    let mut_flags = &mut data_ref.1;
    mut_flags.0.store(new_flags, Ordering::Release);
  }

  pub fn eval(&mut self,
              l2_max_idx: usize,
              exec_thread_id: usize,
              private_data: &mut Private,
              time_us: &AtomicUsize)
  {
    let mut skip    = exec_thread_id;
    let mut l2_idx  = 0;
    let mut now     = time_us.load(Ordering::Acquire);

    for i in &mut self.data {
      if l2_idx >= l2_max_idx { break; }
      let flags = &(i.1).0.load(Ordering::Acquire);
      let stopped = flags&16;
      // execute if not stopped and time is OK
      if stopped == 0 {
        let next_execution_at = flags >> 6;
        if next_execution_at <= now {
          let wrk = i.0.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
          if !wrk.is_null() {
            unsafe {
              let has_dependents = flags&1 == 1;
              let is_conditional = flags&32 == 32;
              let mut stop = false;
              (*wrk).execute(has_dependents, &mut stop, private_data);
              let mut_flags = &mut i.1;
              let end = time_us.load(Ordering::Acquire);
              if stop {
                // the task said to be stopped, so set the stop bit
                mut_flags.0.fetch_or(16, Ordering::Release);
              }
              if is_conditional {
                // for conditionally executed tasks that:
                // 1, wait for external notification
                // 2, wait for message
                // -> set exec time to 10s ahead
                // -> add back original dependent and conditional flags
                let new_flags : usize = (end+10_000_000)<<6 | (flags&3) | 32;
                mut_flags.0.store(new_flags, Ordering::Release);
              }
              now = end;
            }
            i.0.store(wrk, Ordering::Release);
          } else {
            l2_idx += skip;
            skip += exec_thread_id;
          }
        }
      }
      l2_idx += 1;
    }
  }

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {
    let mut pos = 0;
    for i in &self.data {
      let ptr = i.0.load(Ordering::Acquire);
      if !ptr.is_null() {
        let flags = (i.1).0.load(Ordering::Acquire);
        let has_dependents = flags&1 == 1;
        let multi_deps = flags&2 == 2;
        let notified = flags&4 == 4;
        let triggered = flags&8 == 8;
        let stopped = flags&16 == 16;
        let conditional = flags&32 == 32;
        let next_execution_at = flags>>6;
        println!("#{}:{} dep:{:?}/m:{:?} not:{:?} trg:{:?} stop:{:?} cond:{:?} next:{}",
          self.id, pos, has_dependents, multi_deps, notified, triggered,
          stopped, conditional,
          next_execution_at);
      }
      pos += 1;
    }
  }

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new(id: usize) -> TaskPage {
  let sz               = max_idx()+1;
  let mut data         = Vec::with_capacity(sz);

  for _i in 0..sz {
    // default flag is stopped: 16
    let f = ExecFlags(AtomicUsize::new(16));
    data.push( (AtomicPtr::default(), f) );
  }

  TaskPage{
    data:   data,
    id:     id,
  }
}

impl Drop for TaskPage {
  fn drop(&mut self) {
    self.print_stats();
    let slice = self.data.as_mut_slice();
    for i in 0..(1+max_idx()) {
      let data_ref = &mut slice[i];
      let ptr = data_ref.0.swap(
        ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      }
    }
  }
}
