mod event;
mod wrap;
mod array;
mod task_id;
mod data;
mod handle;

use super::common::{Task, Reporter};
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

  pub fn start_with_threads(&mut self, n_threads: usize) {
    if n_threads == 0 {
      return;
    }
    for i in 0..n_threads {
      let mut data_handle = self.data.clone();
      let t = spawn(move || { data_handle.get().entry(i); });
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
  pub count : usize,
}

impl Reporter for CountingReporter {
  fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize) {
    self.count += 1;
  }
}

#[cfg(test)]
pub mod tests;
