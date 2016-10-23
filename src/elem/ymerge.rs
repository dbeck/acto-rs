use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelWrapper, ChannelId, SenderName,
  ReceiverChannelId, ReceiverName, SenderChannelId, ChannelPosition
};
use super::connectable::{ConnectableY};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait YMerge {
  type InputValueA   : Send;
  type InputErrorA   : Send;
  type InputValueB   : Send;
  type InputErrorB   : Send;
  type OutputValue   : Send;
  type OutputError   : Send;

  fn process(
    &mut self,
    input_a:  &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>,
    input_b:  &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>,
    output:   &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop:     &mut bool);
}

pub struct YMergeWrap<InputValueA: Send, InputErrorA: Send,
                      InputValueB: Send, InputErrorB: Send,
                      OutputValue: Send, OutputError: Send>
{
  name         : String,
  state        : Box<YMerge<InputValueA=InputValueA, InputErrorA=InputErrorA,
                            InputValueB=InputValueB, InputErrorB=InputErrorB,
                            OutputValue=OutputValue, OutputError=OutputError>+Send>,
  input_a_rx   : ChannelWrapper<InputValueA, InputErrorA>,
  input_b_rx   : ChannelWrapper<InputValueB, InputErrorB>,
  output_tx    : Sender<Message<OutputValue, OutputError>>,
}

impl<InputValueA: Send, InputErrorA: Send,
     InputValueB: Send, InputErrorB: Send,
     OutputValue: Send, OutputError: Send> IdentifiedInput
    for YMergeWrap<InputValueA, InputErrorA,
                   InputValueB, InputErrorB,
                   OutputValue, OutputError>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 > 1 {
      None
    } else if ch_id.0 == 0 {
      match &self.input_a_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None
      }
    } else {
      match &self.input_b_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None
      }
    }
  }
}

impl<InputValueA: Send, InputErrorA: Send,
     InputValueB: Send, InputErrorB: Send,
     OutputValue: Send, OutputError: Send> InputCounter
    for YMergeWrap<InputValueA, InputErrorA, InputValueB, InputErrorB, OutputValue, OutputError>
{
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize {
    if ch_id.0 > 1 {
      0
    } else if ch_id.0 == 0 {
      match &self.input_a_rx {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0
      }
    } else {
      match &self.input_b_rx {
        &ChannelWrapper::ConnectedReceiver(ref _channel_id, ref receiver, ref _sender_name) => {
          receiver.seqno()
        },
        _ => 0
      }
    }
  }
}

impl<InputValueA: Send, InputErrorA: Send,
     InputValueB: Send, InputErrorB: Send,
     OutputValue: Send, OutputError: Send> OutputCounter
    for YMergeWrap<InputValueA, InputErrorA,
                   InputValueB, InputErrorB,
                   OutputValue, OutputError>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<InputValueA: Send, InputErrorA: Send,
     InputValueB: Send, InputErrorB: Send,
     OutputValue: Send, OutputError: Send> ConnectableY
    for YMergeWrap<InputValueA, InputErrorA,
                   InputValueB, InputErrorB,
                   OutputValue, OutputError>
{
  type InputValueA = InputValueA;
  type InputErrorA = InputErrorA;
  type InputValueB = InputValueB;
  type InputErrorB = InputErrorB;

  fn input_a(&mut self) -> &mut ChannelWrapper<InputValueA, InputErrorA> {
    &mut self.input_a_rx
  }

  fn input_b(&mut self) -> &mut ChannelWrapper<InputValueB, InputErrorB> {
    &mut self.input_b_rx
  }
}

impl<InputValueA: Send, InputErrorA: Send,
     InputValueB: Send, InputErrorB: Send,
     OutputValue: Send, OutputError: Send> Task
    for YMergeWrap<InputValueA, InputErrorA,
                   InputValueB, InputErrorB,
                   OutputValue, OutputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_a_rx,
                       &mut self.input_b_rx,
                       &mut self.output_tx,
                       stop);
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 2 }
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

pub fn new<InputValueA: Send, InputErrorA: Send,
           InputValueB: Send, InputErrorB: Send,
           OutputValue: Send, OutputError: Send>(
    name             : &str,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputValueA=InputValueA, InputErrorA=InputErrorA,
                                  InputValueB=InputValueB, InputErrorB=InputErrorB,
                                  OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<YMergeWrap<InputValueA, InputErrorA,
                         InputValueB, InputErrorB,
                         OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx,  output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      YMergeWrap{
        name          : name.clone(),
        state         : ymerge,
        input_a_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        input_b_rx    : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(1),
          ReceiverName (name.clone())
        ),
        output_tx     : output_tx,
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
