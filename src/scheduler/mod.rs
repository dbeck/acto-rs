mod task_and_outputs;
mod task_page;
mod scheduler_impl;
mod scheduler_impl_handle;
mod thread_private;

use super::{Task, Error, TaskId, SchedulingRule};
use std::thread::{spawn, JoinHandle};

pub struct Scheduler {
  data:     scheduler_impl_handle::SchedulerImplHandle,
  threads:  Vec<JoinHandle<()>>,
}

impl Scheduler {
  fn notify_one(&mut self, idx: usize) {
    let n_threads = self.threads.len();
    if idx < n_threads {
      self.threads[idx].thread().unpark();
    }
  }

  fn notify_all(&mut self) {
    for t in &self.threads {
      t.thread().unpark();
    }
  }

  pub fn add_task(&mut self,
                  task: Box<Task+Send>,
                  rule: SchedulingRule)
    -> Result<TaskId, Error>
  {
    let result = (*self.data.get()).add_task(task, rule);
    self.notify_one(0);
    result
  }

  pub fn start(&mut self) {
    self.start_with_threads(1);
  }

  pub fn notify(&mut self, id: &TaskId)
      -> Result<(), Error>
  {
    let result = (*self.data.get()).notify(id);
    self.notify_one(0);
    result
  }

  pub fn start_with_threads(&mut self, n_threads: usize)
  {
    if n_threads == 0 {
      return;
    }

    // exec threads that don't care about external events
    for _i in 0..n_threads {
      let mut data_handle = self.data.clone();
      let id = self.threads.len();
      let t = spawn(move || { data_handle.get().scheduler_thread_entry(id); });
      self.threads.push(t);
    }

    {
      let mut data_handle = self.data.clone();
      let t = spawn(move || { data_handle.get().ticker_thread_entry(); });
      self.threads.push(t);
    }
  }

  pub fn stop(&mut self) {
    (*self.data.get()).stop();
    self.notify_all();
    while let Some(t) = self.threads.pop() {
      t.join().unwrap();
    }
  }

  pub fn new() -> Scheduler {
    Scheduler{
      data:     scheduler_impl_handle::new(),
      threads:  Vec::new(),
    }
  }
}

pub fn new() -> Scheduler {
  Scheduler::new()
}

#[cfg(test)]
pub mod tests;
