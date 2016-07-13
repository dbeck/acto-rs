extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::channel_id::{Id, Direction};
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
  input_names  : Vec<Id>,
  output_names : Vec<Id>,
  ymerge       : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>,
  input_a_rx   : Receiver<Message<InputA>>,
  input_b_rx   : Receiver<Message<InputB>>,
  output_tx    : Sender<Message<Output>>,
}

impl<InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send> Task for YMergeWrap<InputA, InputB, Output> {
  fn execute(&mut self) -> Schedule {
    self.ymerge.process(
      &mut self.input_a_rx,
      &mut self.input_b_rx,
      &mut self.output_tx
    )
  }
  fn name(&self)         -> &String   { &self.name }
  fn input_names(&self)  -> &Vec<Id>  { &self.input_names }
  fn output_names(&self) -> &Vec<Id>  { &self.output_names }
}

pub fn new<InputA: 'static+Copy+Send, InputB: 'static+Copy+Send, Output: 'static+Copy+Send>(
    name             : String,
    input_a_q_size   : usize,
    input_b_q_size   : usize,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>) ->
    ( Box<Task>,
      Sender<Message<InputA>>,
      Sender<Message<InputB>>,
      Receiver<Message<Output>> )
{
  let (input_a_tx, input_a_rx)   = channel(input_a_q_size, Message::Empty);
  let (input_b_tx, input_b_rx)   = channel(input_b_q_size, Message::Empty);
  let (output_tx,  output_rx)    = channel(output_q_size, Message::Empty);
  (
    Box::new(
      YMergeWrap{
        name          : name.clone(),
        input_names   : vec![
          channel_id::new(name.clone(), Direction::In, 0),
          channel_id::new(name.clone(), Direction::In, 1),],
        output_names  : vec![channel_id::new(name.clone(), Direction::Out, 0),],
        ymerge        : ymerge,
        input_a_rx    : input_a_rx,
        input_b_rx    : input_b_rx,
        output_tx     : output_tx,
      }
    ),
    input_a_tx,
    input_b_tx,
    output_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
