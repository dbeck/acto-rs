extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::task::Task;

pub trait Filter {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

struct FilterWrap<Input: Copy+Send, Output: Copy+Send> {
  name         : String,
  input_names  : Vec<String>,
  output_names : Vec<String>,
  filter       : Box<Filter<InputType=Input,OutputType=Output>>,
  input_rx     : Receiver<Message<Input>>,
  output_tx    : Sender<Message<Output>>,
}

impl<Input: Copy+Send, Output: Copy+Send> Task for FilterWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    self.filter.process(
      &mut self.input_rx,
      &mut self.output_tx
    )
  }
  fn name(&self)         -> &String      { &self.name }
  fn input_names(&self)  -> &Vec<String> { &self.input_names }
  fn output_names(&self) -> &Vec<String> { &self.output_names }
}

pub fn new<Input: 'static+Copy+Send, Output: 'static+Copy+Send>(
    name            : String,
    input_q_size    : usize,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>>) ->
    ( Box<Task>,
      Sender<Message<Input>>,
      Receiver<Message<Output>> )
{
  let (input_tx, input_rx)   = channel(input_q_size, Message::Empty);
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  (
    Box::new(
      FilterWrap{
        name         : name,
        input_names  : vec![],
        output_names : vec![],
        filter       : filter,
        input_rx     : input_rx,
        output_tx    : output_tx,
      }
    ),
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
