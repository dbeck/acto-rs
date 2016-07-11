extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::task::Task;

pub trait YSplit {
  type InputType    : Copy+Send;
  type OutputTypeA  : Copy+Send;
  type OutputTypeB  : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output_a:  &mut Sender<Message<Self::OutputTypeA>>,
    output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> Schedule;
}

pub struct YSplitWrap<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send> {
  name          : String,
  input_names   : Vec<String>,
  output_names  : Vec<String>,
  ysplit        : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>,
  input_rx      : Receiver<Message<Input>>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
}

impl<Input : Copy+Send, OutputA : Copy+Send, OutputB : Copy+Send> Task for YSplitWrap<Input, OutputA, OutputB> {
  fn execute(&mut self) -> Schedule {
    self.ysplit.process(
      &mut self.input_rx,
      &mut self.output_a_tx,
      &mut self.output_b_tx
    )
  }
  fn name(&self)         -> &String      { &self.name }
  fn input_names(&self)  -> &Vec<String> { &self.input_names }
  fn output_names(&self) -> &Vec<String> { &self.output_names }
}

pub fn new<Input: 'static+Copy+Send, OutputA: 'static+Copy+Send, OutputB: 'static+Copy+Send>(
    name              : String,
    input_q_size      : usize,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>) ->
    ( Box<Task>,
      Sender<Message<Input>>,
      Receiver<Message<OutputA>>,
      Receiver<Message<OutputB>> )
{
  let (input_tx, input_rx) = channel(input_q_size, Message::Empty);
  let (output_a_tx, output_a_rx) = channel(output_a_q_size, Message::Empty);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size, Message::Empty);
  (
    Box::new(
      YSplitWrap{
        name          : name,
        input_names   : vec![],
        output_names  : vec![],
        ysplit        : ysplit,
        input_rx      : input_rx,
        output_a_tx   : output_a_tx,
        output_b_tx   : output_b_tx,
      }
    ),
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
