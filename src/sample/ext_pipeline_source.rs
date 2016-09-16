
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Schedule, Message};
use super::super::scheduler::event;

#[allow(dead_code)]
pub struct ExtPipelineSource {
  on_exec: event::Event,
}

impl source::Source for ExtPipelineSource {
  type OutputType = usize;

  fn process(&mut self, output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    output.put(|v| *v = Some(Message::Value(0)));
    self.on_exec.notify();
    Schedule::OnExternalEvent
  }
}

impl ExtPipelineSource {
  pub fn new(on_exec: event::Event) -> ExtPipelineSource {
    ExtPipelineSource{
      on_exec: on_exec,
    }
  }
}

pub fn new(on_exec: event::Event) -> ExtPipelineSource {
  ExtPipelineSource::new(on_exec)
}
