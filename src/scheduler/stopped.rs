extern crate lossyq;

use self::lossyq::spsc::{Receiver};
use super::super::common::{Task, Message};
use super::super::elem::sink::Sink;
use super::super::common::Schedule;
//use std::collections::VecDeque;
//use std::mem;

pub struct Stopped {
  dummy : i32,
}

impl Sink for Stopped {
  type InputType  = Box<Task + Send>;

  fn process(
          &mut self,
          _input:   &mut Receiver<Message<Self::InputType>>) -> Schedule {
    Schedule::Loop
  }
}

pub fn new() -> Stopped {
  Stopped {
    dummy: 0,
  }
}
