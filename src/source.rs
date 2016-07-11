extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::task::Task;

pub trait Source {
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

struct SourceWrap<Output: Copy+Send> {
  name         : String,
  input_names  : Vec<String>,
  output_names : Vec<String>,
  source       : Box<Source<OutputType=Output>>,
  output_tx    : Sender<Message<Output>>,
}

impl<Output: Copy+Send> Task for SourceWrap<Output> {
  fn execute(&mut self) -> Schedule {
    self.source.process(&mut self.output_tx)
  }
  fn name(&self)         -> &String      { &self.name }
  fn input_names(&self)  -> &Vec<String> { &self.input_names }
  fn output_names(&self) -> &Vec<String> { &self.output_names }
}

pub fn new<Output: 'static+Copy+Send>(
    name            : String,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>>) ->
    ( Box<Task>, Receiver<Message<Output>> )
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  (
    Box::new(
      SourceWrap{
      name         : name,
      input_names  : vec![],
      output_names : vec![],
      source       : source,
      output_tx    : output_tx
      }
    ),
    output_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
