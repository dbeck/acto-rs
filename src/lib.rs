extern crate lossyq;
extern crate parking_lot;
extern crate time;
extern crate libc;

pub mod scheduler;
pub mod elem;

use lossyq::spsc::Receiver;
use std::fmt;
use std::sync::atomic::{AtomicUsize, Ordering};

#[derive(Copy,Clone,Debug)]
pub enum Error {
  Busy,
  NonExistent,
  Stopping,
  AlreadyExists,
}

#[derive(Copy,Clone,Debug)]
pub enum Message<T: Send>
{
  Empty,                      //
  Value(T),                   //
  Ack(usize,usize),           // from-to
  Error(usize,&'static str),  // error at
}

#[derive(Clone,Debug)]
pub struct ChannelId {
  task_name  : String,
  id         : usize,
}

#[derive(Copy,Clone,Debug)]
pub enum Schedule {
  Loop,
  OnMessage(usize, usize), // channel id, msg id
  DelayUSec(u64),
  OnExternalEvent,
  Stop,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TaskState {
  Execute,
  TimeWait(usize),
  MessageWait(usize, usize, usize), // input_task_id, ch_id, msg_id
  MessageWaitNeedId(usize, usize),  // ch_id, msg_id
  ExtEventWait(usize),
  Stop,
}

#[derive(Copy,Clone,Debug)]
pub enum Event {
  User(Schedule),
  Execute,
  TimerExpired,
  MessageArrived,
  ExtTrigger,
  Delay,
}

#[derive(Copy,Clone,Debug)]
pub struct EvalInfo {
  task_id: usize,
  at_usec: usize,
  eval_id: usize,
}

impl EvalInfo {
  pub fn new(task_id: usize, at_usec: &AtomicUsize, eval_id: usize) -> EvalInfo {
    EvalInfo{
      task_id:   task_id,
      at_usec:   at_usec.load(Ordering::Acquire),
      eval_id:   eval_id
    }
  }
  pub fn new_with_usec(task_id: usize, at_usec: usize, eval_id: usize) -> EvalInfo {
    EvalInfo{
      task_id:   task_id,
      at_usec:   at_usec,
      eval_id:   eval_id
    }
  }
  pub fn update_at(&mut self, at_usec: &AtomicUsize) {
    self.at_usec = at_usec.load(Ordering::Acquire);
  }
  pub fn get_usec(&self) -> usize {
    self.at_usec
  }
}

pub trait Observer {
  fn eval_started(&mut self, info: &EvalInfo);
  fn executed(&mut self, info: &EvalInfo);
  // fn message_sent(&mut self, channel_id: usize, last_msg_id: usize, info: &EvalInfo);
  fn transition(&mut self, from: &TaskState, event: &Event, to: &TaskState, info: &EvalInfo);
  fn eval_finished(&mut self, info: &EvalInfo);
}

pub trait Task {
  fn execute(&mut self) -> Schedule;
  fn name(&self)  -> &String;
  fn input_count(&self) -> usize;
  fn output_count(&self) -> usize;
  fn input_id(&self, ch_id: usize) -> Option<ChannelId>;

  fn output_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id >= self.output_count() {
      None
    } else {
      Some( new_id(self.name().clone(), ch_id))
    }
  }
  fn tx_count(&self, ch_id: usize) -> usize;
}

pub struct IdentifiedReceiver<Input: Send> {
  pub id    : ChannelId,
  pub input : Receiver<Message<Input>>,
}

impl fmt::Display for ChannelId {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id:({} {})", self.task_name, self.id)
    }
}

pub fn new_id(name: String, id: usize) -> ChannelId {
  ChannelId {
    task_name  : name,
    id         : id,
  }
}


#[cfg(test)]
pub mod tests;
