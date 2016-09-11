use super::super::{Task, Schedule, TaskState, Event, AbsSchedulerTimeInUsec,
  ExtEventSeqno, ChannelId, SenderName, SenderId,
  TaskId, ChannelPosition, ReceiverChannelId, SenderChannelId
};
use super::observer::{Observer, EvalInfo};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::mem;

#[allow(dead_code)]
struct Dependent {
  task_id:           TaskId,
  channel_position:  ChannelPosition,
}

pub struct TaskWrap {
  task:             Box<Task+Send>,
  ext_evt_count:    AtomicUsize,
  state:            TaskState,
  id:               TaskId,
  eval_id:          usize,
  input_ids:        Vec<Option<usize>>,
  dependents:       Vec<Option<Dependent>>,
  n_dependents:     usize,
}

impl TaskWrap {

  pub fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.task.input_id(ch_id)
  }

  fn process(&mut self, event: &Event, observer: &mut Observer, time_us: &AtomicUsize, info: &mut EvalInfo) {
    let old_state = self.state;

    // execute first, if needed
    match event {
      &Event::Execute | &Event::ExtTrigger | &Event::MessageArrived | &Event::TimerExpired =>
        {
          // execute the task, save the schedule request
          let evt = self.task.execute();

          // record statistics about the execution
          let now = time_us.load(Ordering::Acquire);
          info.update_at_with_usec(now);
          observer.executed(&info);

          // check what the schedule tells us
          self.state = match evt {
            Schedule::Loop => TaskState::Execute,
            Schedule::OnMessage(channel_id, channel_position) => {
              let receiver_channel_id = channel_id.receiver_id.0;
              // who is the sender task?
              // check the input_ids vector for cached task ids
              let len = self.input_ids.len();
              if receiver_channel_id < len {
                let slice = self.input_ids.as_mut_slice();
                match slice[receiver_channel_id] {
                  None => {
                    TaskState::MessageWaitNeedSenderId(channel_id, channel_position)
                  },
                  Some(sender_task_id) => {
                    TaskState::MessageWait(SenderId(sender_task_id), channel_id, channel_position)
                  }
                }
              } else {
                TaskState::MessageWaitNeedSenderId(channel_id, channel_position)
              }
            }
            Schedule::DelayUsec(us)
              => TaskState::TimeWait(AbsSchedulerTimeInUsec (now+(us.0 as usize)) ),
            Schedule::OnExternalEvent
              => TaskState::ExtEventWait(ExtEventSeqno (self.ext_evt_count.load(Ordering::Acquire)+1)),
            Schedule::Stop
              => TaskState::Stop,
          };

          // record state transition
          observer.transition(&old_state, &Event::User(evt), &self.state, &info);

          // check if there are any dependents to trigger
          {
            let mut pos  = 0;
            let dep_len  = self.dependents.len();
            let slice    = self.dependents.as_mut_slice();

            loop {
              if self.n_dependents == 0 { break; }
              if pos >= dep_len { break; }
              let mut act_dep : Option<Dependent> = None;
              mem::swap(&mut act_dep, &mut slice[pos]);
              if let Some(dep_swapped) = act_dep {
                let channel_pos = self.task.output_channel_pos(SenderChannelId(pos));
                if channel_pos.0 >= dep_swapped.channel_position.0 {
                  self.n_dependents -= 1;
                  // tell the scheduler, that the dependent task may become
                  // eligible to run
                  observer.msg_trigger(
                    dep_swapped.task_id,
                    channel_pos,
                    &info);
                } else {
                  // the channel positions is not yet what is needed
                  let mut place_dep_back = Some(dep_swapped);
                  mem::swap(&mut place_dep_back, &mut slice[pos]);
                }
              }
              pos += 1;
            }
          }
        },
      _ =>
        {
          match self.state {
            // check if the dependecy ID has been resolved since
            TaskState::MessageWaitNeedSenderId(channel_id, channel_position) => {
              let receiver_channel_id = channel_id.receiver_id.0;
              let len = self.input_ids.len();
              if receiver_channel_id < len {
                let slice = self.input_ids.as_mut_slice();
                match slice[receiver_channel_id] {
                  Some(sender_task_id) => {
                    self.state = TaskState::MessageWait(
                      SenderId (sender_task_id), channel_id, channel_position);
                  }
                  None => {},
                }
              }
            },
            _ => {},
          }
          observer.transition(&old_state, event, &self.state, &info);
        },
    };
  }

  pub fn eval(&mut self,
              observer: &mut Observer,
              time_us: &AtomicUsize) {

    self.eval_id += 1;
    let now = time_us.load(Ordering::Acquire);
    let mut info = EvalInfo::new_with_usec(self.id, now, self.eval_id);
    observer.eval_started(&info);

    // check if there is a condition to exit from a wait-state
    let event = match self.state {
      TaskState::Execute
        => Event::Execute,
      TaskState::TimeWait(exec_at) if exec_at.0 <= now
        => Event::TimerExpired,
      TaskState::ExtEventWait(threshold)
        if self.ext_evt_count.load(Ordering::Acquire) >= threshold.0 => Event::ExtTrigger,
      _ => Event::Delay,
    };

    self.process(&event, observer, time_us, &mut info);

    observer.eval_finished(&info);
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::AcqRel) + 1
  }

  pub fn trigger_message(&mut self,
                         observer: &mut Observer,
                         time_us: &AtomicUsize)
  {
    let old_state = self.state;
    if let TaskState::MessageWait(_sender_id, _channel_id, _channel_position) = old_state {
      self.eval_id += 1;
      let info = EvalInfo::new(self.id, time_us, self.eval_id);
      self.state = TaskState::Execute;
      observer.transition(&old_state, &Event::MessageArrived, &self.state, &info);
    }
  }

  pub fn register_dependent(&mut self, ch: ChannelId, dep_task_id: TaskId, channel_pos: ChannelPosition) {
    use std::mem;
    let n_outputs = self.dependents.len();
    if ch.sender_id.0 < n_outputs {
      let slice = self.dependents.as_mut_slice();
      let mut opt = Some(Dependent{task_id:dep_task_id, channel_position:channel_pos});
      mem::swap(&mut opt, &mut slice[ch.sender_id.0]);
      println!("register_dependent of({:?}:{}) dep:{:?} ch:{:?} pos:{:?}",
        self.id, self.task.name(), dep_task_id, ch, channel_pos);
      match opt {
        None => { self.n_dependents += 1; },
        _    => { }
      }
    }
  }

  pub fn resolve_input_task_id(&mut self, ch: ChannelId, task_id: TaskId) {
    let len = self.input_ids.len();
    if ch.receiver_id.0 < len {
      let slice = self.input_ids.as_mut_slice();
      slice[ch.receiver_id.0] = Some(task_id.0);
    } else {
      for i in len..(ch.receiver_id.0+1) {
        if i == ch.receiver_id.0 {
          self.input_ids.push(Some(task_id.0));
        } else {
          self.input_ids.push(None);
        }
      }
    }
  }
}

pub fn new(task: Box<Task+Send>, id: TaskId, input_task_ids: Vec<Option<usize>>) -> TaskWrap {
  let n_outputs = task.output_count();
  let mut dependents = Vec::with_capacity(n_outputs);
  for _i in 0..n_outputs {
    dependents.push(None);
  }

  TaskWrap{
    task:            task,
    ext_evt_count:   AtomicUsize::new(0),
    state:           TaskState::Execute,
    id:              id,
    eval_id:         0,
    input_ids:       input_task_ids,
    dependents:      dependents,
    n_dependents:    0,
  }
}
