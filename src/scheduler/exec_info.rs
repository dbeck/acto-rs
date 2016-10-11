
use super::super::{SchedulingRule, ChannelId, TaskId};
use std::sync::atomic::{AtomicUsize, AtomicPtr, Ordering};
use std::ptr;

#[allow(dead_code)]
pub struct ExecInfo {
  rule:             SchedulingRule,
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
  // accessors
  pub fn rule(&self) -> SchedulingRule {
    self.rule
  }
  
  pub fn next_execution_at(&self) -> usize {
    self.next_at.load(Ordering::Acquire)
  }

  pub fn new() -> ExecInfo {
    ExecInfo{
      rule:             SchedulingRule::Loop,
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

  pub fn ext_notify(&mut self) {
    match self.rule {
      SchedulingRule::OnExternalEvent => {
        // set next exec time to 0 to schedule the task
        // for immediate execution
        self.next_at.store(0, Ordering::Release);
      },
      _ => {},
    }
  }

  pub fn msg_notify(&mut self) {
    match self.rule {
      SchedulingRule::OnMessage => {
        // set next exec time to 0 to schedule the task
        // for immediate execution
        self.next_at.store(0, Ordering::Release);
      },
      _ => {},
    }
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
