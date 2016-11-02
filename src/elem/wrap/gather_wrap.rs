use lossyq::spsc::{Sender};
use super::super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderName, SenderChannelId, ReceiverChannelId, ChannelPosition
};
use super::super::connectable::{ConnectableN};
use super::super::identified_input::{IdentifiedInput};
use super::super::counter::{OutputCounter, InputCounter};
use super::super::gather::{Gather};

pub struct GatherWrap<InputValue: Send, InputError: Send,
                      OutputValue: Send, OutputError: Send> {
  name           : String,
  state          : Box<Gather<InputValue=InputValue, InputError=InputError,
                              OutputValue=OutputValue, OutputError=OutputError>+Send>,
  input_rx_vec   : Vec<ChannelWrapper<InputValue, InputError>>,
  output_tx      : Sender<Message<OutputValue, OutputError>>,
}

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
          name           : String,
          state          : Box<Gather<InputValue=InputValue, InputError=InputError,
                                      OutputValue=OutputValue, OutputError=OutputError>+Send>,
          input_rx_vec   : Vec<ChannelWrapper<InputValue, InputError>>,
          output_tx      : Sender<Message<OutputValue, OutputError>>)
    -> GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  GatherWrap{ name: name, state: state, input_rx_vec: input_rx_vec, output_tx: output_tx }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> IdentifiedInput
    for GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 < self.input_rx_vec.len() {
      let slice = self.input_rx_vec.as_slice();
      match &slice[ch_id.0] {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    } else {
      None
    }
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> InputCounter
    for GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 < self.input_rx_vec.len() {
      let slice = self.input_rx_vec.as_slice();
      match &slice[ch_id.0] {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0,
      }
    } else {
      0
    }
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> OutputCounter
    for GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> ConnectableN
    for GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  type InputValue = InputValue;
  type InputError = InputError;

  fn input(&mut self, n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputValue, Self::InputError> {
    let ret_slice = self.input_rx_vec.as_mut_slice();
    &mut ret_slice[n.0]
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> Task
    for GatherWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_rx_vec,
                       &mut self.output_tx,
                       stop);
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { self.input_rx_vec.len() }
  fn output_count(&self) -> usize { 1 }

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
