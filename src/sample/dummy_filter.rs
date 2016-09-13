
use lossyq::spsc::Sender;
use super::super::elem::filter;
use super::super::{ChannelWrapper, Message, Schedule};

#[allow(dead_code)]
struct DummyFilter {}

impl filter::Filter for DummyFilter {
  type InputType = usize;
  type OutputType = usize;

  fn process(
    &mut self,
    _input:   &mut ChannelWrapper<Self::InputType>,
    _output:  &mut Sender<Message<Self::OutputType>>) -> Schedule
  {
    Schedule::Loop
  }
}
