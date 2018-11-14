extern crate acto_rs as actors;

#[cfg(feature = "experiment")]
fn main() {
  use actors::{Scheduler, SchedulingRule};
  use actors::sample::dummy_source::DummySource;
  use actors::sample::dummy_sink::DummySink;
  use actors::elem::{source, sink};
  use actors::elem::connectable::{Connectable};
  use std::thread;
  use std::time::Duration;

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

  println!("wait 30 sec before notify and exit");
  for i in 0..30 {
    thread::sleep(Duration::from_secs(1));
    println!("{} ",i);
  }
  sched.notify(&source_id).unwrap();
  sched.stop();
}

#[cfg(not(feature = "experiment"))]
fn main() {
  println!("not running benchmarks. if you need them add --features \"experiment\" flag");
}