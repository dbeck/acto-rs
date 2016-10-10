use super::super::{TaskState, Event, TaskId, AbsSchedulerTimeInUsec};
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

  #[allow(dead_code)]
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

  #[allow(dead_code)]
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
  fn msg_trigger(&mut self, target_task: TaskId, info: &EvalInfo);
  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo);
  fn eval_finished(&mut self, info: &EvalInfo);
}

#[derive(Clone,Debug)]
pub struct TaskObserver {
  exec_count:     u64,
  msg_waits:      Vec<(TaskId, TaskState)>,
  msg_triggers:   Vec<TaskId>,
}

impl TaskObserver {
  pub fn new(n_tasks: usize) -> TaskObserver {
    TaskObserver{
      exec_count:   0,
      msg_waits:    Vec::with_capacity(n_tasks),
      msg_triggers: Vec::with_capacity(n_tasks),
    }
  }

  #[allow(dead_code)]
  pub fn msg_waits(&self) -> &Vec<(TaskId, TaskState)> {
    &self.msg_waits
  }

  #[allow(dead_code)]
  pub fn msg_triggers(&self) -> &Vec<TaskId> {
    &self.msg_triggers
  }

  #[allow(dead_code)]
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
                 _info: &EvalInfo) {
    //println!(" .. Trigger: task:{:?} position:{:?}  info:{:?}",target_task, channel_position, info);
    self.msg_triggers.push(target_task);
  }

  fn transition(&mut self, _from: &TaskState, _event: &Event, to: &TaskState, info: &EvalInfo) {
    //println!(" .. Transition: {:?} + {:?} -> {:?}  =@= {:?}", from, event, to, info);
    match to {
      &TaskState::MessageWait(..) => {
        //println!(" // Message dependency: info:{:?}", info);
        self.msg_waits.push((info.task_id, *to));
      },
      _ => {}
    }
  }

  fn eval_started(&mut self, _info: &EvalInfo) {}
  fn eval_finished(&mut self, _info: &EvalInfo) {}
}
