extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Message, Schedule};
use super::identified_receiver::{IdentifiedReceiver};
use super::task::{Task};
use super::channel_id;

pub trait YSplit {
  type InputType    : Copy+Send;
  type OutputTypeA  : Copy+Send;
  type OutputTypeB  : Copy+Send;

  fn process(
    &mut self,
    input:     &mut Receiver<Message<Self::InputType>>,
    output_a:  &mut Sender<Message<Self::OutputTypeA>>,
    output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> Schedule;
}

pub struct YSplitWrap<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send> {
  name          : String,
  ysplit        : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>,
  input_rx      : Option<IdentifiedReceiver<Input>>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
  output_a_rx   : Option<IdentifiedReceiver<OutputA>>,
  output_b_rx   : Option<IdentifiedReceiver<OutputB>>,
}

impl<Input : Copy+Send, OutputA : Copy+Send, OutputB : Copy+Send> Task for YSplitWrap<Input, OutputA, OutputB> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.ysplit.process(
          &mut identified.input,
          &mut self.output_a_tx,
          &mut self.output_b_tx
        )
      },
      _ => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: 'static+Copy+Send, OutputA: 'static+Copy+Send, OutputB: 'static+Copy+Send>(
    name              : String,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>>) -> Box<Task>
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size, Message::Empty);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size, Message::Empty);

  Box::new(
    YSplitWrap{
      name          : name.clone(),
      ysplit        : ysplit,
      input_rx      : None,
      output_a_tx   : output_a_tx,
      output_b_tx   : output_b_tx,
      output_a_rx   : Some(
        IdentifiedReceiver{
          id:     channel_id::new(name.clone(), channel_id::Direction::Out, 0),
          input:  output_a_rx,
        }
      ),
      output_b_rx   : Some(
        IdentifiedReceiver{
          id:     channel_id::new(name.clone(), channel_id::Direction::Out, 1),
          input:  output_b_rx,
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
