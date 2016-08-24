use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, IdentifiedReceiver, new_id, ChannelId};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::output_counter::{OutputCounter};

pub trait YSplit {
  type InputType    : Send;
  type OutputTypeA  : Send;
  type OutputTypeB  : Send;

  fn process(
    &mut self,
    input:     &mut Option<IdentifiedReceiver<Self::InputType>>,
    output_a:  &mut Sender<Message<Self::OutputTypeA>>,
    output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> Schedule;
}

pub struct YSplitWrap<Input: Send, OutputA: Send, OutputB: Send> {
  name          : String,
  state         : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>,
  input_rx      : Option<IdentifiedReceiver<Input>>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
}

impl<Input: Send, OutputA: Send, OutputB: Send> IdentifiedInput for YSplitWrap<Input, OutputA, OutputB> {
  fn get_input_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id != 0 {
      None
    } else {
      match &self.input_rx {
        &Some(ref ch) => Some(ch.id.clone()),
        _             => None,
      }
    }
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> OutputCounter for  YSplitWrap<Input, OutputA, OutputB> {
  fn get_tx_count(&self, ch_id: usize) -> usize {
    if ch_id == 0 {
      self.output_a_tx.seqno()
    } else if ch_id == 1 {
      self.output_b_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> Connectable for YSplitWrap<Input, OutputA, OutputB> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> Task for YSplitWrap<Input, OutputA, OutputB> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx,
                       &mut self.output_a_tx,
                       &mut self.output_b_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 2 }
  fn input_id(&self, ch_id: usize) -> Option<ChannelId> {
    self.get_input_id(ch_id)
  }
  fn tx_count(&self, ch_id: usize) -> usize {
    self.get_tx_count(ch_id)
  }
}

pub fn new<Input: Send, OutputA: Send, OutputB: Send>(
    name              : &str,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>)
      -> (Box<YSplitWrap<Input,OutputA,OutputB>>,
          Box<Option<IdentifiedReceiver<OutputA>>>,
          Box<Option<IdentifiedReceiver<OutputB>>>)
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size);

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
          id:     new_id(String::from(name), 0),
          input:  output_a_rx,
        }
      )
    ),
    Box::new(
      Some(
        IdentifiedReceiver{
          id:     new_id(String::from(name), 1),
          input:  output_b_rx,
        }
      )
    ),
  )
}
