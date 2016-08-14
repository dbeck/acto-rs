use lossyq::spsc::{noloss, Sender, Receiver};
use time;
use super::super::common::{Task, Message, Schedule, IdentifiedReceiver};
use super::super::elem::scatter::Scatter;
use std::collections::VecDeque;
use super::CountingReporter;
use super::MeasureTime;
use std::mem;

pub struct TaskResults {
  pub task      : Box<Task + Send>,
  pub schedule  : Schedule,
  pub start_ns  : u64,
  pub end_ns    : u64,
}

pub struct Executor {
  overflow: VecDeque<Message<TaskResults>>,
  p1: MeasureTime,
  p2: MeasureTime,
  p3: MeasureTime,
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
          input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
          output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule {

    let mut tmp_overflow = Executor {
      overflow: VecDeque::new(),
      p1: MeasureTime::new(),
      p2: MeasureTime::new(),
      p3: MeasureTime::new(),
    };
    let mut output = output;

    {
      let mut forward = |msg: Self::OutputType,
                         outvec: &mut Vec<Sender<Message<Self::OutputType>>>|
      {
        let mut send_channel = |msg: Self::OutputType,
                                out: &mut Sender<Message<Self::OutputType>>| {
          // pour message in
          let mut opt_item : Option<Message<Self::OutputType>> = Some(Message::Value(msg));
          noloss::pour(&mut opt_item, out, &mut tmp_overflow)
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

      self.p1.start();
      // process the previously overflowed items
      loop {
        match self.overflow.pop_front() {
          // Option<Message<TaskResults>>
          Some(Message::Value(v)) => {
            println!("??? 1");
            match forward(v, &mut output) {
              (noloss::PourResult::Overflowed, _) => { break; },
              _ => {}
            }
          },
          None => { break; }
          _ => {
            println!("??? 2");
          }, // ignore everything else
        }
      }
      self.p1.end();
      self.p1.print("Executor: p1");

      self.p2.start();
      // process inputs
      match input {
        &mut Some(ref mut identified) => {
          for i in identified.input.iter() {
            match i {
              Message::Value(mut v) => {
                let start_ns = time::precise_time_ns();
                let mut reporter = CountingReporter{ count: 0 };
                let schedule = v.execute(&mut reporter);
                let end_ns = time::precise_time_ns();
                let task_res = TaskResults {
                  task: v,
                  schedule: schedule,
                  start_ns: start_ns,
                  end_ns: end_ns
                };
                forward(task_res, &mut output);
              },
              _ => {}, // ignore everything else
            };
          }
        },
        &mut None => {}
      }
    }
    self.p2.end();
    self.p2.print("Executor: p2");

    self.p3.start();
    {
      // move the newly overflown items in
      self.overflow.append(&mut tmp_overflow.overflow);
    }
    self.p3.end();
    self.p3.print("Executor: p3");

    Schedule::Loop
  }
}

pub fn new() -> Executor {
  Executor {
    overflow: VecDeque::new(),
    p1: MeasureTime::new(),
    p2: MeasureTime::new(),
    p3: MeasureTime::new(),
  }
}
