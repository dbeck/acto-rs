extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};

pub trait Sink {
  type InputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>) -> Schedule;
}

pub struct SinkWrap<Input: Copy+Send> {
  name        : String,
  sink        : Box<Sink<InputType=Input>>,
  input_rx    : Receiver<Message<Input>>,
}

impl<Input : Copy+Send> SinkWrap<Input> {
  pub fn process(&mut self) -> Schedule {
    self.sink.process(&mut self.input_rx)
  }
}

pub fn new<Input: Copy+Send>(
    name            : String,
    input_q_size    : usize,
    sink            : Box<Sink<InputType=Input>>) ->
    ( SinkWrap<Input>,
      Sender<Message<Input>> )
{
  // TODO ???? How to glue this with the sender ????
  let (input_tx, input_rx) = channel(input_q_size, Message::Empty);
  (
    SinkWrap{
      name        : name,
      sink        : sink,
      input_rx    : input_rx,
    },
    input_tx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
