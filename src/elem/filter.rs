use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, IdentifiedReceiver, new_id, ChannelId};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::output_counter::{OutputCounter};

pub trait Filter {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct FilterWrap<Input: Send, Output: Send> {
  name         : String,
  state        : Box<Filter<InputType=Input,OutputType=Output>+Send>,
  input_rx     : Option<IdentifiedReceiver<Input>>,
  output_tx    : Sender<Message<Output>>,
}

impl<Input: Send, Output: Send> IdentifiedInput for FilterWrap<Input,Output> {
  fn get_input_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id == 0 {
      match &self.input_rx {
        &Some(ref ch)  => Some(ch.id.clone()),
        _  => None,
      }
    } else {
      None
    }
  }
}

impl<Input: Send, Output: Send> OutputCounter for FilterWrap<Input,Output> {
  fn get_tx_count(&self, ch_id: usize) -> usize {
    if ch_id == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> Connectable for FilterWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Send, Output: Send> Task for FilterWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx, &mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 1 }

  fn input_id(&self, ch_id: usize) -> Option<ChannelId> {
    self.get_input_id(ch_id)
  }
  fn tx_count(&self, ch_id: usize) -> usize {
    self.get_tx_count(ch_id)
  }
}

pub fn new<Input: Send, Output: Send>(
    name            : &str,
    output_q_size   : usize,
    filter          : Box<Filter<InputType=Input,OutputType=Output>+Send>)
      -> (Box<FilterWrap<Input,Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size);

  (
    Box::new(
      FilterWrap{
        name        : String::from(name),
        state       : filter,
        input_rx    : None,
        output_tx   : output_tx,
      }
    ),
    Box::new(
      Some(
        IdentifiedReceiver{
          id:     new_id(String::from(name), 0),
          input:  output_rx,
        }
      )
    )
  )
}
