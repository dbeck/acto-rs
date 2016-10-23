
use super::super::elem::sink;
use super::super::{ChannelWrapper};

pub struct DummySink {}

impl sink::Sink for DummySink {
  type InputValue = usize;
  type InputError = &'static str;

  fn process(&mut self,
             _input: &mut ChannelWrapper<Self::InputValue, Self::InputError>,
             _stop: &mut bool)
  {
  }
}
