use super::super::{TaskState, Event, TaskId, AbsSchedulerTimeInUsec, ChannelPosition};
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy,Clone,Debug)]
pub struct EvalInfo {
  task_id: TaskId,
  at_usec: AbsSchedulerTimeInUsec,
  eval_id: usize,
}

impl EvalInfo {
  #[allow(dead_code)]
  pub fn new(task_id: TaskId, at_usec: &AtomicUsize, eval_id: usize) -> EvalInfo {
    EvalInfo{
      task_id:   task_id,
      at_usec:   AbsSchedulerTimeInUsec (at_usec.load(Ordering::Acquire)),
      eval_id:   eval_id
    }
  }
  pub fn new_with_usec(task_id: TaskId, at_usec: usize, eval_id: usize) -> EvalInfo {
    EvalInfo{
      task_id:   task_id,
      at_usec:   AbsSchedulerTimeInUsec (at_usec),
      eval_id:   eval_id
    }
  }
  #[allow(dead_code)]
  pub fn update_at(&mut self, at_usec: &AtomicUsize) {
    self.at_usec = AbsSchedulerTimeInUsec (at_usec.load(Ordering::Acquire));
  }
  pub fn update_at_with_usec(&mut self, at_usec: usize) {
    self.at_usec = AbsSchedulerTimeInUsec (at_usec);
  }
  #[allow(dead_code)]
  pub fn get_usec(&self) -> AbsSchedulerTimeInUsec {
    self.at_usec
  }
}

pub trait Observer {
  fn eval_started(&mut self, info: &EvalInfo);
  fn executed(&mut self, info: &EvalInfo);
  fn msg_trigger(&mut self, target_task: TaskId, channel_position: ChannelPosition, info: &EvalInfo);
  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo);
  fn eval_finished(&mut self, info: &EvalInfo);
}

#[derive(Copy,Clone,Debug)]
pub struct CountingReporter {
  pub scheduled:   usize,
  pub executed:    usize,
  pub stopped:     usize,
  pub delayed:     usize,
  pub time_wait:   usize,
  pub msg_wait:    usize,
  pub ext_wait:    usize,
  pub channel:     usize,
  pub transition:  usize,
  pub triggered:   usize,
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
      channel:     0,
      transition:  0,
      triggered:   0,
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

  fn msg_trigger(&mut self,
                 _target_task: TaskId,
                 _channel_position: ChannelPosition,
                 _info: &EvalInfo)
  {
    self.triggered += 1;
  }

  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo) {
    self.transition += 1;
    match to {
      &TaskState::ExtEventWait(..)             => { self.ext_wait  += 1; },
      &TaskState::MessageWait(..)              => { self.msg_wait  += 1; },
      &TaskState::MessageWaitNeedSenderId(..)  => { self.msg_wait  += 1; },
      &TaskState::TimeWait(..)                 => { self.time_wait += 1; },
      &TaskState::Stop                         => { self.stopped   += 1; },
      _                                        => { },
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

  fn msg_trigger(&mut self,
                 target_task: TaskId,
                 channel_position: ChannelPosition,
                 info: &EvalInfo)
  {
    println!("Message trigger. target:{:?}, pos:{:?}, ({:?})",
      target_task, channel_position, info);
  }

  fn transition(&mut self,
                from: &TaskState,
                event: &Event,
                to: &TaskState,
                info: &EvalInfo)
  {
    println!("Transition. ({:?})+[{:?}] => ({:?})  ({:?})", from, event, to, info);
  }

  fn eval_finished(&mut self, info: &EvalInfo) {
    println!("Eval finished. ({:?})",info);
  }
}

#[derive(Clone,Debug)]
pub struct TaskObserver {
  exec_count:     u64,
  msg_waits:      Vec<(TaskId, TaskState)>,
  msg_triggers:   Vec<(TaskId, ChannelPosition)>,
}

impl TaskObserver {
  pub fn new(n_tasks: usize) -> TaskObserver {
    TaskObserver{
      exec_count:   0,
      msg_waits:    Vec::with_capacity(n_tasks),
      msg_triggers: Vec::with_capacity(n_tasks),
    }
  }

  pub fn msg_waits(&self) -> &Vec<(TaskId, TaskState)> {
    &self.msg_waits
  }

  pub fn msg_triggers(&self) -> &Vec<(TaskId, ChannelPosition)> {
    &self.msg_triggers
  }

  pub fn exec_count(&self) -> u64 {
    self.exec_count
  }
}

impl Observer for TaskObserver {
  fn executed(&mut self, _info: &EvalInfo) {
    self.exec_count += 1;
  }

  fn msg_trigger(&mut self,
                 target_task: TaskId,
                 channel_position: ChannelPosition,
                 _info: &EvalInfo) {
    //println!(" .. Trigger: task:{:?} position:{:?}  info:{:?}",target_task, channel_position, info);
    self.msg_triggers.push((target_task, channel_position));
  }

  fn transition(&mut self, _from: &TaskState, _event: &Event, to: &TaskState, info: &EvalInfo) {
    //println!(" .. Transition: {:?} + {:?} -> {:?}  =@= {:?}", from, event, to, info);
    match to {
      &TaskState::MessageWait(..) | &TaskState::MessageWaitNeedSenderId(..) => {
        //println!(" // Message dependency: info:{:?}", info);
        self.msg_waits.push((info.task_id, *to));
      },
      _ => {}
    }
  }

  fn eval_started(&mut self, _info: &EvalInfo) {}
  fn eval_finished(&mut self, _info: &EvalInfo) {}
}
