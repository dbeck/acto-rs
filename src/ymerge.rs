extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::identified_receiver::{IdentifiedReceiver};
use super::task::{Task};
use super::channel_id;

pub trait YMerge {
  type InputTypeA   : Copy+Send;
  type InputTypeB   : Copy+Send;
  type OutputType   : Copy+Send;

  fn process(
    &mut self,
    input_a:  &mut Receiver<Message<Self::InputTypeA>>,
    input_b:  &mut Receiver<Message<Self::InputTypeB>>,
    output:   &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct YMergeWrap<InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send> {
  name         : String,
  ymerge       : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>,
  input_a_rx   : Option<IdentifiedReceiver<InputA>>,
  input_b_rx   : Option<IdentifiedReceiver<InputB>>,
  output_tx    : Sender<Message<Output>>,
  output_rx    : Option<IdentifiedReceiver<Output>>,
}

impl<InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send> Task for YMergeWrap<InputA, InputB, Output> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_a_rx {
      &mut Some(ref mut identified_a) => {
        match &mut self.input_b_rx {
          &mut Some(ref mut identified_b) => {
            self.ymerge.process(
              &mut identified_a.input,
              &mut identified_b.input,
              &mut self.output_tx
            )
          },
          _ => Schedule::EndPlusUSec(10_000)
        }
      },
      _ => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<InputA: 'static+Copy+Send, InputB: 'static+Copy+Send, Output: 'static+Copy+Send>(
    name             : String,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>) -> Box<Task>
{
  let (output_tx,  output_rx)    = channel(output_q_size, Message::Empty);
  
  Box::new(
    YMergeWrap{
      name          : name.clone(),
      ymerge        : ymerge,
      input_a_rx    : None,
      input_b_rx    : None,
      output_tx     : output_tx,
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
