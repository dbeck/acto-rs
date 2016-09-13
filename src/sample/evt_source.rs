
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Schedule, Message};
use super::super::scheduler::event;

#[allow(dead_code)]
pub struct EvtSource {
  on_exec: event::Event,
}

impl source::Source for EvtSource {
  type OutputType = usize;

  fn process(&mut self, output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    output.put(|v| *v = Some(Message::Value(0)));
    self.on_exec.notify();
    Schedule::Loop
  }
}

pub fn new(on_exec: event::Event) -> EvtSource {
  EvtSource{
    on_exec: on_exec,
  }
}
