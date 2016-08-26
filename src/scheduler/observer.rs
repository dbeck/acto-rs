use super::super::{Observer, TaskState, ChannelId, Schedule};

pub struct CountingReporter {
  pub scheduled:   usize,
  pub executed:    usize,
  pub stopped:     usize,
  pub delayed:     usize,
  pub time_wait:   usize,
  pub msg_wait:    usize,
  pub ext_wait:    usize,
  pub sent:        usize,
  pub channel:     usize,
  pub transition:  usize,
}

impl CountingReporter {
  pub fn new() -> CountingReporter {
    CountingReporter{
      scheduled:   0,
      executed:    0,
      stopped:     0,
      delayed:     0,
      time_wait:   0,
      msg_wait:    0,
      ext_wait:    0,
      sent:        0,
      channel:     0,
      transition:  0,
    }
  }
}

impl Observer for CountingReporter {
  fn scheduled(&mut self, _task_id: usize, _at_usec: usize) {
    self.scheduled += 1;
  }
  fn executed(&mut self, _task_id: usize, _at_usec: usize) {
    self.executed += 1;
  }
  fn stopped(&mut self, _task_id: usize, _at_usec: usize) {
    self.stopped += 1;
  }
  fn delayed(&mut self, _task_id: usize, reason: &TaskState, _at_usec: usize) {
    self.delayed += 1;
    match reason {
      &TaskState::TimeWait(_)       => { self.time_wait += 1; },
      &TaskState::MessageWait(_,_)  => { self.msg_wait += 1; },
      &TaskState::ExtEventWait(_)   => { self.ext_wait += 1; },
      _ => {}
    }
  }
  fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize, _task_id: usize, _at_usec: usize) {
    self.sent += 1;
  }
  fn wait_channel(&mut self, _channel_id: &ChannelId, _last_msg_id: usize, _task_id: usize, _at_usec: usize) {
    self.channel += 1;
  }
  fn transition(&mut self, _from: &TaskState, _event: &Schedule, _to: &TaskState, _task_id: usize, _at_usec: usize) {
    self.transition += 1;
  }
}

pub struct TaskTracer { }

impl TaskTracer {
  pub fn new() -> TaskTracer {
    TaskTracer{}
  }
}

impl Observer for TaskTracer {
  fn scheduled(&mut self, task_id: usize, at_usec: usize) {
    println!("Task scheduled: {} @{}_us", task_id, at_usec);
  }
  fn executed(&mut self, task_id: usize, at_usec: usize) {
    println!("Task executed: {} @{}_us", task_id, at_usec);
  }
  fn stopped(&mut self, task_id: usize, at_usec: usize) {
    println!("Task stopped: {} @{}_us", task_id, at_usec);
  }
  fn delayed(&mut self, task_id: usize, reason: &TaskState, at_usec: usize) {
    println!("Task delayed: {} reason: {:?} @{}_us", task_id, reason, at_usec);
  }
  fn message_sent(&mut self, channel_id: usize, last_msg_id: usize, task_id: usize, at_usec: usize) {
    println!("Task message_sent: channel_id: {} last_msg_id: {} task_id: {} @{}_us",
      channel_id, last_msg_id, task_id, at_usec);
  }
  fn wait_channel(&mut self, channel_id: &ChannelId, last_msg_id: usize, task_id: usize, at_usec: usize) {
    println!("Task wait channel: channel_id: {}, last_msg_id: {} task_id: {} @{}_us",
      channel_id, last_msg_id, task_id, at_usec);
  }
  fn transition(&mut self, from: &TaskState, event: &Schedule, to: &TaskState, task_id: usize, at_usec: usize) {
    println!("Task transition: from: {:?} + event: {:?} => to: {:?} | task_id: {} @{}_us",
      from, event, to, task_id, at_usec);
  }
}
