extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver};
use super::common::{Reply, Result};

pub trait Filter {
  type InputType : Copy+Send;
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Reply<Self::InputType>>,
    output: &mut Sender<Reply<Self::OutputType>>) -> Result;
}

pub struct FilterWrap<Input: Copy+Send, Output: Copy+Send> {
  name        : String,
  filter      : Box<Filter<InputType=Input,OutputType=Output>>,
  input_rx    : Receiver<Reply<Input>>,
  output_tx   : Sender<Reply<Output>>,
}

impl<Input : Copy+Send, Output : Copy+Send> FilterWrap<Input, Output> {
  pub fn process(&mut self) -> Result {
    self.filter.process(&mut self.input_rx, &mut self.output_tx)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
