use lossyq::spsc::{noloss, Sender, Receiver};
use super::super::common::{Task, Message, IdentifiedReceiver};
use super::super::elem::filter::Filter;
use super::super::common::Schedule;
use super::executor::TaskResults;
use std::collections::VecDeque;
use std::mem;

pub struct LoopBack {
  overflow: VecDeque<Message<Box<Task + Send>>>,
  tmp_overflow: VecDeque<Message<Box<Task + Send>>>,
}

impl noloss::Overflow for LoopBack {
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

impl Filter for LoopBack {
  type InputType  = TaskResults;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
          output:  &mut Sender<Message<Self::OutputType>>) -> Schedule {

    {
      // process the previously overflown items
      loop {
        match self.overflow.pop_front() {
          Some(item) => {
            let mut opt_item : Option<Message<Self::OutputType>> = Some(item);
            match noloss::pour(&mut opt_item, output, self) {
              (noloss::PourResult::Overflowed, _) => { break; }
              _ => {}
            }
          },
          None => { break; }
        }
      }

      match input {
        &mut Some(ref mut identified) => {
          // process the incoming items

          for item in identified.input.iter() {
            match item {
              Message::Value(v) => {
                let mut opt_item : Option<Message<Self::OutputType>> = Some(Message::Value(v.task));
                match noloss::pour(&mut opt_item, output, self) {
                  (noloss::PourResult::Overflowed, _) => {}
                  _ => {}
                }
              },
              Message::Empty => {},       // ignore
              Message::Ack(_,_) => {},    // ignore
              Message::Error(_,_) => {},  // ignore
            }
          }
        },
        &mut None => {}
      }

      // move the newly overflown items in
      self.overflow.append(&mut self.tmp_overflow);
    }
    Schedule::Loop
  }
}

pub fn new() -> LoopBack {
  LoopBack {
    overflow: VecDeque::new(),
    tmp_overflow: VecDeque::new(),
  }
}
