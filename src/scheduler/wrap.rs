use super::super::{Task, Observer, Schedule, TaskState, Event, EvalInfo};
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct TaskWrap {
  task:             Box<Task+Send>,
  ext_evt_count:    AtomicUsize,
  state:            TaskState,
  id:               usize,
  eval_id:          usize,
  //tx_counts:        Vec<usize>,
  input_ids:        Vec<Option<usize>>,
  dependents:       Vec<Option<(Box<TaskWrap>, usize)>>,
  n_dependents:     usize,
}

impl TaskWrap {

  /*
  fn execute(&mut self, observer: &mut Observer, time_us: &AtomicUsize) -> Schedule {
    let evt = self.task.execute();
    let info = EvalInfo::new(self.id, self.task.name(), time_us, self.eval_id);
    observer.executed(&info);

    //let ln = self.tx_counts.len();
    //let otx_slice = self.tx_counts.as_mut_slice();
    //for i in 0..ln {
    //  let new_msg_id = self.task.tx_count(i);
    //  if otx_slice[i] != new_msg_id {
    //    observer.message_sent(i, new_msg_id, &info);
    //    otx_slice[i] = new_msg_id;
    //  }
    //}

    evt
  }
  */

  fn process(&mut self, event: &Event, observer: &mut Observer, time_us: &AtomicUsize, info: &mut EvalInfo) {
    let old_state = self.state;

    // execute first, if needed
    match event {
      &Event::Execute | &Event::ExtTrigger | &Event::MessageArrived | &Event::TimerExpired =>
        {
          let evt = self.task.execute();
          let now = time_us.load(Ordering::Acquire);
          info.update_at_with_usec(now);
          observer.executed(&info);

          self.state = match evt {
            Schedule::Loop => TaskState::Execute,
            Schedule::OnMessage(ch,msg) => {
              let len = self.input_ids.len();
              if ch < len {
                let slice = self.input_ids.as_mut_slice();
                match slice[ch] {
                  None => TaskState::MessageWaitNeedId(ch,msg),
                  Some(task_id) => TaskState::MessageWait(task_id, ch,msg)
                }
              } else {
                TaskState::MessageWaitNeedId(ch,msg)
              }
            }
            Schedule::DelayUSec(us)   => TaskState::TimeWait(now+(us as usize)),
            Schedule::OnExternalEvent => TaskState::ExtEventWait(self.ext_evt_count.load(Ordering::Acquire)+1),
            Schedule::Stop            => TaskState::Stop,
          };

          observer.transition(&old_state, &Event::User(evt), &self.state, &info);
        },
      _ =>
        {
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
      TaskState::Execute => Event::Execute,
      TaskState::TimeWait(exec_at)
        if exec_at <= now => Event::TimerExpired,
      TaskState::ExtEventWait(threshold)
        if self.ext_evt_count.load(Ordering::Acquire) >= threshold => Event::ExtTrigger,
      _ => Event::Delay,
    };

    self.process(&event, observer, time_us, &mut info);

    observer.eval_finished(&info);
  }

  pub fn notify(&mut self) -> usize {
    self.ext_evt_count.fetch_add(1, Ordering::AcqRel) + 1
  }

  #[allow(dead_code)]
  pub fn attach(&mut self, idx: usize, dep: Box<TaskWrap>) {
    use std::mem;
    let n_outputs = self.dependents.len();
    if idx < n_outputs {
      let slice = self.dependents.as_mut_slice();
      let mut opt = Some((dep, self.task.tx_count(idx)));
      mem::swap(&mut opt, &mut slice[idx]);
      match opt {
        None => { self.n_dependents += 1; },
        _    => { }
      }
    }
  }

  #[allow(dead_code)]
  pub fn detach(&mut self, idx: usize) -> Option<(Box<TaskWrap>, usize)> {
    use std::mem;
    let mut ret = None;
    let n_outputs = self.dependents.len();
    if  idx < n_outputs && self.n_dependents > 0 {
      let slice = self.dependents.as_mut_slice();
      mem::swap(&mut ret, &mut slice[idx]);
      match ret {
        Some(_) => { self.n_dependents -= 1; },
        None    => { }
      }
    }
    ret
  }

  #[allow(dead_code)]
  pub fn resolve_input_task_id(&mut self, idx: usize, task_id: usize) {
    let len = self.input_ids.len();
    if idx < len {
      let slice = self.input_ids.as_mut_slice();
      slice[idx] = Some(task_id);
    }
  }
}

pub fn new(task: Box<Task+Send>, id: usize) -> TaskWrap {
  let n_outputs = task.output_count();
  let mut dependents = Vec::with_capacity(n_outputs);
  for _i in 0..n_outputs {
    dependents.push(None);
  }

  let n_inputs = task.input_count();
  let mut input_ids = Vec::with_capacity(n_inputs);
  for _i in 0..n_inputs {
    input_ids.push(None);
  }

  TaskWrap{
    task:            task,
    ext_evt_count:   AtomicUsize::new(0),
    state:           TaskState::Execute,
    id:              id,
    eval_id:         0,
    input_ids:       input_ids,
    dependents:      dependents,
    n_dependents:    0,
  }
}
