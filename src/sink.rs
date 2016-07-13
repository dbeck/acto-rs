extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::channel_id::{Id, Direction};
use super::task::{Task};
use super::channel_id;

pub trait Sink {
  type InputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>) -> Schedule;
}

struct SinkWrap<Input: Copy+Send> {
  name         : String,
  input_names  : Vec<Id>,
  output_names : Vec<Id>,
  sink         : Box<Sink<InputType=Input>>,
  input_rx     : Receiver<Message<Input>>,
}

impl<Input: Copy+Send> Task for SinkWrap<Input> {
  fn execute(&mut self) -> Schedule {
    self.sink.process(&mut self.input_rx)
  }
  fn name(&self)         -> &String    { &self.name }
  fn input_names(&self)  -> &Vec<Id>   { &self.input_names }
  fn output_names(&self) -> &Vec<Id>   { &self.output_names }
}

pub fn new<Input: 'static+Copy+Send>(
    name            : String,
    input_q_size    : usize,
    sink            : Box<Sink<InputType=Input>>) ->
    ( Box<Task>,
      Sender<Message<Input>> )
{
  let (input_tx, input_rx) = channel(input_q_size, Message::Empty);
  (
    Box::new(
      SinkWrap{
        name          : name.clone(),
        input_names   : vec![channel_id::new(name.clone(), Direction::In, 0),],
        output_names  : vec![],
        sink          : sink,
        input_rx      : input_rx,
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
