extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::identified_receiver::{IdentifiedReceiver};
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

pub struct FilterWrap<Input: Copy+Send, Output: Copy+Send> {
  name         : String,
  filter       : Box<Filter<InputType=Input,OutputType=Output>>,
  input_rx     : Option<IdentifiedReceiver<Input>>,
  output_tx    : Sender<Message<Output>>,
  output_rx    : Option<IdentifiedReceiver<Output>>,
}

impl<Input: Copy+Send, Output: Copy+Send> FilterWrap<Input,Output> {
  pub fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
  pub fn output(&mut self) -> &mut Option<IdentifiedReceiver<Output>> {
    &mut self.output_rx
  }
}

impl<Input: Copy+Send, Output: Copy+Send> Task for FilterWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.filter.process(
          &mut identified.input,
          &mut self.output_tx
        )
      },
      &mut None => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: 'static+Copy+Send, Output: 'static+Copy+Send>(
    name            : &str,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>>)
      -> Box<FilterWrap<Input,Output>>
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  Box::new(
    FilterWrap{
      name        : String::from(name),
      filter      : filter,
      input_rx    : None,
      output_tx   : output_tx,
      output_rx   : Some(
        IdentifiedReceiver{
          id:     channel_id::new(String::from(name), channel_id::Direction::Out, 0),
          input:  output_rx,
        }
      ),
    }
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
