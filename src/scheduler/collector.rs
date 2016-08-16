use lossyq::spsc::{noloss, Sender};
use super::super::common::{Task, Message, IdentifiedReceiver};
use super::super::elem::gather::Gather;
use super::super::common::Schedule;
use std::collections::VecDeque;
use std::mem;

pub struct Collector {
  overflow: VecDeque<Message<Box<Task + Send>>>,
  tmp_overflow: VecDeque<Message<Box<Task + Send>>>,
}

impl noloss::Overflow for Collector {
  type Input = Message<Box<Task + Send>>;

  fn overflow(&mut self, val : &mut Option<Self::Input>) {
    let mut tmp : Option<Self::Input> = None;
    mem::swap(&mut tmp, val);
    match tmp {
      Some(v) => {
        self.tmp_overflow.push_back(v);
      },
      None => {}
    }
  }
}

impl Gather for Collector {
  type InputType  = Box<Task + Send>;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          input_vec:   &mut Vec<Option<IdentifiedReceiver<Self::InputType>>>,
          output:      &mut Sender<Message<Self::OutputType>>) -> Schedule {

    {
      // process the previously overflown items
      loop {
        match self.overflow.pop_front() {
          Some(item) => {
            let mut opt_item : Option<Message<Self::InputType>> = Some(item);
            match noloss::pour(&mut opt_item, output, self) {
              (noloss::PourResult::Overflowed, _) => { break; }
              _ => {}
            }
          },
          None => { break; }
        }
      }

      // process the incoming items
      for input in input_vec {
        match input {
          &mut Some(ref mut identified) => {
            for item in identified.input.iter() {
              let mut opt_item : Option<Message<Self::InputType>> = Some(item);
              match noloss::pour(&mut opt_item, output, self) {
                (noloss::PourResult::Overflowed, _) => {}
                _ => {}
              }
            }
          },
          &mut None => {},
        }
      }

      // move the newly overflown items in
      self.overflow.append(&mut self.tmp_overflow);
    }

    Schedule::Loop
  }
}

pub fn new() -> Collector {
  Collector {
    overflow: VecDeque::new(),
    tmp_overflow: VecDeque::new(),
  }
}
