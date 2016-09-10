use super::super::{Task, Schedule, ChannelWrapper, ChannelId, SenderName,
  ReceiverChannelId, ReceiverName
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};

pub trait Sink {
  type InputType : Send;

  fn process(
    &mut self,
    input: &mut ChannelWrapper<Self::InputType>) -> Schedule;
}

pub struct SinkWrap<Input: Send> {
  name      : String,
  state     : Box<Sink<InputType=Input>+Send>,
  input_rx  : ChannelWrapper<Input>,
}

impl<Input: Send> IdentifiedInput for SinkWrap<Input> {
  fn get_input_id(&self, ch_id: usize) -> Option<(ChannelId, SenderName)> {
    if ch_id != 0 {
      None
    } else {
      match &self.input_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    }
  }
}

impl<Input: Send> Connectable for SinkWrap<Input> {
  type Input = Input;

  fn input(&mut self) -> &mut ChannelWrapper<Input> {
    &mut self.input_rx
  }
}

impl<Input: Send> Task for SinkWrap<Input> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 0 }

  fn input_id(&self, ch_id: usize) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }
}

pub fn new<Input: Send>(
    name   : &str,
    sink   : Box<Sink<InputType=Input>+Send>)
      -> Box<SinkWrap<Input>>
{
  let name = String::from(name);
  Box::new(
    SinkWrap{
      name          : String::from(name.clone()),
      state         : sink,
      input_rx      : ChannelWrapper::ReceiverNotConnected(
        ReceiverChannelId(0),
        ReceiverName (name)
      ),
    }
  )
}
