extern crate lossyq;
use self::lossyq::spsc::{Sender, channel};
use super::common::{Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::task::{Task};

pub trait Source {
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct SourceWrap<Output: Copy+Send> {
  name       : String,
  source     : Box<Source<OutputType=Output>>,
  output_tx  : Sender<Message<Output>>,
}

impl<Output: 'static+Copy+Send> Task for SourceWrap<Output> {
  fn execute(&mut self) -> Schedule {
    self.source.process(&mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Output: Copy+Send>(
    name            : &str,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>>)
      -> (Box<SourceWrap<Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);

  (
    Box::new(
      SourceWrap{
        name        : String::from(name),
        source      : source,
        output_tx   : output_tx,
      }
    ),
    Box::new(
      Some(
          IdentifiedReceiver{
            id:     new_id(String::from(name), Direction::Out, 0),
            input:  output_rx,
          }
        )
    )
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
