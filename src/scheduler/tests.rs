use lossyq::spsc::*;
use scheduler;
use super::{wrap};
use super::observer::{CountingReporter, TaskTracer};
use super::super::{Message, Schedule, TaskState, Error};
use super::super::elem::{source};
use std::sync::atomic::{AtomicUsize, Ordering};

struct ExecLogSource {
  ret: Schedule,
  exec_count: usize,
  with_send: bool,
}

impl source::Source for ExecLogSource {
  type OutputType = usize;

  fn process(
        &mut self,
        output: &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    self.exec_count += 1;
    println!("exec count: {}",self.exec_count);
    if self.with_send {
      output.put(|v| *v = Some(Message::Value(self.exec_count)) );
    }
    self.ret
  }
}

impl ExecLogSource {
  fn new(sched: Schedule) -> ExecLogSource {
    ExecLogSource {
      ret: sched,
      exec_count: 0,
      with_send: false,
    }
  }

  fn new_with_send(sched: Schedule) -> ExecLogSource {
    ExecLogSource {
      ret: sched,
      exec_count: 0,
      with_send: true,
    }
  }
}

// TaskArray tests
// SchedulerData tests
// Event tests
// Handle tests

// Scheduler tests
//#[test]
//fn sched_add_task_() {}
//fn sched_start_() {}
//fn sched_start_with_threads_() {}
//fn sched_notify_() {}
//fn sched_stop_() {}

#[test]
fn sched_add_task() {
  let mut sched = scheduler::new();
  let first_id : usize;
  // first add succeeds
  {
    let (source_task, mut _source_out) =
      source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::DelayUSec(2_000))));
    let result = sched.add_task(source_task);
    assert!(result.is_ok());
    first_id = match result {
      Ok(task_id) => { task_id.id() }
      _           => { 9999 }
    };
    assert!(first_id != 9999);
  }
  // second add with the same name fails
  {
    let (source_task, mut _source_out) =
      source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::DelayUSec(2_000))));
    let result = sched.add_task(source_task);
    assert!(result.is_err());
    let already_exists =  match result {
      Err(Error::AlreadyExists) => { true },
      _ => { false }
    };
    assert!(already_exists);
  }
  // third add succeeds and returns a different id
  {
    let (source_task, mut _source_out) =
      source::new( "Source 3", 2, Box::new(ExecLogSource::new(Schedule::DelayUSec(2_000))));
    let result = sched.add_task(source_task);
    assert!(result.is_ok());
    let third_id = match result {
      Ok(task_id) => { task_id.id() }
      _           => { 9999 }
    };
    assert!(third_id != first_id);
    assert!(third_id != 9999);
  }
}

// TaskWrap tests
#[test]
fn wrap_execute_time_delayed() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::DelayUSec(2_000))));
  let mut wrp = wrap::new(source_task, 99);
  let mut obs = CountingReporter::new();
  let tim = AtomicUsize::new(0);

  // check, that first execution happens
  assert_eq!(wrp.eval(&mut obs, &tim), TaskState::TimeWait(2_000));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 0);
  assert_eq!(obs.time_wait, 1);

  // check, that the second execution gets delayed
  assert_eq!(wrp.eval(&mut obs, &tim), TaskState::TimeWait(2_000));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.time_wait, 2);

  // check, that it gets executed when time comes
  tim.fetch_add(2_001, Ordering::SeqCst);
  assert_eq!(wrp.eval(&mut obs, &tim), TaskState::TimeWait(4_001));
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.time_wait, 3);

  // the next execution gets delayed again
  assert_eq!(wrp.eval(&mut obs, &tim), TaskState::TimeWait(4_001));
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 2);
  assert_eq!(obs.time_wait, 4);
}

#[test]
fn wrap_execute_traced() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new_with_send(Schedule::DelayUSec(2_000))));
  let mut wrp = wrap::new(source_task, 99);
  let mut obs = TaskTracer::new();
  let tim = AtomicUsize::new(0);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  tim.fetch_add(2_001, Ordering::SeqCst);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  //assert_eq!(true, false);
}
