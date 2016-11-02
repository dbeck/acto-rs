use lossyq::spsc::{Sender};
use super::super::super::{Task, Message, ChannelId, SenderName,
  SenderChannelId, ReceiverChannelId, ChannelPosition
};
use super::super::counter::{OutputCounter};
use super::super::source::{Source};

pub struct SourceWrap<OutputValue: Send, OutputError: Send>
{
  name       : String,
  state      : Box<Source<OutputValue=OutputValue, OutputError=OutputError>+Send>,
  output_tx  : Sender<Message<OutputValue, OutputError>>,
}

pub fn new<OutputValue: Send, OutputError: Send>(
           name       : String,
           state      : Box<Source<OutputValue=OutputValue, OutputError=OutputError>+Send>,
           output_tx  : Sender<Message<OutputValue, OutputError>>)
  -> SourceWrap<OutputValue, OutputError>
{
  SourceWrap{ name: name, state: state, output_tx: output_tx }
}

impl<OutputValue: Send, OutputError: Send> OutputCounter
    for SourceWrap<OutputValue, OutputError>
{
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize {
    if ch_id.0 == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<OutputValue: Send, OutputError: Send> Task
    for SourceWrap<OutputValue, OutputError>
{
  fn execute(&mut self, stop: &mut bool) {
    self.state.process(&mut self.output_tx, stop);
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 0 }
  fn output_count(&self) -> usize { 1 }

  fn input_id(&self, _ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    None
  }

  fn input_channel_pos(&self, _ch_id: ReceiverChannelId) -> ChannelPosition {
    ChannelPosition( 0 )
  }

  fn output_channel_pos(&self, ch_id: SenderChannelId) -> ChannelPosition {
    ChannelPosition( self.get_tx_count(ch_id) )
  }
}
