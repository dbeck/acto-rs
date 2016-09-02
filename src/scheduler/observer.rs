use super::super::{Observer, TaskState, EvalInfo, Event};

#[derive(Copy,Clone,Debug)]
pub struct CountingReporter {
  pub scheduled:   usize,
  pub executed:    usize,
  pub stopped:     usize,
  pub delayed:     usize,
  pub time_wait:   usize,
  pub msg_wait:    usize,
  pub ext_wait:    usize,
  // pub sent:        usize,
  pub channel:     usize,
  pub transition:  usize,
}

#[allow(dead_code)]
impl CountingReporter {
  pub fn new() -> CountingReporter {
    CountingReporter{
      scheduled:   0,
      executed:    0,
      stopped:     0,
      delayed:     0,
      time_wait:   0,
      msg_wait:    0,
      ext_wait:    0,
      // sent:        0,
      channel:     0,
      transition:  0,
    }
  }

  #[allow(dead_code)]
  pub fn print(&self) {
    println!("{:?}", self);
  }
}

impl Observer for CountingReporter {
  fn eval_started(&mut self, _info: &EvalInfo) {
    self.scheduled += 1;
  }

  fn executed(&mut self, _info: &EvalInfo) {
    self.executed += 1;
  }

  fn msg_trigger(&mut self, _target_task: usize, _last_msg_id: usize, _info: &EvalInfo) {
  }

  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo) {
    self.transition += 1;
    match to {
      &TaskState::ExtEventWait(..)       => { self.ext_wait += 1; }
      &TaskState::MessageWait(..)        => { self.msg_wait += 1; }
      &TaskState::MessageWaitNeedId(..)  => { self.msg_wait += 1; }
      &TaskState::TimeWait(..)           => { self.time_wait += 1; }
      &TaskState::Stop                   => { self.stopped += 1; }
      _                                  => {}
    };
    match event {
      &Event::Delay => { self.delayed += 1; }
      _ => {}
    }
    println!("transition: {:?} + {:?} => {:?}  / {:?}", from, event, to, info);
  }
  fn eval_finished(&mut self, _info: &EvalInfo) {}
}

#[allow(dead_code)]
pub struct TaskTracer { }

#[allow(dead_code)]
impl TaskTracer {
  pub fn new() -> TaskTracer {
    TaskTracer{}
  }
}

impl Observer for TaskTracer {
  fn eval_started(&mut self, info: &EvalInfo) {
    println!("Eval started. ({:?})",info);
  }

  fn executed(&mut self, info: &EvalInfo) {
    println!("Executed. ({:?})",info);
  }

  fn msg_trigger(&mut self, target_task: usize, last_msg_id: usize, info: &EvalInfo) {
    println!("Message trigger. target:{}, last_msg_id:{}, ({:?})", target_task, last_msg_id, info);
  }

  //fn message_sent(&mut self, channel_id: usize, last_msg_id: usize, info: &EvalInfo) {
  //  println!("Message sent. Ch:[{:}] last_msg_id:{} ({:?})",channel_id ,last_msg_id ,info);
  //}

  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo) {
    println!("Transition. ({:?})+[{:?}] => ({:?})  ({:?})", from, event, to, info);
  }

  fn eval_finished(&mut self, info: &EvalInfo) {
    println!("Eval finished. ({:?})",info);
  }
}

#[derive(Clone,Debug)]
pub struct TaskObserver {
  exec_count:   u64,
  msg_waits:    Vec<(usize, TaskState)>, // task_id, state
}

impl TaskObserver {
  pub fn new(n_tasks: usize) -> TaskObserver {
    TaskObserver{
      exec_count: 0,
      msg_waits:  Vec::with_capacity(n_tasks),
    }
  }

  pub fn exec_count(&self) -> u64 {
    self.exec_count
  }

  pub fn msg_waits(&self) -> &Vec<(usize, TaskState)> {
    &self.msg_waits
  }
}

impl Observer for TaskObserver {
  fn executed(&mut self, _info: &EvalInfo) {
    self.exec_count += 1;
  }

  fn msg_trigger(&mut self, _target_task: usize, _last_msg_id: usize, _info: &EvalInfo) {}

  fn transition(&mut self, _from: &TaskState, _event: &Event, to: &TaskState, info: &EvalInfo) {
    match to {
      &TaskState::MessageWait(..) | &TaskState::MessageWaitNeedId(..) => {
        self.msg_waits.push((info.task_id, *to));
      },
      _ => {}
    }
  }

  fn eval_started(&mut self, _info: &EvalInfo) {}
  fn eval_finished(&mut self, _info: &EvalInfo) {}
}

impl Drop for TaskObserver {
  fn drop(&mut self) {
    //println!("Drop {:?}",*self);
  }
}
