use lossyq::spsc::{Sender, channel};
use super::super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver, new_id};
use super::super::connectable::{ConnectableY};

pub trait YMerge {
  type InputTypeA   : Send;
  type InputTypeB   : Send;
  type OutputType   : Send;

  fn process(
    &mut self,
    input_a:  &mut Option<IdentifiedReceiver<Self::InputTypeA>>,
    input_b:  &mut Option<IdentifiedReceiver<Self::InputTypeB>>,
    output:   &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct YMergeWrap<InputA: Send, InputB: Send, Output: Send> {
  name         : String,
  state        : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>+Send>,
  input_a_rx   : Option<IdentifiedReceiver<InputA>>,
  input_b_rx   : Option<IdentifiedReceiver<InputB>>,
  output_tx    : Sender<Message<Output>>,
}

impl<InputA: Send, InputB: Send, Output: Send> ConnectableY for YMergeWrap<InputA, InputB, Output> {
  type InputA = InputA;
  type InputB = InputB;

  fn input_a(&mut self) -> &mut Option<IdentifiedReceiver<InputA>> {
    &mut self.input_a_rx
  }

  fn input_b(&mut self) -> &mut Option<IdentifiedReceiver<InputB>> {
    &mut self.input_b_rx
  }
}

impl<InputA: Send, InputB: Send, Output: Send> Task for YMergeWrap<InputA, InputB, Output> {
  fn execute(&mut self, reporter: &mut Reporter) -> Schedule {
    // TODO : make this nicer. repetitive for all elems!
    let msg_id = self.output_tx.seqno();
    let retval = self.state.process(&mut self.input_a_rx,
                                    &mut self.input_b_rx,
                                    &mut self.output_tx);
    let new_msg_id = self.output_tx.seqno();
    if msg_id != new_msg_id {
      reporter.message_sent(0, new_msg_id);
    }
    retval
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 2 }
  fn output_count(&self) -> usize { 1 }
}

pub fn new<InputA: Send, InputB: Send, Output: Send>(
    name             : &str,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputTypeA=InputA, InputTypeB=InputB, OutputType=Output>+Send>)
      -> (Box<YMergeWrap<InputA,InputB,Output>>, Box<Option<IdentifiedReceiver<Output>>>)
{
  let (output_tx,  output_rx) = channel(output_q_size);

  (
    Box::new(
      YMergeWrap{
        name          : String::from(name),
        state         : ymerge,
        input_a_rx    : None,
        input_b_rx    : None,
        output_tx     : output_tx,
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
