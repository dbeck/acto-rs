use lossyq::spsc::{Sender};
use super::super::super::{Task, Message, ChannelWrapper, ChannelId,
  SenderChannelId, ReceiverChannelId, SenderName, ChannelPosition
};
use super::super::connectable::{Connectable};
use super::super::identified_input::{IdentifiedInput};
use super::super::counter::{OutputCounter, InputCounter};
use super::super::ysplit::{YSplit};

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

pub fn new<InputValue: Send,   InputError: Send,
           OutputValueA: Send, OutputErrorA: Send,
           OutputValueB: Send, OutputErrorB: Send>(
             name          : String,
             state         : Box<YSplit<InputValue=InputValue, InputError=InputError,
                                        OutputValueA=OutputValueA, OutputErrorA=OutputErrorA,
                                        OutputValueB=OutputValueB, OutputErrorB=OutputErrorB>+Send>,
             input_rx      : ChannelWrapper<InputValue, InputError>,
             output_a_tx   : Sender<Message<OutputValueA, OutputErrorA>>,
             output_b_tx   : Sender<Message<OutputValueB, OutputErrorB>>)
      -> YSplitWrap<InputValue, InputError, OutputValueA, OutputErrorA, OutputValueB, OutputErrorB>
{
  YSplitWrap{
    name: name,
    state: state,
    input_rx: input_rx,
    output_a_tx: output_a_tx,
    output_b_tx: output_b_tx
  }
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
