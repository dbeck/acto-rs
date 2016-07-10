extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
// use super::task::{Task};

pub trait Filter {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct FilterWrap<Input: Copy+Send, Output: Copy+Send> {
  name        : String,
  filter      : Box<Filter<InputType=Input,OutputType=Output>>,
  input_rx    : Receiver<Message<Input>>,
  output_tx   : Sender<Message<Output>>,
}

impl<Input : Copy+Send, Output : Copy+Send> FilterWrap<Input, Output> {
  pub fn process(&mut self) -> Schedule {
    self.filter.process(&mut self.input_rx, &mut self.output_tx)
  }
}

pub fn new<Input: Copy+Send, Output: Copy+Send>(
    name            : String,
    input_q_size    : usize,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>>) ->
    ( FilterWrap<Input, Output>,
      Sender<Message<Input>>,
      Receiver<Message<Output>> )
{
  let (input_tx, input_rx)   = channel(input_q_size, Message::Empty);
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  (
    FilterWrap{
      name        : name,
      filter      : filter,
      input_rx    : input_rx,
      output_tx   : output_tx,
    },
    input_tx,
    output_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
