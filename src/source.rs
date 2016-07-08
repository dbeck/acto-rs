extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};

pub trait Source {
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct SourceWrap<Output: Copy+Send> {
  name        : String,
  source      : Box<Source<OutputType=Output>>,
  output_tx   : Sender<Message<Output>>,
}

impl<Output : Copy+Send> SourceWrap<Output> {
  pub fn process(&mut self) -> Schedule {
    self.source.process(&mut self.output_tx)
  }
}

pub fn new<Output: Copy+Send>(
    name            : String,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>>) ->
    ( SourceWrap<Output>,
      Receiver<Message<Output>> )
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  (
    SourceWrap{
      name        : name,
      source      : source,
      output_tx   : output_tx,
    },
    output_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
