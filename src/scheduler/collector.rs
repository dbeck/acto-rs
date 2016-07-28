extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use std::collections::VecDeque;
use super::super::common::{Message};
use super::super::elem::gather::Gather;
use super::super::common::Schedule;

struct Collector {
  overflow: VecDeque<usize>,
}

impl Gather for Collector {
  type InputType  = usize;
  type OutputType = usize;

  fn process(
          &mut self,
          _input_vec:   Vec<&mut Receiver<Message<Self::InputType>>>,
          _output:      &mut Sender<Message<Self::OutputType>>) -> Schedule {
    Schedule::Loop
  }
}

  //fn process(
  //      &mut self,
  //      input: &mut Receiver<Message<Self::InputType>>,
  //      output: &mut Sender<Message<Self::OutputType>>)
  //    -> common::Schedule {
  //  for i in input.iter() {
  //    match i {
  //      Message::Value(v) => {
  //        self.state = v;
  //        output.put(|x| *x = Message::Value(self.state));
  //      }
  //      _ => { println!("Unknown value"); }
  //    }
  //  }
  //  common::Schedule::Loop
  //}
