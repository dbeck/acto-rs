
use lossyq::spsc::Sender;
use super::super::elem::filter;
use super::super::{ChannelWrapper, Message};

#[allow(dead_code)]
struct DummyFilter {}

impl filter::Filter for DummyFilter {
  type InputValue   = usize;
  type InputError   = &'static str;
  type OutputValue  = usize;
  type OutputError  = &'static str;

  fn process(
    &mut self,
    _input:   &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    _output:  &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    _stop:    &mut bool)
  {
  }
}
