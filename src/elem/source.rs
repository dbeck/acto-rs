use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, IdentifiedReceiver, new_id, ChannelId};
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

  fn input_id(&self, _ch_id: usize) -> Option<ChannelId> {
    None
  }
  fn tx_count(&self, ch_id: usize) -> usize {
    self.get_tx_count(ch_id)
  }
}

pub fn new<Output: Send>(
    name            : &str,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>+Send>)
      -> (Box<SourceWrap<Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size);

  (
    Box::new(
      SourceWrap{
        name        : String::from(name),
        state       : source,
        output_tx   : output_tx,
      }
    ),
    Box::new(
      Some(
          IdentifiedReceiver{
            id:     new_id(String::from(name), 0),
            input:  output_rx,
          }
        )
    )
  )
}
