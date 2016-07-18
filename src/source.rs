extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::task::{Task};
use super::identified_receiver::{IdentifiedReceiver};
use super::channel_id;

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

impl<Output: Copy+Send> Task for SourceWrap<Output> {
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
            id:     channel_id::new(String::from(name), channel_id::Direction::Out, 0),
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
