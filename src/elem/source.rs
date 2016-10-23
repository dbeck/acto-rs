use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, ChannelId, SenderName,
  ChannelWrapper, SenderChannelId, ReceiverChannelId, ChannelPosition
};
use super::counter::{OutputCounter};

pub trait Source {
  type OutputValue : Send;
  type OutputError : Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop: &mut bool);
}

pub struct SourceWrap<OutputValue: Send, OutputError: Send>
{
  name       : String,
  state      : Box<Source<OutputValue=OutputValue, OutputError=OutputError>+Send>,
  output_tx  : Sender<Message<OutputValue, OutputError>>,
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

pub fn new<OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    source          : Box<Source<OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<SourceWrap<OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      SourceWrap{
        name        : name.clone(),
        state       : source,
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
