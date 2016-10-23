use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderChannelId, ReceiverChannelId, ReceiverName, SenderName, ChannelPosition
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait Filter {
  type InputValue   : Send;
  type InputError   : Send;
  type OutputValue  : Send;
  type OutputError  : Send;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    output:  &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop: &mut bool);
}

pub struct FilterWrap<InputValue: Send, InputError: Send,
                      OutputValue: Send, OutputError: Send> {
  name         : String,
  state        : Box<Filter<InputValue=InputValue, InputError=InputError,
                            OutputValue=OutputValue, OutputError=OutputError>+Send>,
  input_rx     : ChannelWrapper<InputValue, InputError>,
  output_tx    : Sender<Message<OutputValue, OutputError>>,
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> IdentifiedInput
    for FilterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 == 0 {
      match &self.input_rx {
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
    for FilterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 == 0 {
      if let &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) = &self.input_rx {
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
    for FilterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> Connectable
    for FilterWrap<InputValue, InputError, OutputValue, OutputError>
{
  type InputValue = InputValue;
  type InputError = InputError;

  fn input(&mut self) -> &mut ChannelWrapper<InputValue, InputError> {
    &mut self.input_rx
  }
}

impl<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send> Task
    for FilterWrap<InputValue, InputError, OutputValue, OutputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_rx, &mut self.output_tx, stop);
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
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
    filter          : Box<Filter<InputValue=InputValue, InputError=InputError,
                                 OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<FilterWrap<InputValue, InputError, OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      FilterWrap{
        name        : name.clone(),
        state       : filter,
        input_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_tx   : output_tx,
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
