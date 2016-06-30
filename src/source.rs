extern crate lossyq;
use self::lossyq::spsc::Sender;
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

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
