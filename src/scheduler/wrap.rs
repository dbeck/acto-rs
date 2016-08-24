use super::super::{Task, Reporter, Schedule};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy,Clone,Debug)]
pub enum State {
  Execute,
  TimeWait(usize),
  MessageWait(usize, usize), // ch_id, msg_id
  ExtEventWait(usize),
  Stop,
}

pub struct TaskWrap {
  task:             Box<Task+Send>,
  ext_evt_count:    AtomicUsize,
  state:            State,
  id:               usize,
  tx_counts:        Vec<usize>,
}

impl TaskWrap {
  pub fn execute(&mut self,
                 reporter: &mut Reporter,
                 time_us: &AtomicUsize) -> State {
    // check if we can move to Execute state
    self.state = match self.state {
      State::TimeWait(tm) => {
        let start = time_us.load(Ordering::Acquire);
        if tm >= start { State::Execute }
        else           { State::TimeWait(tm) }
      },
      State::ExtEventWait(threshold) => {
        if self.ext_evt_count.load(Ordering::Acquire) > threshold {
          State::Execute
        } else {
          State::ExtEventWait(threshold)
        }
      }
      /*no change on Stop, Execute and MessageWait*/
      State::Execute              => { State::Execute },
      State::Stop                 => { State::Stop },
      State::MessageWait(ch,msg)  => { State::MessageWait(ch,msg) },
    };

    match self.state {
      State::Execute => {
        let new_state = match self.task.execute() {
          Schedule::Loop              => { State::Execute }
          Schedule::Stop              => { State::Stop }
          Schedule::OnMessage(ch,msg) => {
            match &self.task.input_id(ch) {
              &Some(ref channel_id) => {
                reporter.wait_channel(channel_id, msg, self.id);
                State::MessageWait(ch,msg)
              },
              _ => {
                // if channel id is not available, then delay by 10 usec
                State::TimeWait(time_us.load(Ordering::Acquire)+10)
              }
            }
          }
          Schedule::OnExternalEvent   => { State::ExtEventWait(self.ext_evt_count.load(Ordering::Acquire)+1) }
          Schedule::DelayUSec(us)     => { State::TimeWait(time_us.load(Ordering::Acquire)+(us as usize)) }
        };
        self.state = new_state;
        let ln = self.tx_counts.len();
        let otx_slice = self.tx_counts.as_mut_slice();
        for i in 0..ln {
          let new_msg_id = self.task.tx_count(i);
          if otx_slice[i] != new_msg_id {
            reporter.message_sent(i, new_msg_id, self.id);
            otx_slice[i] = new_msg_id;
          }
        }
      },
      _ => { /*delay otherwise*/ }
    };
    self.state
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::Release)
  }
}

pub fn new(task: Box<Task+Send>, id: usize) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task: task,
    ext_evt_count: AtomicUsize::new(0),
    state: State::Execute,
    id: id,
    tx_counts: vec![0; n_outputs],
  }
}
