extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::task::Task;

pub trait Sink {
  type InputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>) -> Schedule;
}

struct SinkWrap<Input: Copy+Send> {
  name         : String,
  input_names  : Vec<String>,
  output_names : Vec<String>,
  sink         : Box<Sink<InputType=Input>>,
  input_rx     : Receiver<Message<Input>>,
}

impl<Input: Copy+Send> Task for SinkWrap<Input> {
  fn execute(&mut self) -> Schedule {
    self.sink.process(&mut self.input_rx)
  }
  fn name(&self)         -> &String      { &self.name }
  fn input_names(&self)  -> &Vec<String> { &self.input_names }
  fn output_names(&self) -> &Vec<String> { &self.output_names }
}

pub fn new<Input: 'static+Copy+Send>(
    name            : String,
    input_q_size    : usize,
    sink            : Box<Sink<InputType=Input>>) ->
    ( Box<Task>,
      Sender<Message<Input>> )
{
  // TODO ???? How to glue this with the sender ????
  let (input_tx, input_rx) = channel(input_q_size, Message::Empty);
  (
    Box::new(
      SinkWrap{
        name        : name,
        input_names  : vec![],
        output_names : vec![],
        sink        : sink,
        input_rx    : input_rx,
      }
    ),
    input_tx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
