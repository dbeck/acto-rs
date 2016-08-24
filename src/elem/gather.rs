use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, IdentifiedReceiver, new_id, ChannelId};
use super::connectable::{ConnectableN};
use super::identified_input::{IdentifiedInput};
use super::output_counter::{OutputCounter};

pub trait Gather {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut Vec<Option<IdentifiedReceiver<Self::InputType>>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct GatherWrap<Input: Send, Output: Send> {
  name           : String,
  state          : Box<Gather<InputType=Input,OutputType=Output>+Send>,
  input_rx_vec   : Vec<Option<IdentifiedReceiver<Input>>>,
  output_tx      : Sender<Message<Output>>,
}

impl<Input: Send, Output: Send> IdentifiedInput for GatherWrap<Input,Output> {
  fn get_input_id(&self, ch_id: usize) -> Option<ChannelId> {
    if ch_id < self.input_rx_vec.len() {
      let slice = self.input_rx_vec.as_slice();
      match &slice[ch_id] {
        &Some(ref ch) => Some(ch.id.clone()),
        _             => None,
      }
    } else {
      None
    }
  }
}

impl<Input: Send, Output: Send> OutputCounter for GatherWrap<Input,Output> {
  fn get_tx_count(&self, ch_id: usize) -> usize {
    if ch_id == 0 {
      self.output_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, Output: Send> ConnectableN for GatherWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self, n: usize) -> &mut Option<IdentifiedReceiver<Input>> {
    let ret_slice = self.input_rx_vec.as_mut_slice();
    &mut ret_slice[n]
  }
}

impl<Input: Send, Output: Send> Task for GatherWrap<Input,Output> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx_vec, &mut self.output_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { self.input_rx_vec.len() }
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
    gather          : Box<Gather<InputType=Input,OutputType=Output>+Send>,
    n_channels      : usize)
      -> (Box<GatherWrap<Input,Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let mut inputs = vec![];
  for _i in 0..n_channels { inputs.push(None); }

  (
    Box::new(
      GatherWrap{
        name                   : String::from(name),
        state                  : gather,
        input_rx_vec           : inputs,
        output_tx              : output_tx,
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
