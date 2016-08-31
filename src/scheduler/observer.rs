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

  //fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize, _info: &EvalInfo) {
  //  self.sent += 1;
  //}

  fn transition(&mut self, _from: &TaskState, event: &Event, to: &TaskState, _info: &EvalInfo) {
    self.transition += 1;
    match to {
      &TaskState::ExtEventWait(..) => { self.ext_wait += 1; }
      &TaskState::MessageWait(..)  => { self.msg_wait += 1; }
      &TaskState::TimeWait(..)     => { self.time_wait += 1; }
      &TaskState::Stop         => { self.stopped += 1; }
      _ => {}
    };
    match event {
      &Event::Delay => { self.delayed += 1; }
      _ => {}
    }
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
