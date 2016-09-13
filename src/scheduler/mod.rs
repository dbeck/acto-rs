mod wrap;
mod array;
mod task_id;
mod data;
mod handle;
mod observer;
pub mod event;

use super::{Task, Error};
use std::thread::{spawn, JoinHandle};

#[allow(dead_code)]
pub struct Scheduler {
  data:     handle::SchedulerDataHandle,
  threads:  Vec<JoinHandle<()>>,
}

// L1: 64k entries preallocated
// L2: 4k entries on-demand
impl Scheduler {

  pub fn add_task(&mut self, task: Box<Task+Send>) -> Result<task_id::TaskId, Error> {
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
    while let Some(t) = self.threads.pop() {
      t.join().unwrap();
    }
  }
}

pub fn new() -> Scheduler {
  Scheduler{
    data:     handle::new(),
    threads:  Vec::new(),
  }
}

#[cfg(test)]
pub mod tests;
