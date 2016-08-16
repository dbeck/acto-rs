use lossyq::spsc::{noloss, Sender};
use time;
use super::super::common::{Task, Message, Schedule, IdentifiedReceiver};
use super::super::elem::scatter::Scatter;
use std::collections::VecDeque;
use super::CountingReporter;
use std::mem;

pub struct TaskResults {
  pub task      : Box<Task + Send>,
  pub schedule  : Schedule,
  pub start_ns  : u64,
  pub end_ns    : u64,
}

pub struct Executor {
  overflow: VecDeque<Message<TaskResults>>,
  tmp_overflow: VecDeque<Message<TaskResults>>,
}

impl noloss::Overflow for Executor {
  type Input = Message<TaskResults>;

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

impl Scatter for Executor {
  type InputType  = Box<Task + Send>;
  type OutputType = TaskResults;

  fn process(
          &mut self,
          input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
          output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule {

    let mut output = output;
    {
      let forward = |me: &mut Self,
                         msg: Self::OutputType,
                         outvec: &mut Vec<Sender<Message<Self::OutputType>>>|
      {
        let mut send_channel = |msg: Self::OutputType,
                                out: &mut Sender<Message<Self::OutputType>>| {
          // pour message in
          let mut opt_item : Option<Message<Self::OutputType>> = Some(Message::Value(msg));
          noloss::pour(&mut opt_item, out, me)
        };

        // 0: stopped
        // 1: loop_back
        // 2: on_msg
        // 3: timer
        let outp_slice = outvec.as_mut_slice();
        match msg.schedule {
          Schedule::Loop => {
            send_channel(msg, &mut outp_slice[1])
          },
          Schedule::OnMessage(_id) => {
            send_channel(msg, &mut outp_slice[2])
          },
          Schedule::EndPlusUSec(usec) => {
            let now = time::precise_time_ns();
            if msg.end_ns + usec < now {
              // send to loop
              send_channel(msg, &mut outp_slice[1])
            } else {
              // send to timer
              send_channel(msg, &mut outp_slice[3])
            }
          },
          Schedule::StartPlusUSec(usec) => {
            let now = time::precise_time_ns();
            if msg.start_ns + usec < now {
              // send to loop
              send_channel(msg, &mut outp_slice[1])
            } else {
              // send to timer
              send_channel(msg, &mut outp_slice[3])
            }
          },
          Schedule::Stop => {
            send_channel(msg, &mut outp_slice[0])
          },
        }
      };

      // process the previously overflowed items
      loop {
        match self.overflow.pop_front() {
          // Option<Message<TaskResults>>
          Some(Message::Value(v)) => {
            println!("??? 1");
            match forward(self, v, &mut output) {
              (noloss::PourResult::Overflowed, _) => { break; },
              _ => {}
            }
          },
          None => { break; }
          _ => {
          }, // ignore everything else
        }
      }

      // process inputs
      match input {
        &mut Some(ref mut identified) => {
          for i in identified.input.iter() {
            match i {
              Message::Value(mut v) => {
                //let start_ns = time::precise_time_ns();
                let mut reporter = CountingReporter{ count: 0 };
                let schedule = v.execute(&mut reporter);
                //let end_ns = time::precise_time_ns();
                let task_res = TaskResults {
                  task: v,
                  schedule: schedule,
                  start_ns: 0,
                  end_ns: 0
                };
                forward(self, task_res, &mut output);
              },
              _ => {}, // ignore everything else
            };
          }
        },
        &mut None => {}
      }
    }

    {
      // move the newly overflown items in
      self.overflow.append(&mut self.tmp_overflow);
    }

    Schedule::Loop
  }
}

pub fn new() -> Executor {
  Executor {
    overflow: VecDeque::new(),
    tmp_overflow: VecDeque::new(),
  }
}
