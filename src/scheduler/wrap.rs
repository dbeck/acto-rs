use super::super::{Task, Observer, Schedule, TaskState, Event, EvalInfo};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TaskWrap {
  task:             Box<Task+Send>,
  ext_evt_count:    AtomicUsize,
  state:            TaskState,
  id:               usize,
  eval_id:          usize,
  tx_counts:        Vec<usize>,
}

impl TaskWrap {

  fn execute(&mut self, observer: &mut Observer, time_us: &AtomicUsize) -> Schedule {
    let evt = self.task.execute();
    let info = EvalInfo::new(self.id, self.task.name(), time_us, self.eval_id);
    observer.executed(&info);

    let ln = self.tx_counts.len();
    let otx_slice = self.tx_counts.as_mut_slice();
    for i in 0..ln {
      let new_msg_id = self.task.tx_count(i);
      if otx_slice[i] != new_msg_id {
        observer.message_sent(i, new_msg_id, &info);
        otx_slice[i] = new_msg_id;
      }
    }
    evt
  }

  fn transition(&mut self, event: &Event, observer: &mut Observer, time_us: &AtomicUsize) {
    let old_state = self.state;
    let mut input_event : Event = *event;

    // execute first, if needed
    match input_event {
      Event::Execute | Event::ExtTrigger | Event::MessageArrived | Event::TimerExpired =>
        { input_event = Event::User(self.execute(observer, time_us)); },
      _ =>
        { },
    };

    // evaluate user transitions too
    let now = time_us.load(Ordering::Acquire);
    let new_state = match input_event {
      Event::User(Schedule::Loop)              => TaskState::Execute,
      Event::User(Schedule::Stop)              => TaskState::Stop,
      Event::User(Schedule::OnMessage(ch,msg)) => TaskState::MessageWait(ch,msg),
      Event::User(Schedule::OnExternalEvent)   => TaskState::ExtEventWait(self.ext_evt_count.load(Ordering::Acquire)+1),
      Event::User(Schedule::DelayUSec(us))     => TaskState::TimeWait(now+(us as usize)),
      _ => old_state,
    };

    {
      let info = EvalInfo::new_with_usec(self.id, self.task.name(), now, self.eval_id);
      observer.transition(&old_state, &input_event, &new_state, &info);
    }
    self.state = new_state;
  }

  pub fn eval(&mut self,
              observer: &mut Observer,
              time_us: &AtomicUsize) -> TaskState {

    self.eval_id += 1;
    {
      let info = EvalInfo::new(self.id, self.task.name(), time_us, self.eval_id);
      observer.eval_started(&info);
    }

    // check if there is a condition to exit from a wait-state
    let event = match self.state {
      TaskState::Execute => Event::Execute,
      TaskState::TimeWait(exec_at)
        if exec_at <= time_us.load(Ordering::Acquire) => Event::TimerExpired,
      TaskState::ExtEventWait(threshold)
        if self.ext_evt_count.load(Ordering::Acquire) >= threshold => Event::ExtTrigger,
      _ => Event::Delay,
    };

    self.transition(&event, observer, time_us);

    {
      let info = EvalInfo::new(self.id, self.task.name(), time_us, self.eval_id);
      observer.eval_finished(&info);
    }

    self.state
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::Release) + 1
  }
}

//TODO: attach, detach

pub fn new(task: Box<Task+Send>, id: usize) -> TaskWrap {
  let n_outputs = task.output_count();

  TaskWrap{
    task:            task,
    ext_evt_count:   AtomicUsize::new(0),
    state:           TaskState::Execute,
    id:              id,
    eval_id:         0,
    tx_counts:       vec![0; n_outputs],
  }
}
