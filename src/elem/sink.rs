use super::super::{Task, Schedule, IdentifiedReceiver, ChannelId};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};

pub trait Sink {
  type InputType : Send;

  fn process(
    &mut self,
    input: &mut Option<IdentifiedReceiver<Self::InputType>>) -> Schedule;
}

pub struct SinkWrap<Input: Send> {
  name      : String,
  state     : Box<Sink<InputType=Input>+Send>,
  input_rx  : Option<IdentifiedReceiver<Input>>,
}

impl<Input: Send> IdentifiedInput for SinkWrap<Input> {
  fn get_input_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id != 0 {
      None
    } else {
      match &self.input_rx {
        &Some(ref ch) => Some(ch.id.clone()),
        _             => None,
      }
    }
  }
}

impl<Input: Send> Connectable for SinkWrap<Input> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
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

  fn input_id(&self, ch_id: usize) -> Option<ChannelId> {
    self.get_input_id(ch_id)
  }
  fn tx_count(&self, _ch_id: usize) -> usize { 0 }
}

pub fn new<Input: Send>(
    name   : &str,
    sink   : Box<Sink<InputType=Input>+Send>)
      -> Box<SinkWrap<Input>>
{
  Box::new(
    SinkWrap{
      name          : String::from(name),
      state         : sink,
      input_rx      : None,
      }
    )
}
