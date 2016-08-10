extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::scatter::Scatter;
use super::super::common::Schedule;
//use std::collections::VecDeque;
//use std::mem;

pub struct TaskResults {
  pub task      : Box<Task + Send>,
  pub schedule  : Schedule,
}

pub struct Executor {
  //dummy : i32,
}

impl Scatter for Executor {
  type InputType  = Box<Task + Send>;
  type OutputType = TaskResults;

  fn process(
          &mut self,
          input:   &mut Receiver<Message<Self::InputType>>,
          output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule {

    for _i in input.iter() {
      /*match i {

      }*/
    }
    Schedule::Loop
  }
}

pub fn new() -> Executor {
  Executor {
    //dummy: 0,
  }
}
