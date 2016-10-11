
use super::super::elem::sink;
use super::super::{ChannelWrapper};

pub struct DummySink {}

impl sink::Sink for DummySink {
  type InputType = usize;

  fn process(&mut self,
             _input: &mut ChannelWrapper<Self::InputType>,
             _stop: &mut bool)
  {
  }
}
