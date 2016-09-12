use lossyq::spsc::*;
use super::super::{Message, Schedule, ChannelWrapper};
use super::{source, sink};
use super::connectable::{Connectable};

#[test]
fn connect_disconnect() {
  let (_source_task, mut source_out) = source::new( "Source", 20, Box::new(DummySource{}));
  let mut sink_task = sink::new( "Sink", Box::new(DummySink{}));

  assert!(sink_task.connect(&mut source_out).is_ok());
  assert!(sink_task.disconnect(&mut source_out).is_ok());
  assert!(sink_task.connect(&mut source_out).is_ok());
}

struct DummySource {}

impl source::Source for DummySource {
  type OutputType = usize;

  fn process(&mut self, _output: &mut Sender<Message<Self::OutputType>>) -> Schedule {
    Schedule::Loop
  }
}

struct DummySink {}

impl sink::Sink for DummySink {
  type InputType = usize;

  fn process(&mut self, _input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    Schedule::Loop
  }
}
