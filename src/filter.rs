extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::channel_id::{Id, Direction};
use super::task::{Task};
use super::channel_id;

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
  input_names  : Vec<Id>,
  output_names : Vec<Id>,
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
  fn name(&self)         -> &String    { &self.name }
  fn input_names(&self)  -> &Vec<Id>   { &self.input_names }
  fn output_names(&self) -> &Vec<Id>   { &self.output_names }
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
        name          : name.clone(),
        input_names   : vec![channel_id::new(name.clone(), Direction::In, 0),],
        output_names  : vec![channel_id::new(name.clone(), Direction::Out, 0),],
        filter        : filter,
        input_rx      : input_rx,
        output_tx     : output_tx,
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
