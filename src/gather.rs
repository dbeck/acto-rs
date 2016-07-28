extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::task::{Task};
use super::connectable::{ConnectableN};

pub trait Gather {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Vec<Receiver<Message<Self::InputType>>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct GatherWrap<Input: Copy+Send, Output: Copy+Send> {
  name         : String,
  state        : Box<Gather<InputType=Input,OutputType=Output>+Send>,
  input_rx     : Vec<Option<IdentifiedReceiver<Input>>>,
  output_tx    : Sender<Message<Output>>,
}

impl<Input: Copy+Send, Output: Copy+Send> ConnectableN for GatherWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self, n: usize) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Copy+Send, Output: Copy+Send> Task for GatherWrap<Input,Output> {
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

pub fn new<Input: Copy+Send, Output: Copy+Send>(
    name            : &str,
    output_q_size   : usize,
    gather          : Box<Gather<InputType=Input,OutputType=Output>+Send>)
      -> (Box<GatherWrap<Input,Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);

  (
    Box::new(
      GatherWrap{
        name        : String::from(name),
        state       : gather,
        input_rx    : None,
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
