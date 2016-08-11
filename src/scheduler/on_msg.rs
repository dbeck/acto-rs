use lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::filter::Filter;
use super::super::common::Schedule;
use super::executor::TaskResults;
//use std::collections::VecDeque;
//use std::mem;

pub struct OnMsg {
  // dummyx: usize,
}

impl Filter for OnMsg {
  type InputType  = TaskResults;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          _input:   &mut Receiver<Message<Self::InputType>>,
          _output:  &mut Sender<Message<Self::OutputType>>) -> Schedule {
    Schedule::Loop
  }
}

pub fn new() -> OnMsg {
  OnMsg {
    // dummyx: 0,
  }
}
