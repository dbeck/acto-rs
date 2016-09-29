mod wrap;
mod page;
mod data;
mod handle;
mod observer;
mod state;
mod notification;
pub mod event;

use super::{Task, Error, TaskId, SchedulingRule};
use std::thread::{spawn, JoinHandle};

#[allow(dead_code)]
pub struct Scheduler {
  data:     handle::SchedulerDataHandle,
  threads:  Vec<JoinHandle<()>>,
}

impl Scheduler {

  pub fn add_task(&mut self, task: Box<Task+Send>, rule: SchedulingRule) -> Result<TaskId, Error> {
    (*self.data.get()).add_task(task, rule)
  }

  pub fn start(&mut self) {
    self.start_with_threads(1);
  }

  pub fn notify(&mut self, id: &TaskId) -> Result<usize, Error> {
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
    while let Some(t) = self.threads.pop() {
      t.join().unwrap();
    }
  }

  pub fn new() -> Scheduler {
    Scheduler{
      data:     handle::new(),
      threads:  Vec::new(),
    }
  }
}

pub fn new() -> Scheduler {
  Scheduler::new()
}

#[cfg(test)]
pub mod tests;
