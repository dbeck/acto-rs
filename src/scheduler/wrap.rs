use super::super::{Task, Reporter, Schedule};
use std::sync::atomic::{AtomicUsize, Ordering};

enum State {
  Loop,
  TimeWait(usize),
  MessageWait(usize),
  ExtEventWait(usize),
}

pub struct TaskWrap {
  task: Box<Task+Send>,
  ext_evt_count: AtomicUsize,
  state: State,
}

impl TaskWrap {
  pub fn execute(&mut self, reporter: &mut Reporter, time_us: &AtomicUsize) -> Schedule {
    let _start = time_us.load(Ordering::Acquire);
    let ret = self.task.execute(reporter);
    let _end = time_us.load(Ordering::Acquire);
    ret
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::SeqCst)
  }
}

pub fn new(task: Box<Task+Send>) -> TaskWrap {
  TaskWrap{
    task: task,
    ext_evt_count: AtomicUsize::new(0),
    state: State::Loop,
  }
}
