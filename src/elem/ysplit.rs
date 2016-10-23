use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderChannelId, ReceiverChannelId, ReceiverName, SenderName, ChannelPosition
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::counter::{OutputCounter, InputCounter};

pub trait YSplit {
  type InputValue    : Send;
  type InputError    : Send;
  type OutputValueA  : Send;
  type OutputErrorA  : Send;
  type OutputValueB  : Send;
  type OutputErrorB  : Send;

  fn process(
    &mut self,
    input:     &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    output_a:  &mut Sender<Message<Self::OutputValueA, Self::OutputErrorA>>,
    output_b:  &mut Sender<Message<Self::OutputValueB, Self::OutputErrorB>>,
    stop:      &mut bool);
}

pub struct YSplitWrap<InputValue: Send,   InputError: Send,
                      OutputValueA: Send, OutputErrorA: Send,
                      OutputValueB: Send, OutputErrorB: Send> {
  name          : String,
  state         : Box<YSplit<InputValue=InputValue, InputError=InputError,
                             OutputValueA=OutputValueA, OutputErrorA=OutputErrorA,
                             OutputValueB=OutputValueB, OutputErrorB=OutputErrorB>+Send>,
  input_rx      : ChannelWrapper<InputValue, InputError>,
  output_a_tx   : Sender<Message<OutputValueA, OutputErrorA>>,
  output_b_tx   : Sender<Message<OutputValueB, OutputErrorB>>,
}

impl<InputValue: Send,   InputError: Send,
     OutputValueA: Send, OutputErrorA: Send,
     OutputValueB: Send, OutputErrorB: Send> IdentifiedInput
    for YSplitWrap<InputValue, InputError,
                   OutputValueA, OutputErrorA,
                   OutputValueB, OutputErrorB>
{
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 != 0 {
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

impl<InputValue: Send,   InputError: Send,
     OutputValueA: Send, OutputErrorA: Send,
     OutputValueB: Send, OutputErrorB: Send> InputCounter
    for YSplitWrap<InputValue, InputError,
                   OutputValueA, OutputErrorA,
                   OutputValueB, OutputErrorB>
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

impl<InputValue: Send,   InputError: Send,
     OutputValueA: Send, OutputErrorA: Send,
     OutputValueB: Send, OutputErrorB: Send> OutputCounter
    for YSplitWrap<InputValue, InputError,
                   OutputValueA, OutputErrorA,
                   OutputValueB, OutputErrorB>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_a_tx.seqno()
    } else if ch_id.0 == 1 {
      self.output_b_tx.seqno()
    } else {
      0
    }
  }
}

impl<InputValue: Send,   InputError: Send,
     OutputValueA: Send, OutputErrorA: Send,
     OutputValueB: Send, OutputErrorB: Send> Connectable
    for YSplitWrap<InputValue, InputError,
                   OutputValueA, OutputErrorA,
                   OutputValueB, OutputErrorB>
{
  type InputValue = InputValue;
  type InputError = InputError;

  fn input(&mut self) -> &mut ChannelWrapper<InputValue, InputError> {
    &mut self.input_rx
  }
}

impl<InputValue: Send,   InputError: Send,
     OutputValueA: Send, OutputErrorA: Send,
     OutputValueB: Send, OutputErrorB: Send> Task
    for YSplitWrap<InputValue, InputError,
                   OutputValueA, OutputErrorA,
                   OutputValueB, OutputErrorB>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.input_rx,
                       &mut self.output_a_tx,
                       &mut self.output_b_tx,
                       stop);
  }

  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 2 }

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

pub fn new<InputValue: Send,   InputError: Send,
           OutputValueA: Send, OutputErrorA: Send,
           OutputValueB: Send, OutputErrorB: Send>(
    name              : &str,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputValue=InputValue, InputError=InputError,
                                   OutputValueA=OutputValueA, OutputErrorA=OutputErrorA,
                                   OutputValueB=OutputValueB, OutputErrorB=OutputErrorB>+Send>)
      -> (Box<YSplitWrap<InputValue, InputError,
                        OutputValueA, OutputErrorA,
                        OutputValueB, OutputErrorB>>,
          Box<ChannelWrapper<OutputValueA, OutputErrorA>>,
          Box<ChannelWrapper<OutputValueB, OutputErrorB>>)
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size);
  let name = String::from(name);

  (
    Box::new(
      YSplitWrap{
        name          : name.clone(),
        state         : ysplit,
        input_rx      : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_a_tx   : output_a_tx,
        output_b_tx   : output_b_tx,
      }
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_a_rx,
        SenderName(name.clone())
      )
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(1),
        output_b_rx,
        SenderName(name)
      )
    ),
  )
}
