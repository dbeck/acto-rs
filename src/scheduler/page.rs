
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, TaskId, ChannelId};
use super::observer::{Observer};
use super::{wrap};
use std::ptr;

struct Notification {
  pending:   AtomicUsize,
  delivered: AtomicUsize,
}

pub struct TaskPage {
  l2:           Vec<AtomicPtr<wrap::TaskWrap>>,
  ext_notif:    Vec<Notification>,
  msg_trigger:  Vec<Notification>,
}

pub fn max_idx() -> usize {
  4095
}

pub fn position(idx: usize) -> (usize, usize) {
  // note: this depends on max_idx !!!
  (idx>>12, idx&0xfff)
}

impl TaskPage {
  pub fn store(&mut self, idx: usize, task: Box<Task+Send>, id: TaskId, input_task_ids: Vec<Option<usize>>) {
    let wrap = Box::new(wrap::new(task, id, input_task_ids));
    let slice = self.l2.as_mut_slice();
    let old = slice[idx].swap(Box::into_raw(wrap), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
  }

  pub fn blocking_apply<F>(&self, l2_idx: usize, f: F) where F : FnMut(*mut wrap::TaskWrap) {
    let slice = self.l2.as_slice();
    loop {
      let wrk = slice[l2_idx].swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if wrk.is_null() == false {
        let mut f = f;
        f(wrk);
        slice[l2_idx].store(wrk, Ordering::Release);
        break;
      }
    }
  }

  pub fn eval(&mut self,
                 l2_max_idx: usize,
                 id: usize,
                 observer: &mut Observer,
                 time_us: &AtomicUsize) {
    let mut skip = id;
    let mut l2_idx = 0;
    loop {
      if l2_idx >= l2_max_idx { break; }
      let wrk_ref = unsafe { self.l2.get_unchecked_mut(l2_idx) };
      let wrk = wrk_ref.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if wrk.is_null() == false {

        let ext_notif_slice = self.ext_notif.as_mut_slice();
        let ext_notif_ref = &mut ext_notif_slice[l2_idx];
        let ext_notif_diff = ext_notif_ref.pending.load(Ordering::Acquire) - ext_notif_ref.delivered.load(Ordering::Acquire);

        let msg_trig_slice = self.msg_trigger.as_mut_slice();
        let msg_trig_ref = &mut msg_trig_slice[l2_idx];
        let msg_trig_diff = msg_trig_ref.pending.load(Ordering::Acquire) - msg_trig_ref.delivered.load(Ordering::Acquire);

        unsafe {
          // deliver external notifications
          if ext_notif_diff > 0 {
            /* XXX
            (*wrk).ext_notify(ext_notif_diff, observer, &time_us);
            */
            ext_notif_ref.delivered.fetch_add(ext_notif_diff, Ordering::AcqRel);
          }

          // deliver message triggers
          if msg_trig_diff > 0 {
            /* XXX
            (*wrk).msg_trigger(observer, &time_us);
            */
            msg_trig_ref.delivered.fetch_add(msg_trig_diff, Ordering::AcqRel);
          }

          /* XXX
          (*wrk).eval(observer, &time_us);
          */
          (*wrk).eval(&time_us);
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
    let not_slice = self.ext_notif.as_mut_slice();
    let res = not_slice[l2_idx].pending.fetch_add(1, Ordering::AcqRel);
    res + 1
  }

  /* XXX !!!
  pub fn msg_trigger(&mut self, l2_idx: usize) {
    let trig_slice = self.msg_trigger.as_mut_slice();
    trig_slice[l2_idx].pending.fetch_add(1, Ordering::AcqRel);
  }
  */

  /* XXX
  pub fn resolve_input_task_id(&mut self, l2_idx: usize, sender: &TaskId, channel: &ChannelId) {
    self.blocking_apply(l2_idx, |task_wrapper| {
      unsafe { (*task_wrapper).resolve_input_task_id(*channel, *sender) };
    });
  }
  */

  /* XXX
  pub fn register_dependent(&mut self, l2_idx: usize, ch: ChannelId, dep_task_id: TaskId) {
    self.blocking_apply(l2_idx, |task_wrapper| {
      unsafe { (*task_wrapper).register_dependent(ch, dep_task_id) };
    });
  }
  */

  #[cfg(feature = "printstats")]
  fn print_stats(&self) {}

  #[cfg(not(feature = "printstats"))]
  fn print_stats(&self) {}
}

pub fn new() -> TaskPage {
  let sz               = max_idx()+1;
  let mut l2           = Vec::with_capacity(sz);
  let mut ext_notif    = Vec::with_capacity(sz);
  let mut msg_trigger  = Vec::with_capacity(sz);

  for _i in 0..sz {
    l2.push(AtomicPtr::default());
    ext_notif.push(Notification{
      pending: AtomicUsize::new(0),
      delivered: AtomicUsize::new(0)}
    );
    msg_trigger.push(Notification{
      pending: AtomicUsize::new(0),
      delivered: AtomicUsize::new(0)}
    );
  }

  TaskPage{
    l2: l2,
    ext_notif: ext_notif,
    msg_trigger: msg_trigger,
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
