extern crate lossyq;
extern crate libc;

pub mod scheduler;
pub mod elem;

// re-exports
pub use lossyq::spsc::{Sender,Receiver};
pub use elem::{source, sink, filter, scatter, gather, ymerge, ysplit, connectable};
pub use scheduler::Scheduler;

#[derive(Copy, Clone, Debug)]
pub enum ChannelState {
  ReceiverNotConnected,
  ConnectedReceiver,
  SenderNotConnected,
  ConnectedSender,
}

#[derive(Copy, Clone, Debug)]
pub struct ExpectedChannelState (pub ChannelState);

#[derive(Copy, Clone, Debug)]
pub struct ActualChannelState (pub ChannelState);

#[derive(Copy, Clone, Debug)]
pub enum Error {
  Busy,
  NonExistent,
  Stopping,
  AlreadyExists,
  InvalidChannelState(ExpectedChannelState, ActualChannelState)
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct InclusiveMessageRange {
  pub from: usize,
  pub to:   usize,
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct ChannelPosition (pub usize);

#[derive(Copy, Clone,Debug)]
pub enum Message<ValueType: Send, ErrorType: Send> {
  Value(ValueType),
  Ack(InclusiveMessageRange),
  Error(ChannelPosition, ErrorType),
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct SenderChannelId (pub usize);

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct ReceiverChannelId (pub usize);

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct ChannelId {
  pub sender_id:    SenderChannelId,
  pub receiver_id:  ReceiverChannelId,
}

#[derive(Copy,Clone,Debug,PartialEq,Eq)]
pub struct PeriodLengthInUsec (pub usize);

#[derive(Copy,Clone,Debug)]
pub enum SchedulingRule {
  Loop,
  OnMessage,
  Periodic(PeriodLengthInUsec),
  OnExternalEvent,
}

#[derive(Copy,Clone,Debug,PartialEq,Eq,Hash)]
pub struct TaskId (usize);


#[derive(Clone,Debug,PartialEq,Eq)]
pub struct SenderName (pub String);

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct ReceiverName (pub String);

pub trait Task {
  fn execute(&mut self, stop: &mut bool);
  fn name(&self) -> &String;
  fn input_count(&self) -> usize;
  fn output_count(&self) -> usize;
  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)>;
  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition;
  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition;
}

pub enum ChannelWrapper<Value: Send, Error: Send> {
  ReceiverNotConnected(ReceiverChannelId, ReceiverName),
  ConnectedReceiver(ChannelId, Receiver<Message<Value, Error>>, SenderName),
  SenderNotConnected(SenderChannelId, Receiver<Message<Value, Error>>, SenderName),
  ConnectedSender(ChannelId, ReceiverName),
}

#[cfg(test)]
pub mod tests;

#[cfg(any(test, feature = "bench", feature = "experiment"))]
pub mod sample;

#[cfg(any(test, feature = "bench"))]
pub mod bench;
