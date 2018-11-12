use lossyq::spsc::{Sender};
use super::super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderName, SenderChannelId, ReceiverChannelId, ChannelPosition
};
use super::super::connectable::{Connectable};
use super::super::identified_input::{IdentifiedInput};
use super::super::counter::{OutputCounter, InputCounter};
use super::super::scatter::{Scatter};

pub struct ScatterWrap<InputValue: Send, InputError: Send,
                      OutputValue: Send, OutputError: Send> {
  name           : String,
  state          : Box<Scatter<InputValue=InputValue, InputError=InputError,
                               OutputValue=OutputValue, OutputError=OutputError>+Send>,
  input_rx       : ChannelWrapper<InputValue, InputError>,
  output_tx_vec  : Vec<Sender<Message<OutputValue, OutputError>>>,
}

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
          name           : String,
          state          : Box<Scatter<InputValue=InputValue, InputError=InputError,
                                       OutputValue=OutputValue, OutputError=OutputError>+Send>,
          input_rx       : ChannelWrapper<InputValue, InputError>,
          output_tx_vec  : Vec<Sender<Message<OutputValue, OutputError>>>)
    -> ScatterWrap<InputValue, InputError, OutputValue, OutputError>
{
  ScatterWrap{ name: name, state: state, input_rx: input_rx, output_tx_vec: output_tx_vec }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> IdentifiedInput
    for ScatterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
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

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> InputCounter
    for ScatterWrap<InputValue, InputError, OutputValue, OutputError>
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

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> OutputCounter
    for ScatterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 < self.output_tx_vec.len() {
      let otx_slice = self.output_tx_vec.as_slice();
      otx_slice[ch_id.0].seqno()
    } else {
      0
    }
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> Connectable
    for ScatterWrap<InputValue, InputError, OutputValue, OutputError>
{
  type InputValue = InputValue;
  type InputError = InputError;

  fn input(&mut self) -> &mut ChannelWrapper<Self::InputValue, Self::InputError> {
    &mut self.input_rx
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> Task
    for ScatterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_rx,
                       &mut self.output_tx_vec,
                       stop);
  }

  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { self.output_tx_vec.len() }

  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }

  fn input_channel_pos(&self, ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( self.get_rx_count(ch_id) )
  }

  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition {
    ChannelPosition( self.get_tx_count(ch_id) )
  }
}
