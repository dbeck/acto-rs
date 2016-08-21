use super::super::{Task, Reporter, Schedule};
use super::event;

pub struct TaskWrap {
  task: Box<Task+Send>,
  event: event::Event,
  ev_ticket: u64,
}

impl TaskWrap {
  pub fn execute(&mut self, reporter: &mut Reporter) -> Schedule {
    self.task.execute(reporter)
  }
}

pub fn new(task: Box<Task+Send>) -> TaskWrap {
  TaskWrap{
    task: task,
    event: event::new(),
    ev_ticket: 0,
  }
}
