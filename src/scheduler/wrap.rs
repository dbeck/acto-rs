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

    let mut delayed = false;
    let mut stopped = false;
    // check if we can move to Execute state
    self.state = match self.state {
      TaskState::TimeWait(exec_at) => {
        let now = time_us.load(Ordering::Acquire);
        if exec_at <= now {
          TaskState::Execute
        } else {
          delayed = true;
          TaskState::TimeWait(exec_at)
        }
      },
      TaskState::ExtEventWait(threshold) => {
        if self.ext_evt_count.load(Ordering::Acquire) > threshold {
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
      observer.delayed(self.id, &self.state);
      return self.state
    } else if stopped {
      observer.stopped(self.id);
      return self.state;
    }

    // execute the task
    self.state = match self.task.execute() {
      Schedule::Loop              => { TaskState::Execute }
      Schedule::Stop              => { TaskState::Stop }
      Schedule::OnMessage(ch,msg) => {
        match &self.task.input_id(ch) {
          &Some(ref channel_id) => {
            observer.wait_channel(channel_id, msg, self.id);
            TaskState::MessageWait(ch,msg)
          },
          _ => {
            // if channel id is not available, then delay by 10 usec
            TaskState::TimeWait(time_us.load(Ordering::Acquire)+10)
          }
        }
      }
      Schedule::OnExternalEvent => { TaskState::ExtEventWait(self.ext_evt_count.load(Ordering::Acquire)+1) }
      Schedule::DelayUSec(us)   => { TaskState::TimeWait(time_us.load(Ordering::Acquire)+(us as usize)) }
    };

    // report task exec
    observer.executed(self.id);

    // check and report output channel changes if any
    let ln = self.tx_counts.len();
    let otx_slice = self.tx_counts.as_mut_slice();
    for i in 0..ln {
      let new_msg_id = self.task.tx_count(i);
      if otx_slice[i] != new_msg_id {
        observer.message_sent(i, new_msg_id, self.id);
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
