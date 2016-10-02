use super::{source, sink};
use super::connectable::{Connectable};
use super::super::sample::{dummy_source, dummy_sink};

#[test]
fn connect_disconnect() {
  let (_source_task, mut source_out) = source::new( "Source", 20, Box::new(dummy_source::DummySource{}));
  let mut sink_task = sink::new( "Sink", Box::new(dummy_sink::DummySink{}));

  assert!(sink_task.connect(&mut source_out).is_ok());
  assert!(sink_task.disconnect(&mut source_out).is_ok());
  assert!(sink_task.connect(&mut source_out).is_ok());
}
