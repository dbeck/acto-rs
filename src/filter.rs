extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Request, Reply, Result};

pub trait Filter {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Reply<Self::InputType>>,
    output:  &mut Sender<Reply<Self::OutputType>>) -> Result;
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

pub fn new<'a, Input: Copy+Send, Output: Copy+Send>(
    name            : String,
    input_q_size    : usize,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>>) ->
    ( FilterWrap<Input, Output>,
      Sender<Reply<Input>>,
      Receiver<Reply<Output>> )
{
  let (input_tx, input_rx) = channel(input_q_size, Reply::Empty);
  let (output_tx, outpu_rx) = channel(output_q_size, Reply::Empty);
  (
    FilterWrap{
      name        : name,
      filter      : filter,
      input_rx    : input_rx,
      output_tx   : output_tx,
    },
    input_tx,
    outpu_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
