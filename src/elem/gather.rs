use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderName, SenderChannelId, ReceiverChannelId, ReceiverName, ChannelPosition
};
use super::connectable::{ConnectableN};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait Gather {
  type InputValue   : Send;
  type InputError   : Send;
  type OutputValue  : Send;
  type OutputError  : Send;

  fn process(
    &mut self,
    input:   &mut Vec<ChannelWrapper<Self::InputValue, Self::InputError>>,
    output:  &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop:    &mut bool);
}

pub struct GatherWrap<InputValue: Send, InputError: Send,
                      OutputValue: Send, OutputError: Send> {
  name           : String,
  state          : Box<Gather<InputValue=InputValue, InputError=InputError,
                              OutputValue=OutputValue, OutputError=OutputError>+Send>,
  input_rx_vec   : Vec<ChannelWrapper<InputValue, InputError>>,
  output_tx      : Sender<Message<OutputValue, OutputError>>,
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

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    gather          : Box<Gather<InputValue=InputValue,   InputError=InputError,
                                 OutputValue=OutputValue, OutputError=OutputError>+Send>,
    n_channels      : usize)
      -> (Box<GatherWrap<InputValue, InputError, OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);
  let mut inputs = vec![];
  for i in 0..n_channels {
    inputs.push(ChannelWrapper::ReceiverNotConnected(
      ReceiverChannelId(i),
      ReceiverName (name.clone())
    ));
  }

  (
    Box::new(
      GatherWrap{
        name                   : name.clone(),
        state                  : gather,
        input_rx_vec           : inputs,
        output_tx              : output_tx,
      }
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
