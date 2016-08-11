use lossyq::spsc::{Sender, Receiver, channel};
use super::super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::super::connectable::{Connectable};

pub trait Filter {
  type InputType   : Send;
  type OutputType  : Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Message<Self::InputType>>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule;
}

pub struct FilterWrap<Input: Send, Output: Send> {
  name         : String,
  state        : Box<Filter<InputType=Input,OutputType=Output>+Send>,
  input_rx     : Option<IdentifiedReceiver<Input>>,
  output_tx    : Sender<Message<Output>>,
}

impl<Input: Send, Output: Send> Connectable for FilterWrap<Input,Output> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Send, Output: Send> Task for FilterWrap<Input,Output> {
  fn execute(&mut self, reporter: &mut Reporter) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        // TODO : make this nicer. repetitive for all elems!
        let msg_id = self.output_tx.seqno();
        let retval = self.state.process(&mut identified.input,
                                        &mut self.output_tx);
        let new_msg_id = self.output_tx.seqno();
        if msg_id != new_msg_id {
          reporter.message_sent(0, new_msg_id);
        }
        retval
      },
      &mut None => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
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
          id:     new_id(String::from(name), Direction::Out, 0),
          input:  output_rx,
        }
      )
    )
  )
}
