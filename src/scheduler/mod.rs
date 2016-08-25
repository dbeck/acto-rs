mod wrap;
mod array;
mod task_id;
mod data;
mod handle;
//mod event;

use super::{Task, Observer, Error, ChannelId, TaskState};
use std::thread::{spawn, JoinHandle};

#[allow(dead_code)]
pub struct Scheduler {
  data:     handle::SchedulerDataHandle,
  threads:  Vec<JoinHandle<()>>,
}

// L1: 64k entries preallocated
// L2: 4k entries on-demand
impl Scheduler {

  pub fn add_task(&mut self, task: Box<Task+Send>) -> task_id::TaskId {
    (*self.data.get()).add_task(task)
  }

  pub fn start(&mut self) {
    self.start_with_threads(1);
  }

  pub fn notify(&mut self, id: &task_id::TaskId) -> Result<usize, Error> {
    (*self.data.get()).notify(id)
  }

  pub fn start_with_threads(&mut self, n_threads: usize) {
    if n_threads == 0 {
      return;
    }

    for i in 0..n_threads {
      let mut data_handle = self.data.clone();
      let t = spawn(move || { data_handle.get().entry(i); });
      self.threads.push(t);
    }

    {
      let mut data_handle = self.data.clone();
      let t = spawn(move || { data_handle.get().ticker(); });
      self.threads.push(t);
    }
  }

  pub fn stop(&mut self) {
    (*self.data.get()).stop();
    loop {
      match self.threads.pop() {
        Some(t) => {
          t.join().unwrap();
        },
        None => {
          break;
        }
      }
    }
  }
}

pub fn new() -> Scheduler {
  Scheduler{
    data:     handle::new(),
    threads:  Vec::new(),
  }
}

//////////////////////////////////////////////////////
// Old/Slow Implementation Below
//////////////////////////////////////////////////////

pub struct CountingReporter {
  pub executed: usize,
  pub stopped: usize,
  pub delayed: usize,
  pub time_wait: usize,
  pub msg_wait: usize,
  pub ext_wait: usize,
  pub sent : usize,
  pub channel : usize,
}

impl CountingReporter {
  pub fn new() -> CountingReporter {
    CountingReporter{
      executed:    0,
      stopped:     0,
      delayed:     0,
      time_wait:   0,
      msg_wait:    0,
      ext_wait:    0,
      sent :       0,
      channel :    0,
    }
  }
}

impl Observer for CountingReporter {
  fn executed(&mut self, _task_id: usize) {
    self.executed += 1;
  }
  fn stopped(&mut self, _task_id: usize) {
    self.stopped += 1;
  }
  fn delayed(&mut self, _task_id: usize, reason: &TaskState) {
    self.delayed += 1;
    match reason {
      &TaskState::TimeWait(_)      => { self.time_wait += 1; },
      &TaskState::MessageWait(_,_) => { self.msg_wait += 1; },
      &TaskState::ExtEventWait(_)  => { self.ext_wait += 1; },
      _ => {}
    }
  }
  fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize, _task_id: usize) {
    self.sent += 1;
  }
  fn wait_channel(&mut self, _channel_id: &ChannelId, _last_msg_id: usize, _task_id: usize) {
    self.channel += 1;
  }
}

#[cfg(test)]
pub mod tests;
