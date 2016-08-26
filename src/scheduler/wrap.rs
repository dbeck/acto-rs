use super::super::{Task, Observer, Schedule, TaskState};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TaskWrap {
  task:             Box<Task+Send>,
  ext_evt_count:    AtomicUsize,
  state:            TaskState,
  id:               usize,
  tx_counts:        Vec<usize>,
}

impl TaskWrap {
  pub fn execute(&mut self,
                 observer: &mut Observer,
                 time_us: &AtomicUsize) -> TaskState {

    let mut now = time_us.load(Ordering::Acquire);
    let mut delayed = false;
    let mut stopped = false;
    // check if we can move to Execute state
    self.state = match self.state {
      TaskState::TimeWait(exec_at) => {
        if exec_at <= now {
          //fn transition(&mut self, from: &TaskState, event: &Schedule, to: &TaskState, task_id: usize, at_usec: usize);
          observer.transition(&self.state, &Schedule::DelayUSec(0), &TaskState::Execute, self.id, now);
          TaskState::Execute
        } else {
          delayed = true;
          TaskState::TimeWait(exec_at)
        }
      },
      TaskState::ExtEventWait(threshold) => {
        if self.ext_evt_count.load(Ordering::Acquire) > threshold {
          observer.transition(&self.state, &Schedule::OnExternalEvent, &TaskState::Execute, self.id, now);
          TaskState::Execute
        } else {
          delayed = true;
          TaskState::ExtEventWait(threshold)
        }
      }
      TaskState::Execute  => {
        TaskState::Execute
      },
      TaskState::Stop => {
        stopped = true;
        TaskState::Stop
      },
      TaskState::MessageWait(ch,msg) => {
        delayed = true;
        TaskState::MessageWait(ch,msg)
      },
    };

    if delayed {
      observer.delayed(self.id, &self.state, now);
      return self.state
    } else if stopped {
      observer.stopped(self.id, now);
      return self.state;
    }

    // execute the task and match event
    let evt = self.task.execute();
    now = time_us.load(Ordering::Acquire);
    let new_state = match &evt {
      &Schedule::Loop => { TaskState::Execute },
      &Schedule::Stop => { TaskState::Stop },
      &Schedule::OnMessage(ch,msg) => {
        match &self.task.input_id(ch) {
          &Some(ref channel_id) => {
            observer.wait_channel(channel_id, msg, self.id, now);
            TaskState::MessageWait(ch,msg)
          },
          _ => {
            // if channel id is not available, then delay by 10 usec
            TaskState::TimeWait(now+10)
          }
        }
      }
      &Schedule::OnExternalEvent => { TaskState::ExtEventWait(self.ext_evt_count.load(Ordering::Acquire)+1) }
      &Schedule::DelayUSec(us)   => { TaskState::TimeWait(now+(us as usize)) }
    };

    // report task exec
    observer.executed(self.id, now);
    observer.transition(&self.state, &evt, &new_state, self.id, now);
    self.state = new_state;

    // check and report output channel changes if any
    let ln = self.tx_counts.len();
    let otx_slice = self.tx_counts.as_mut_slice();
    for i in 0..ln {
      let new_msg_id = self.task.tx_count(i);
      if otx_slice[i] != new_msg_id {
        observer.message_sent(i, new_msg_id, self.id, now);
        otx_slice[i] = new_msg_id;
      }
    }
    self.state
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::Release)
  }
}

// TODO: attach , detach

pub fn new(task: Box<Task+Send>, id: usize) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task:            task,
    ext_evt_count:   AtomicUsize::new(0),
    state:           TaskState::Execute,
    id:              id,
    tx_counts:       vec![0; n_outputs],
  }
}
