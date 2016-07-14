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

struct SourceWrap<Output: Copy+Send> {
  name       : String,
  source     : Box<Source<OutputType=Output>>,
  output_tx  : Sender<Message<Output>>,
  output_rx  : Option<IdentifiedReceiver<Output>>,
}

impl<Output: Copy+Send> Task for SourceWrap<Output> {
  fn execute(&mut self) -> Schedule {
    self.source.process(&mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Output: 'static+Copy+Send>(
    name            : String,
    output_q_size   : usize,
    source          : Box<Source<OutputType=Output>>) -> Box<Task>
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  
  Box::new(
    SourceWrap{
      name        : name.clone(),
      source      : source,
      output_tx   : output_tx,
      output_rx   : Some(
        IdentifiedReceiver{
          id:     channel_id::new(name.clone(), channel_id::Direction::Out, 0),
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
