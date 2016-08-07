extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::scatter::Scatter;
use super::super::common::Schedule;
//use std::collections::VecDeque;
//use std::mem;

pub struct Executor {
  //dummy : i32,
}

impl Scatter for Executor {
  type InputType  = Box<Task + Send>;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          _input:   &mut Receiver<Message<Self::InputType>>,
          _output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule {
    Schedule::Loop
  }
}

pub fn new() -> Executor {
  Executor {
    //dummy: 0,
  }
}
