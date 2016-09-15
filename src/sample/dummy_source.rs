
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Schedule, Message};

#[allow(dead_code)]
pub struct DummySource {}

impl source::Source for DummySource {
  type OutputType = usize;

  fn process(&mut self, _output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    Schedule::Loop
  }
}
