extern crate lossyq;
extern crate parking_lot;
extern crate libc;

pub mod scheduler;
pub mod elem;

use lossyq::spsc::Receiver;

#[derive(Copy,Clone,Debug)]
pub enum Error {
  Busy,
  NonExistent,
  Stopping,
  AlreadyExists,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct InclusiveMessageRange {
  pub from: usize,
  pub to:   usize,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct ChannelPosition (usize);

#[derive(Copy,Clone,Debug)]
pub enum Message<T: Send>
{
  Empty,                                   //
  Value(T),                                //
  Ack(InclusiveMessageRange),              // from-to
  Error(ChannelPosition, &'static str),    // error at
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct SenderChannelId (usize);

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct ReceiverChannelId (usize);

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct ChannelId {
  pub sender_id:    SenderChannelId,
  pub receiver_id:  ReceiverChannelId,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct DelayFromNowInUsec (usize);

#[derive(Copy,Clone,Debug)]
pub enum Schedule {
  Loop,
  OnMessage(ChannelId, ChannelPosition),
  DelayUsec(DelayFromNowInUsec),
  OnExternalEvent,
  Stop,
}

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct AbsSchedulerTimeInUsec (usize);

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct TaskId (usize);

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct SenderId (usize);

#[derive(Clone,Debug,PartialEq)]
pub struct SenderName (String);

#[derive(Clone,Debug,PartialEq)]
pub struct ReceiverName (String);

#[derive(Copy,Clone,Debug,PartialEq)]
pub struct ExtEventSeqno (usize);

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum TaskState {
  Execute,
  TimeWait(AbsSchedulerTimeInUsec),
  MessageWait(SenderId, ChannelId, ChannelPosition),
  MessageWaitNeedSenderId(ChannelId, ChannelPosition),
  ExtEventWait(ExtEventSeqno),
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

pub trait Task {
  fn execute(&mut self) -> Schedule;
  fn name(&self) -> &String;
  fn input_count(&self) -> usize;
  fn output_count(&self) -> usize;
  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)>;
  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition;
  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition;
}

pub enum ChannelWrapper<Input: Send> {
  ReceiverNotConnected(ReceiverChannelId, ReceiverName),
  ConnectedReceiver(ChannelId, Receiver<Message<Input>>, SenderName),
  SenderNotConnected(SenderChannelId, Receiver<Message<Input>>, SenderName),
  ConnectedSender(ChannelId, ReceiverName),
}

#[cfg(test)]
pub mod tests;

#[cfg(any(test, feature = "bench"))]
pub mod sample;

#[cfg(feature = "bench")]
pub mod bench;
