
use super::notification::Notification;
use super::super::{SchedulingRule, ChannelId, TaskId};
use std::sync::atomic::{AtomicUsize, AtomicPtr};

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

  #[allow(dead_code)]
  pub fn init(&mut self, _output_count: usize, rule: SchedulingRule) {
    // TODO : output count
    self.rule = rule;
  }

  #[allow(dead_code)]
  pub fn register_dependent(&mut self, _channel: ChannelId, _dep_id: TaskId) {
    // TODO : register dependent
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
