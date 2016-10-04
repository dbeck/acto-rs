
use super::notification::Notification;
use super::super::SchedulingRule;
use std::sync::atomic::AtomicUsize;

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
    }
  }

  #[allow(dead_code)]
  pub fn init(&mut self, rule: SchedulingRule) {
    self.rule = rule;
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
