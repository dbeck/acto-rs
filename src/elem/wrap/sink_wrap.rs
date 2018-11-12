use super::super::super::{Task, ChannelWrapper, ChannelId, SenderName,
  ReceiverChannelId, SenderChannelId, ChannelPosition
};
use super::super::connectable::{Connectable};
use super::super::identified_input::{IdentifiedInput};
use super::super::counter::{InputCounter};
use super::super::sink::{Sink};

pub struct SinkWrap<InputValue: Send, InputError: Send> {
  name      : String,
  state     : Box<Sink<InputValue=InputValue, InputError=InputError>+Send>,
  input_rx  : ChannelWrapper<InputValue, InputError>,
}

pub fn new<InputValue: Send, InputError: Send>(
          name      : String,
          state     : Box<Sink<InputValue=InputValue, InputError=InputError>+Send>,
          input_rx  : ChannelWrapper<InputValue, InputError>)
    -> SinkWrap<InputValue, InputError>
{
  SinkWrap{ name: name, state: state, input_rx: input_rx }
}

impl<InputValue: 'static+Send, InputError: 'static+Send> IdentifiedInput
    for SinkWrap<InputValue, InputError>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)>
  {
    if ch_id.0 != 0 {
      None
    } else {
      match &self.input_rx {
        ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    }
  }
}

impl<InputValue: Send, InputError: Send> InputCounter
    for SinkWrap<InputValue, InputError>
{
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 == 0 {
      if let ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) = &self.input_rx {
        receiver.seqno()
      } else {
        0
      }
    } else {
      0
    }
  }
}

impl<InputValue: Send, InputError: Send> Connectable
    for SinkWrap<InputValue, InputError>
{
  type InputValue = InputValue;
  type InputError = InputError;

  fn input(&mut self) -> &mut ChannelWrapper<InputValue, InputError> {
    &mut self.input_rx
  }
}

impl<InputValue: 'static+Send, InputError: 'static+Send> Task
    for SinkWrap<InputValue, InputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_rx, stop);
  }

  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 0 }

  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }

  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( self.get_rx_count(ch_id) )
  }

  fn output_channel_pos(&self, _ch_id: SenderChannelId) -> ChannelPosition { ChannelPosition(0) }
}
