extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Result};

pub trait YSplit {
  type InputType    : Copy+Send;
  type OutputTypeA  : Copy+Send;
  type OutputTypeB  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output_a:  &mut Sender<Message<Self::OutputTypeA>>,
    output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> Result;
}

pub struct YSplitWrap<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send> {
  name          : String,
  ysplit        : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>,
  input_rx      : Receiver<Message<Input>>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
}

impl<Input : Copy+Send, OutputA : Copy+Send, OutputB : Copy+Send> YSplitWrap<Input, OutputA, OutputB> {
  pub fn process(&mut self) -> Result {
    self.ysplit.process(&mut self.input_rx, &mut self.output_a_tx, &mut self.output_b_tx)
  }
}

pub fn new<'a, Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send>(
    name              : String,
    input_q_size      : usize,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>) ->
    ( YSplitWrap<Input, OutputA, OutputB>,
      Sender<Message<Input>>,
      Receiver<Message<OutputA>>,
      Receiver<Message<OutputB>> )
{
  let (input_tx, input_rx) = channel(input_q_size, Message::Empty);
  let (output_a_tx, output_a_rx) = channel(output_a_q_size, Message::Empty);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size, Message::Empty);
  (
    YSplitWrap{
      name          : name,
      ysplit        : ysplit,
      input_rx      : input_rx,
      output_a_tx   : output_a_tx,
      output_b_tx   : output_b_tx,
    },
    input_tx,
    output_a_rx,
    output_b_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
