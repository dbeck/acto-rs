use lossyq::spsc::{noloss, Sender, Receiver};
use super::super::common::{Task, Message, Schedule};
use super::super::elem::scatter::Scatter;
use std::collections::VecDeque;
use super::CountingReporter;
use std::mem;

pub struct TaskResults {
  pub task      : Box<Task + Send>,
  pub schedule  : Schedule,
}

pub struct Executor {
  overflow: VecDeque<Message<TaskResults>>,
}

impl noloss::Overflow for Executor {
  type Input = Message<TaskResults>;

  fn overflow(&mut self, val : &mut Option<Self::Input>) {
    let mut tmp : Option<Self::Input> = None;
    mem::swap(&mut tmp, val);
    match tmp {
      Some(v) => {
        self.overflow.push_back(v);
      },
      None => {}
    }
  }
}

impl Scatter for Executor {
  type InputType  = Box<Task + Send>;
  type OutputType = TaskResults;

  fn process(
          &mut self,
          input:   &mut Receiver<Message<Self::InputType>>,
          output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule {

    for i in input.iter() {

      match i {
        // TODO : check these cases
        Message::Empty => {}, // ignore
        Message::Value(mut v) => {
          let mut reporter = CountingReporter{ count: 0 };
          let result = v.execute(&mut reporter);
          // 0: stopped
          // 1: loop_back
          // 2: on_msg
          // 3: timer
          let _out_channels = output.as_mut_slice();
          match result {
            Schedule::Loop => {},
            Schedule::OnMessage(_id)  => {},
            Schedule::EndPlusUSec(_usec) => {},
            Schedule::StartPlusUSec(_usec) => {},
            Schedule::Stop => {},
          };
        },
        Message::Ack(_,_) => {}, // ignore
        Message::Error(_,_) => {},  // ignore
      };
    }
    Schedule::Loop
  }
}

pub fn new() -> Executor {
  Executor {
    overflow: VecDeque::new(),
  }
}
