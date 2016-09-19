
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Schedule, Message};
use super::super::scheduler::event;
use super::tick::{Tick};

pub struct MeasuredPipelineSource {
  on_exec:  event::Event,
}

impl source::Source for MeasuredPipelineSource {
  type OutputType = Tick;

  fn process(&mut self, output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    output.put(|v| *v = Some(Message::Value(Tick::new())));
    self.on_exec.notify();
    Schedule::OnExternalEvent
  }
}

impl MeasuredPipelineSource {
  pub fn new(on_exec: event::Event) -> MeasuredPipelineSource {
    MeasuredPipelineSource{
      on_exec: on_exec,
    }
  }
}

pub fn new(on_exec: event::Event) -> MeasuredPipelineSource {
  MeasuredPipelineSource::new(on_exec)
}

impl Drop for MeasuredPipelineSource {
  fn drop(&mut self) {
    let (_r, exec_count) = self.on_exec.ready(0);
    println!(" @drop MeasuredPipelineSource exec_count:{}",exec_count);
  }
}
