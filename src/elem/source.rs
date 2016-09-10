use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, ChannelId, SenderName,
  ChannelWrapper, SenderChannelId, ReceiverChannelId};
use super::output_counter::{OutputCounter};

pub trait Source {
  type OutputType : Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct SourceWrap<Output: Send> {
  name       : String,
  state      : Box<Source<OutputType=Output>+Send>,
  output_tx  : Sender<Message<Output>>,
}

impl<Output: 'static+Send> OutputCounter for SourceWrap<Output> {
  fn get_tx_count(&self, ch_id: usize) -> usize {
    if ch_id == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<Output: 'static+Send> Task for SourceWrap<Output> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 0 }
  fn output_count(&self) -> usize { 1 }

  fn input_id(&self, _ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    None
  }
}

pub fn new<Output: Send>(
    name            : &str,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>+Send>)
      -> (Box<SourceWrap<Output>>, Box<ChannelWrapper<Output>>)
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
