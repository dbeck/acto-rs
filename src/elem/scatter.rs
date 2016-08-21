use lossyq::spsc::{Sender, channel};
use super::super::{Task, Reporter, Message, Schedule, IdentifiedReceiver, new_id};
use super::connectable::{Connectable};

pub trait Scatter {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
    output:  &mut Vec<Sender<Message<Self::OutputType>>>) -> Schedule;
}

pub struct ScatterWrap<Input: Send, Output: Send> {
  name           : String,
  state          : Box<Scatter<InputType=Input,OutputType=Output>+Send>,
  input_rx       : Option<IdentifiedReceiver<Input>>,
  output_tx_vec  : Vec<Sender<Message<Output>>>,
  msg_ids        : Vec<usize>,
}

impl<Input: Send, Output: Send> Connectable for ScatterWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Send, Output: Send> Task for ScatterWrap<Input,Output> {
  fn execute(&mut self, reporter: &mut Reporter) -> Schedule {
    // TODO : make this nicer. repetitive for all elems!
    let retval = self.state.process(&mut self.input_rx,
                                    &mut self.output_tx_vec);
    let otx_slice = self.output_tx_vec.as_slice();
    let ln = self.msg_ids.len();
    let ids_slice = self.msg_ids.as_mut_slice();
    for i in 0..ln {
      let new_msg_id = otx_slice[i].seqno();
      if ids_slice[i] != new_msg_id {
        reporter.message_sent(i, new_msg_id);
      }
      ids_slice[i] = new_msg_id;
    }
    retval
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { self.output_tx_vec.len() }
}

pub fn new<Input: Send, Output: Send>(
    name            : &str,
    output_q_size   : usize,
    scatter         : Box<Scatter<InputType=Input,OutputType=Output>+Send>,
    n_channels      : usize)
      -> (Box<ScatterWrap<Input,Output>>, Vec<Box<Option<IdentifiedReceiver<Output>>>>)
{
  let mut tx_vec = Vec::with_capacity(n_channels);
  let mut rx_vec = Vec::with_capacity(n_channels);

  for i in 0..n_channels {
    let (output_tx, output_rx) = channel(output_q_size);
    tx_vec.push(output_tx);
    rx_vec.push(
      Box::new(
        Some(
          IdentifiedReceiver{
            id:     new_id(String::from(name), i),
            input:  output_rx,
          }
        )
      )
    );
  }

  (
    Box::new(
      ScatterWrap{
        name           : String::from(name),
        state          : scatter,
        input_rx       : None,
        output_tx_vec  : tx_vec,
        msg_ids        : vec![n_channels;0],
      }
    ),
    rx_vec
  )
}
