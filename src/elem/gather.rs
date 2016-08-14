use lossyq::spsc::{Sender, Receiver, channel};
use super::super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::super::connectable::{ConnectableN};

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

impl<Input: Send, Output: Send> ConnectableN for GatherWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self, n: usize) -> &mut Option<IdentifiedReceiver<Input>> {
    let ret_slice = self.input_rx_vec.as_mut_slice();
    &mut ret_slice[n]
  }
}

impl<Input: Send, Output: Send> Task for GatherWrap<Input,Output> {
  fn execute(&mut self, reporter: &mut Reporter) -> Schedule {
    // TODO : make this nicer. repetitive for all elems!
    let msg_id = self.output_tx.seqno();
    let retval = self.state.process(&mut self.input_rx_vec,
                                    &mut self.output_tx);
    let new_msg_id = self.output_tx.seqno();
    if msg_id != new_msg_id {
      reporter.message_sent(0, new_msg_id);
    }
    retval
  }
  fn name(&self) -> &String { &self.name }
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
          id:     new_id(String::from(name), Direction::Out, 0),
          input:  output_rx,
        }
      )
    )
  )
}
