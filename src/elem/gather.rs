extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::super::common::{Task, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::super::connectable::{ConnectableN};

pub trait Gather {
  type InputType   : Copy+Send;
  type OutputType  : Copy+Send;

  fn process(
    &mut self,
    input:   Vec<&mut Receiver<Message<Self::InputType>>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct GatherWrap<Input: Copy+Send, Output: Copy+Send> {
  name          : String,
  state         : Box<Gather<InputType=Input,OutputType=Output>+Send>,
  input_rx_vec  : Vec<Option<IdentifiedReceiver<Input>>>,
  output_tx     : Sender<Message<Output>>,
}

impl<Input: Copy+Send, Output: Copy+Send> ConnectableN for GatherWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self, n: usize) -> &mut Option<IdentifiedReceiver<Input>> {
    let ret_slice = self.input_rx_vec.as_mut_slice();
    &mut ret_slice[n]
  }
}

impl<Input: Copy+Send, Output: Copy+Send> Task for GatherWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    let mut input_vec = vec![];
    for ch in &mut self.input_rx_vec {
      match ch {
        &mut Some(ref mut identified) => {
          input_vec.push(&mut identified.input);
        },
        &mut None => {}
      }
    }
    if input_vec.len() == 0 {
      Schedule::EndPlusUSec(10_000)
    } else {
      self.state.process(input_vec, &mut self.output_tx)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: Copy+Send, Output: Copy+Send>(
    name            : &str,
    output_q_size   : usize,
    gather          : Box<Gather<InputType=Input,OutputType=Output>+Send>,
    n_channels      : usize)
      -> (Box<GatherWrap<Input,Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size, Message::Empty);
  let mut inputs = vec![];
  for _i in 0..n_channels { inputs.push(None); }

  (
    Box::new(
      GatherWrap{
        name          : String::from(name),
        state         : gather,
        input_rx_vec  : inputs,
        output_tx     : output_tx,
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
