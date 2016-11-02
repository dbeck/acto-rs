
use super::super::Scheduler;
use super::dummy_source::DummySource;
use super::dummy_sink::DummySink;
use super::super::elem::{source, sink};
use super::super::elem::connectable::{Connectable};
use super::super::{SchedulingRule};

#[allow(dead_code)]
fn setup_pipeline() {

  let mut sched = Scheduler::new();
  sched.start_with_threads(4);

  let dummy_queue_size = 2_000;
  let (source_task, mut source_out) =
    source::new( "Source", dummy_queue_size, Box::new(DummySource{}));

  let mut sink_task =
    sink::new( "Sink", Box::new(DummySink{}));

  sink_task.connect(&mut source_out).unwrap();

  let source_id = sched.add_task(source_task, SchedulingRule::OnExternalEvent).unwrap();
  sched.add_task(sink_task, SchedulingRule::OnMessage).unwrap();

  sched.notify(&source_id).unwrap();

  sched.stop();
}
