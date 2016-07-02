extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Result};

pub trait YMerge {
  type InputTypeA   : Copy+Send;
  type InputTypeB   : Copy+Send;
  type OutputType   : Copy+Send;

  fn process(
    &mut self,
    input_a:  &mut Receiver<Message<Self::InputTypeA>>,
    input_b:  &mut Receiver<Message<Self::InputTypeB>>,
    output:   &mut Sender<Message<Self::OutputType>>) -> Result;
}

pub struct YMergeWrap<InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send> {
  name        : String,
  ymerge      : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>,
  input_a_rx  : Receiver<Message<InputA>>,
  input_b_rx  : Receiver<Message<InputB>>,
  output_tx   : Sender<Message<Output>>,
}

impl<InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send> YMergeWrap<InputA, InputB, Output> {
  pub fn process(&mut self) -> Result {
    self.ymerge.process(&mut self.input_a_rx, &mut self.input_b_rx, &mut self.output_tx)
  }
}

pub fn new<'a, InputA: Copy+Send, InputB: Copy+Send, Output: Copy+Send>(
    name             : String,
    input_a_q_size   : usize,
    input_b_q_size   : usize,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>>) ->
    ( YMergeWrap<InputA, InputB, Output>,
      Sender<Message<InputA>>,
      Sender<Message<InputB>>,
      Receiver<Message<Output>> )
{
  let (input_a_tx, input_a_rx)   = channel(input_a_q_size, Message::Empty);
  let (input_b_tx, input_b_rx)   = channel(input_b_q_size, Message::Empty);
  let (output_tx,  output_rx)    = channel(output_q_size, Message::Empty);
  (
    YMergeWrap{
      name        : name,
      ymerge      : ymerge,
      input_a_rx  : input_a_rx,
      input_b_rx  : input_b_rx,
      output_tx   : output_tx,
    },
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
