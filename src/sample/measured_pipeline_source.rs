
use lossyq::spsc::Sender;
use super::super::elem::source;
use super::super::{Schedule, Message};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

pub struct MeasuredPipelineSource {
  on_exec:  u64,
  spinned:  Arc<AtomicUsize>,
}

impl source::Source for MeasuredPipelineSource {
  type OutputType = usize;

  fn process(&mut self, output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    self.on_exec += 1;
    output.put(|v| *v = Some(Message::Value(self.spinned.load(Ordering::Acquire))));
    Schedule::OnExternalEvent
  }
}

impl MeasuredPipelineSource {
  pub fn new(spinned: Arc<AtomicUsize>) -> MeasuredPipelineSource {
    MeasuredPipelineSource{
      on_exec: 0,
      spinned: spinned,
    }
  }
}

pub fn new(spinned: Arc<AtomicUsize>) -> MeasuredPipelineSource {
  MeasuredPipelineSource::new(spinned)
}

impl Drop for MeasuredPipelineSource {
  fn drop(&mut self) {
    println!(" @drop MeasuredPipelineSource exec_count:{} spinned:{}",
      self.on_exec, self.spinned.load(Ordering::Acquire));
  }
}
