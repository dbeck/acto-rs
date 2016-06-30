extern crate lossyq;
use self::lossyq::spsc::Receiver;
use super::common::{Reply, Result};

pub trait Sink {
  type InputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Reply<Self::InputType>>) -> Result;
}

pub struct SinkWrap<Input: Copy+Send> {
  name        : String,
  sink        : Box<Sink<InputType=Input>>,
  input_rx    : Receiver<Reply<Input>>,
}

impl<Input : Copy+Send> SinkWrap<Input> {
  pub fn process(&mut self) -> Result {
    self.sink.process(&mut self.input_rx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
