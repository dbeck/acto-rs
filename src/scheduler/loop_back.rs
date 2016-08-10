extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::filter::Filter;
use super::super::common::Schedule;
use super::executor::TaskResults;
use std::collections::VecDeque;
use std::mem;

pub struct LoopBack {
  overflow: VecDeque<Box<Task + Send>>,
}

impl Filter for LoopBack {
  type InputType  = TaskResults;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          input:   &mut Receiver<Message<Self::InputType>>,
          output:  &mut Sender<Message<Self::OutputType>>) -> Schedule {

    {
      // this closure sends a value to the output channel, but also checks if
      // we were replacing an item in the output queue. if so it saves that in
      // the overflow queue and returns false. otherwise true.
      let mut send_value = |self_val: &mut Self, mut val: Option<Message<Self::OutputType>>| {
        // put value in the output queue
        output.put( |o| { mem::swap(&mut val, o); });

        // check the result of put()
        match val {
          Some(Message::Value(task)) => {
            // saving the replaced item
            self_val.overflow.push_back(task);
            false
          },
          _ => { true }
        }
      };

      // process the previously overflown items
      loop {
        match self.overflow.pop_front() {
          Some(item) => {
            let optional_item : Option<Message<Self::OutputType>> = Some(Message::Value(item));
            if send_value(self, optional_item) == false {
              break;
            }
          },
          None => { break; }
        }
      }

      // process the incoming items
      for item in input.iter() {
        match item {
          Message::Value(v) => {
            let optional_item : Option<Message<Self::OutputType>> = Some(Message::Value(v.task));
            send_value(self, optional_item);
          },
          Message::Empty => {},       // ignore
          Message::Ack(_,_) => {},    // ignore
          Message::Error(_,_) => {},  // ignore
        }
      }
    }

    // check tmp value of output channel and saves it
    // to the overflow queue
    {
      let mut val : Option<Message<Self::OutputType>> = None;
      output.tmp( |t| { mem::swap(&mut val, t); });
      match val {
        Some(Message::Value(task)) => {
          // saving the replaced item
          self.overflow.push_back(task);
        },
        _ => {}
      }
    }

    Schedule::Loop
  }
}

pub fn new() -> LoopBack {
  LoopBack {
    overflow: VecDeque::new(),
  }
}
