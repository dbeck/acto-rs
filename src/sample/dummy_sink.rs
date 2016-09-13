
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule};

#[allow(dead_code)]
struct DummySink {}

impl sink::Sink for DummySink {
  type InputType = usize;

  fn process(&mut self, _input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    Schedule::Loop
  }
}
