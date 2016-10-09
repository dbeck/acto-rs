
use super::notification::Notification;
use super::super::{SchedulingRule, ChannelId, TaskId};
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::ptr;

#[allow(dead_code)]
pub struct ExecInfo {
  rule:             SchedulingRule,
  ext_notif:        Notification,
  msg_trigger:      Notification,
  last_at:          AtomicUsize,
  next_at:          AtomicUsize,
  last_duration:    AtomicUsize,
  exec_count:       AtomicUsize,
  total_duration:   AtomicUsize,
  output_count:     AtomicUsize,
  dependent_count:  AtomicUsize,
  dependents:       AtomicPtr<Vec<AtomicUsize>>,
}

impl ExecInfo {
  pub fn new() -> ExecInfo {
    ExecInfo{
      rule:             SchedulingRule::Loop,
      ext_notif:        Notification::new(),
      msg_trigger:      Notification::new(),
      last_at:          AtomicUsize::new(0),
      next_at:          AtomicUsize::new(0),
      last_duration:    AtomicUsize::new(0),
      exec_count:       AtomicUsize::new(0),
      total_duration:   AtomicUsize::new(0),
      output_count:     AtomicUsize::new(0),
      dependent_count:  AtomicUsize::new(0),
      dependents:       AtomicPtr::default(),
    }
  }

  pub fn init(&mut self,
              output_count: usize,
              rule: SchedulingRule)
  {
    self.rule = rule;
    let mut boxed = Box::new(Vec::new());
    for _i in 0..output_count {
      boxed.push(AtomicUsize::new(0));
    }
    let old = self.dependents.swap(Box::into_raw(boxed), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
  }

  pub fn register_dependents(&mut self,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let ptr = self.dependents.load(Ordering::Acquire);
    if ptr.is_null() == false {
      let sz = unsafe { (*ptr).len() };
      for dep in deps {
        if sz >= dep.0.sender_id.0 {
          self.dependent_count.fetch_add(1, Ordering::AcqRel);
          let slice = unsafe { (*ptr).as_mut_slice() };
          let dep_task_id = dep.1;
          slice[dep.0.sender_id.0].store(dep_task_id.0, Ordering::Release);
        }
      }
    }
  }

  pub fn ext_notify(&mut self) -> usize {
    self.ext_notif.notify()
  }

  #[allow(dead_code)]
  pub fn msg_notify(&mut self) -> usize {
    self.msg_trigger.notify()
  }

  pub fn ext_flush(&mut self) -> usize {
    self.ext_notif.flush()
  }

  pub fn msg_flush(&mut self) -> usize {
    self.msg_trigger.flush()
  }
}

impl Drop for ExecInfo {
  fn drop(&mut self) {
    let ptr = self.dependents.swap(ptr::null_mut::<Vec<AtomicUsize>>(), Ordering::AcqRel);
    if ptr.is_null() == false {
      // make sure we drop the content of dependents
      let _b = unsafe { Box::from_raw(ptr) };
    }
  }
}
