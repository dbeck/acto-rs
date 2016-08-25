use lossyq::spsc::*;
use super::{wrap, CountingReporter};
use super::super::{Message, Schedule, TaskState};
use super::super::elem::{source};
use std::sync::atomic::{AtomicUsize};

struct TestSource {
  ret: Schedule,
  exec_count: usize,
}

impl source::Source for TestSource {
  type OutputType = u64;

  fn process(
        &mut self,
        _output: &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    self.exec_count += 1;
    println!("exec count: {}",self.exec_count);
    self.ret
  }
}

#[test]
fn wrap_delayed() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(TestSource{ret:Schedule::DelayUSec(2_000), exec_count:0}));
  let mut wrp = wrap::new(source_task, 99);
  let mut obs = CountingReporter::new();
  let tim = AtomicUsize::new(0);
  assert_eq!(wrp.execute(&mut obs, &tim), TaskState::TimeWait(2_000));
  // TODO : check counts
  assert_eq!(wrp.execute(&mut obs, &tim), TaskState::TimeWait(2_000));
  // TODO : check counts
}
