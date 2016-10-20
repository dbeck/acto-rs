
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Message};

pub struct DummySource {}

impl source::Source for DummySource {
  type OutputType = usize;

  fn process(&mut self,
             _output: &mut Sender<Message<Self::OutputType>>,
             _stop: &mut bool)
  {
  }
}

impl DummySource {
  pub fn new() -> DummySource {
    DummySource{}
  }
}
