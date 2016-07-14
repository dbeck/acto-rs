extern crate lossyq;
use self::lossyq::spsc::Receiver;
use super::common::{Message, Schedule};
use super::identified_receiver;
use super::task::{Task};

pub trait Sink {
  type InputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>) -> Schedule;
}

struct SinkWrap<Input: Copy+Send> {
  name      : String,
  sink      : Box<Sink<InputType=Input>>,
  input_rx  : Option<identified_receiver::IdentifiedReceiver<Input>>,
}

impl<Input: Copy+Send> Task for SinkWrap<Input> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.sink.process(&mut identified.input)
      },
      _ => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: 'static+Copy+Send>(
    name   : String,
    sink   : Box<Sink<InputType=Input>>) -> Box<Task>
{
  Box::new(
    SinkWrap{
      name          : name.clone(),
      sink          : sink,
      input_rx      : None,
      }
    )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
