use super::super::{Task, Schedule, TaskState, Event, AbsSchedulerTimeInUsec,
  ExtEventSeqno, ChannelId, SenderId, TaskId
};
use super::observer::{Observer, EvalInfo};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TaskWrap {
  task:             Box<Task+Send>,
  /* XXX
  ext_evt_count:    usize,
  state:            TaskState,
  */
  id:               TaskId,
  /* XXX
  eval_id:          usize,
  input_ids:        Vec<Option<usize>>,
  dependents:       Vec<Option<TaskId>>,
  n_dependents:     usize,
  */
  exec_count:       u64,
  /* XXX
  delay_count:      u64,
  */
}

impl TaskWrap {

  /* XXX
  fn process(&mut self, event: &Event, observer: &mut Observer, time_us: &AtomicUsize, info: &mut EvalInfo) {
    let old_state     = self.state;
    let mut executed  = false;

    // execute first, if needed
    match event {
      &Event::Execute | &Event::ExtTrigger | &Event::MessageArrived | &Event::TimerExpired =>
        {
          if let &Event::Execute = event {
            //
          } else {
            // record original transition
            observer.transition(&old_state, event, &self.state, &info);
          }

          // execute the task, save the schedule request
          let user_evt = self.task.execute();
          executed = true;

          // record statistics about the execution
          let now = time_us.load(Ordering::Acquire);
          info.update_at_with_usec(now);
          observer.executed(&info);

          // check what the schedule tells us
          self.state = match user_evt {
            Schedule::Loop => TaskState::Execute,
            Schedule::OnMessage(channel_id) => {
              let receiver_channel_id = channel_id.receiver_id.0;
              // who is the sender task?
              // check the input_ids vector for cached task ids
              let len = self.input_ids.len();
              if receiver_channel_id < len {
                let slice = self.input_ids.as_mut_slice();
                match slice[receiver_channel_id] {
                  None => {
                    TaskState::Execute
                  },
                  Some(sender_task_id) => {
                    TaskState::MessageWait(SenderId(sender_task_id), channel_id)
                  }
                }
              } else {
                TaskState::Execute
              }
            }
            Schedule::DelayUsec(us)
              => TaskState::TimeWait(AbsSchedulerTimeInUsec (now+(us.0 as usize)) ),
            Schedule::OnExternalEvent
              => TaskState::ExtEventWait(ExtEventSeqno (self.ext_evt_count+1)),
            Schedule::Stop
              => TaskState::Stop,
          };

          // record state transition
          observer.transition(&old_state, &Event::User(user_evt), &self.state, &info);

          // check if there are any dependents to trigger
          {
            if self.n_dependents > 0 {
              let mut pos         = 0;
              let dep_len         = self.dependents.len();
              let slice           = self.dependents.as_slice();

              loop {
                if pos >= dep_len { break; }
                if let &Some(ref dep) = &slice[pos] {
                  observer.msg_trigger(*dep, &info);
                }
                pos += 1;
              }
            }
          }
        },
      _ =>
        {
          observer.transition(&old_state, event, &self.state, &info);
        },
    };

    if executed {
      self.exec_count += 1;
    } else {
      self.delay_count += 1;
    }
  }
  XXX
  */

  /* XXX
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
      _ => Event::Delay,
    };

    self.process(&event, observer, time_us, &mut info);

    observer.eval_finished(&info);
  }
  */

  pub fn eval(&mut self,
              // observer: &mut Observer,
              _time_us: &AtomicUsize) {
    let _res = self.task.execute();
    self.exec_count += 1;
  }

  /* XXX
  pub fn ext_notify(&mut self,
                    incr: usize,
                    observer: &mut Observer,
                    time_us: &AtomicUsize) {

    let old_state = self.state;
    self.ext_evt_count += incr;
    if let TaskState::ExtEventWait(threshold) = old_state {
      let event = if self.ext_evt_count >= threshold.0 {
        self.state = TaskState::Execute;
        Event::ExtTrigger
      } else {
        Event::Delay
      };
      self.eval_id += 1;
      let info = EvalInfo::new(self.id, time_us, self.eval_id);
      observer.transition(&old_state, &event, &self.state, &info);
    }
  }
  */

  /* XXX
  pub fn msg_trigger(&mut self,
                     observer: &mut Observer,
                     time_us: &AtomicUsize)
  {
    let old_state = self.state;
    if let TaskState::MessageWait(_sender_id, _ch_id) = old_state {
      self.state = TaskState::Execute;
      self.eval_id += 1;
      let info = EvalInfo::new(self.id, time_us, self.eval_id);
      observer.transition(&old_state, &Event::MessageArrived, &self.state, &info);
    }
  }
  */

  /* XXX
  pub fn register_dependent(&mut self, ch: ChannelId, dep_task_id: TaskId)
  {
    use std::mem;
    let n_outputs = self.dependents.len();
    if ch.sender_id.0 < n_outputs {
      let slice = self.dependents.as_mut_slice();
      let mut opt = Some(dep_task_id);
      mem::swap(&mut opt, &mut slice[ch.sender_id.0]);
      //println!("register_dependent of({:?}:{}) dep:{:?} ch:{:?} pos:{:?}",
        //self.id, self.task.name(), dep_task_id, ch, channel_pos);
      match opt {
        None => { self.n_dependents += 1; },
        _    => { }
      }
    }
  }
  */

  /* XXX
  pub fn resolve_input_task_id(&mut self, ch: ChannelId, task_id: TaskId) {
    println!("resolve_input_task_id: {:?} -> {:?}/{:?}",
      self.id,
      ch,
      task_id
    );
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
  */

  #[cfg(feature = "printstats")]
  fn print_stats(&self) {
    println!(" @drop TaskWrap task_id:{:?} exec_count:{} delay_count:{} eval_id:{} ext_evt:{}",
     self.id,
     self.exec_count,
     self.delay_count,
     self.eval_id,
     self.ext_evt_count);
  }

  #[cfg(not(feature = "printstats"))]
  fn print_stats(&self) {}
}

pub fn new(task: Box<Task+Send>, id: TaskId, input_task_ids: Vec<Option<usize>>) -> TaskWrap {
  /*
  let n_outputs = task.output_count();
  let mut dependents = Vec::with_capacity(n_outputs);
   for _i in 0..n_outputs {
     dependents.push(None);
  }
  */

  TaskWrap{
    task:            task,
    // ext_evt_count:   0,
    // state:           TaskState::Execute,
    id:              id,
    // eval_id:         0,
    // input_ids:       input_task_ids,
    // dependents:      dependents,
    // n_dependents:    0,
    exec_count:      0,
    // delay_count:     0,
  }
}

impl Drop for TaskWrap {
  fn drop(&mut self) {
    self.print_stats();
  }
}
