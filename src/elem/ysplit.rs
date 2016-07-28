extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::super::common::{Task, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::super::connectable::{Connectable};

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
  state         : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>,
  input_rx      : Option<IdentifiedReceiver<Input>>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
}

impl<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send> Connectable for YSplitWrap<Input, OutputA, OutputB> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send> Task for YSplitWrap<Input, OutputA, OutputB> {
  fn execute(&mut self) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.state.process(
          &mut identified.input,
          &mut self.output_a_tx,
          &mut self.output_b_tx
        )
      },
      &mut None => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: Copy+Send, OutputA: Copy+Send, OutputB: Copy+Send>(
    name              : &str,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>)
      -> (Box<YSplitWrap<Input,OutputA,OutputB>>,
          Box<Option<IdentifiedReceiver<OutputA>>>,
          Box<Option<IdentifiedReceiver<OutputB>>>)
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size, Message::Empty);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size, Message::Empty);

  (
    Box::new(
      YSplitWrap{
        name          : String::from(name),
        state         : ysplit,
        input_rx      : None,
        output_a_tx   : output_a_tx,
        output_b_tx   : output_b_tx,
      }
    ),
    Box::new(
      Some(
        IdentifiedReceiver{
          id:     new_id(String::from(name), Direction::Out, 0),
          input:  output_a_rx,
        }
      )
    ),
    Box::new(
      Some(
        IdentifiedReceiver{
          id:     new_id(String::from(name), Direction::Out, 1),
          input:  output_b_rx,
        }
      )
    ),
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
