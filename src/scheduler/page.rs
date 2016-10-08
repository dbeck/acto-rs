
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, SchedulingRule, TaskId, ChannelId};
use super::observer::{Observer};
use super::notification::{Notification};
use super::exec_info::{ExecInfo};
use super::{wrap};
use std::ptr;

pub struct TaskPage {
  l2:           Vec<AtomicPtr<wrap::TaskWrap>>,
  info:         Vec<ExecInfo>,
}

pub fn max_idx() -> usize {
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
    let slice = self.l2.as_mut_slice();
    let old = slice[idx].swap(Box::into_raw(wrap), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
  }

  pub fn init_info(&mut self,
                   idx: usize,
                   output_count: usize,
                   rule: SchedulingRule)
  {
    let slice = self.info.as_mut_slice();
    slice[idx].init(output_count, rule);
  }

  pub fn register_dependents(&mut self,
                             idx: usize,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let slice = self.info.as_mut_slice();
    slice[idx].register_dependents(deps);
  }

  pub fn eval(&mut self,
                 l2_max_idx: usize,
                 id: usize,
                 _observer: &mut Observer,
                 time_us: &AtomicUsize) {
    let mut skip = id;
    let mut l2_idx = 0;
    loop {
      if l2_idx >= l2_max_idx { break; }
      let wrk_ref = unsafe { self.l2.get_unchecked_mut(l2_idx) };
      let wrk = wrk_ref.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if wrk.is_null() == false {

        let info_slice      = self.info.as_mut_slice();
        let info_ref        = &mut info_slice[l2_idx];
        let ext_notif_diff  = info_ref.ext_flush();
        let msg_trig_diff   = info_ref.msg_flush();

        unsafe {
          // deliver external notifications
          if ext_notif_diff > 0 {
          }

          // deliver message triggers
          if msg_trig_diff > 0 {
          }

          let start = time_us.load(Ordering::Acquire);
          let _result = (*wrk).execute();
          let _took = time_us.load(Ordering::Acquire) - start;
        }
        wrk_ref.store(wrk, Ordering::Release);
      } else {
        l2_idx += skip;
        skip += id;
      }
      l2_idx += 1;
    }
  }

  pub fn notify(&mut self, l2_idx: usize) -> usize {
    let info_slice = self.info.as_mut_slice();
    info_slice[l2_idx].ext_notify()
  }

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {}

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new() -> TaskPage {
  let sz               = max_idx()+1;
  let mut l2           = Vec::with_capacity(sz);
  let mut ext_notif    = Vec::with_capacity(sz);
  let mut msg_trigger  = Vec::with_capacity(sz);
  let mut info         = Vec::with_capacity(sz);

  for _i in 0..sz {
    l2.push(AtomicPtr::default());
    ext_notif.push(Notification::new());
    msg_trigger.push(Notification::new());
    info.push(ExecInfo::new());
  }

  TaskPage{
    l2:           l2,
    info:         info,
  }
}

impl Drop for TaskPage {
  fn drop(&mut self) {
    self.print_stats();
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
