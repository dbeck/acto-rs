extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Request, Result};

pub trait Source {
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    output: &mut Sender<Request<Self::OutputType>>) -> Result;
}

pub struct SourceWrap<Output: Copy+Send> {
  name        : String,
  source      : Box<Source<OutputType=Output>>,
  output_tx   : Sender<Request<Output>>,
}

impl<Output : Copy+Send> SourceWrap<Output> {
  pub fn process(&mut self) -> Result {
    self.source.process(&mut self.output_tx)
  }
}

pub fn new<'a, Output: Copy+Send>(
    name            : String,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>>) ->
    ( SourceWrap<Output>,
      Receiver<Request<Output>> )
{
  let (output_tx, output_rx) = channel(output_q_size, Request::Empty);
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
