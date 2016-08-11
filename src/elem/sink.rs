use lossyq::spsc::Receiver;
use super::super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver};
use super::super::connectable::{Connectable};

pub trait Sink {
  type InputType : Send;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>) -> Schedule;
}

pub struct SinkWrap<Input: Send> {
  name      : String,
  state     : Box<Sink<InputType=Input>+Send>,
  input_rx  : Option<IdentifiedReceiver<Input>>,
}

impl<Input: Send> Connectable for SinkWrap<Input> {
  type Input = Input;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Input>> {
    &mut self.input_rx
  }
}

impl<Input: Send> Task for SinkWrap<Input> {
  fn execute(&mut self, _reporter: &mut Reporter) -> Schedule {
    match &mut self.input_rx {
      &mut Some(ref mut identified) => {
        self.state.process(&mut identified.input)
      },
      &mut None => Schedule::EndPlusUSec(10_000)
    }
  }
  fn name(&self) -> &String { &self.name }
}

pub fn new<Input: Send>(
    name   : &str,
    sink   : Box<Sink<InputType=Input>+Send>)
      -> Box<SinkWrap<Input>>
{
  Box::new(
    SinkWrap{
      name          : String::from(name),
      state         : sink,
      input_rx      : None,
      }
    )
}
