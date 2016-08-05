extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::gather::Gather;
use super::super::common::Schedule;
use std::collections::VecDeque;
use std::mem;

pub struct Collector {
  overflow: VecDeque<Box<Task + Send>>,
}

impl Gather for Collector {
  type InputType  = Box<Task + Send>;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          input_vec:   Vec<&mut Receiver<Message<Self::InputType>>>,
          output:      &mut Sender<Message<Self::OutputType>>) -> Schedule {

    {
      // this closure sends a value to the output channel, but also checks if
      // we were replacing an item in the output queue. if so it saves that in
      // the overflow queue and returns false. otherwise true.
      let mut send_value = |self_val: &mut Self, mut val: Option<Message<Self::InputType>>| {
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
            let optional_item : Option<Message<Self::InputType>> = Some(Message::Value(item));
            if send_value(self, optional_item) == false {
              break;
            }
          },
          None => { break; }
        }
      }

      // process the incoming items
      for input in input_vec {
        for item in input.iter() {
          let optional_item : Option<Message<Self::InputType>> = Some(item);
          send_value(self, optional_item);
        }
      }
    }

    // check tmp value of output channel
    {
      let mut val : Option<Message<Self::InputType>> = None;
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

pub fn new() -> Collector {
  Collector {
    overflow: VecDeque::new()
  }
}
