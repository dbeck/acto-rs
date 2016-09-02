use lossyq::spsc::*;
use scheduler;
use super::{wrap, data};
use super::observer::{CountingReporter, TaskTracer};
use super::super::{Message, Schedule, Error, IdentifiedReceiver};
use super::super::elem::{source, filter, sink};
use std::sync::atomic::{AtomicUsize, Ordering};

// SchedulerData tests
//#[test]
//fn data_notify_()
//fn data_stop_()

#[test]
fn data_add_task() {
  let (filter_task, mut _filter_out) =
    filter::new( "Filter", 2, Box::new(ExecLogFilter::new(Schedule::OnExternalEvent)));

  let mut dta = data::new();
  let result = dta.add_task(filter_task);
  assert!(result.is_ok());
}

#[test]
fn data_entry_check_msg_wait_state() {

  use super::super::elem::connectable::Connectable;

  let (source_task, mut source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::OnExternalEvent)));

  let (mut filter_task, mut filter_out) =
    filter::new( "Filter", 2, Box::new(ExecLogFilter::new(Schedule::OnMessage(0,1))));

  let mut sink_task =
    sink::new( "Sink", Box::new(ExecLogSink::new(Schedule::OnMessage(0,1))));

  filter_task.connect(&mut source_out).unwrap();
  sink_task.connect(&mut filter_out).unwrap();

  let mut dta = data::new();
  assert!(dta.add_task(source_task).is_ok());
  assert!(dta.add_task(filter_task).is_ok());
  assert!(dta.add_task(sink_task).is_ok());

  // stop it first, so only a single execution is expected
  dta.stop();

  // launch an execution cycle
  dta.entry(0);

  //assert!(false);
}


// Event tests
// Handle tests

// TaskArray tests
//#[test]
//fn task_array_store_() {}
//fn task_array_eval_() {}
//fn task_array_notify_() {}

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
//fn wrap_attach_()
//fn wrap_detach_()

#[test]
fn wrap_eval_msg_triggered() {
  let sink_task =
    sink::new( "Sink", Box::new(ExecLogSink::new(Schedule::OnMessage(0, 1))));

  let input_ids : Vec<Option<usize>> = vec![Some(1)];
  let mut wrp = wrap::new(sink_task, 99, input_ids);
  let mut obs = CountingReporter::new();
  //let mut obs = TaskTracer::new();
  let tim = AtomicUsize::new(0);

  // first eval executes and changes the state
  wrp.eval(&mut obs, &tim); //, TaskState::MessageWait(0,1));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 0);
  assert_eq!(obs.msg_wait, 1);

  // second eval delays and leaves the state
  wrp.eval(&mut obs, &tim); //, TaskState::MessageWait(0,1));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.msg_wait, 2);

  // assert!(false);
}

#[test]
fn wrap_eval_ext_triggered() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::OnExternalEvent)));
  let mut wrp = wrap::new(source_task, 99, Vec::new());
  let mut obs = CountingReporter::new();
  //let mut obs = TaskTracer::new();
  let tim = AtomicUsize::new(0);

  // first eval will execute
  wrp.eval(&mut obs, &tim); //, TaskState::ExtEventWait(1));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 0);
  assert_eq!(obs.ext_wait, 1);

  // second eval will be delayed
  wrp.eval(&mut obs, &tim); //, TaskState::ExtEventWait(1));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.ext_wait, 2);

  // send a notify to stop delays and return the new value
  assert_eq!(wrp.notify(), 1);

  // third eval will execute and report the new
  wrp.eval(&mut obs, &tim); //, TaskState::ExtEventWait(2));
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.ext_wait, 3);

  // fourth eval will not
  wrp.eval(&mut obs, &tim); //, TaskState::ExtEventWait(2));
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 2);
  assert_eq!(obs.ext_wait, 4);
}

#[test]
fn wrap_eval_time_delayed() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new(Schedule::DelayUSec(2_000))));
  let mut wrp = wrap::new(source_task, 99, Vec::new());
  let mut obs = CountingReporter::new();
  let tim = AtomicUsize::new(0);

  // check, that first execution happens
  wrp.eval(&mut obs, &tim); //, TaskState::TimeWait(2_000));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 0);
  assert_eq!(obs.time_wait, 1);

  // check, that the second execution gets delayed
  wrp.eval(&mut obs, &tim); // , TaskState::TimeWait(2_000));
  assert_eq!(obs.executed, 1);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.time_wait, 2);

  // check, that it gets executed when time comes
  tim.fetch_add(2_001, Ordering::AcqRel);
  wrp.eval(&mut obs, &tim); //, TaskState::TimeWait(4_001));
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 1);
  assert_eq!(obs.time_wait, 3);

  // the next execution gets delayed again
  wrp.eval(&mut obs, &tim); // , TaskState::TimeWait(4_001);
  assert_eq!(obs.executed, 2);
  assert_eq!(obs.delayed, 2);
  assert_eq!(obs.time_wait, 4);
}

#[test]
fn wrap_eval_traced() {
  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(ExecLogSource::new_with_send(Schedule::DelayUSec(2_000))));
  let mut wrp = wrap::new(source_task, 99, Vec::new());
  let mut obs = TaskTracer::new();
  let tim = AtomicUsize::new(0);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  tim.fetch_add(2_001, Ordering::AcqRel);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  wrp.eval(&mut obs, &tim);
  //assert_eq!(true, false);
}


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

struct ExecLogFilter {
  ret: Schedule,
  exec_count: usize,
  with_send: bool,
}

impl filter::Filter for ExecLogFilter {
  type InputType = usize;
  type OutputType = usize;

  fn process(
        &mut self,
        _input:   &mut Option<IdentifiedReceiver<Self::InputType>>,
        output:  &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    self.exec_count += 1;
    println!("exec count: {}",self.exec_count);
    if self.with_send {
      output.put(|v| *v = Some(Message::Value(self.exec_count)) );
    }
    self.ret
  }
}

#[allow(dead_code)]
impl ExecLogFilter {
  fn new(sched: Schedule) -> ExecLogFilter {
    ExecLogFilter {
      ret: sched,
      exec_count: 0,
      with_send: false,
    }
  }

  fn new_with_send(sched: Schedule) -> ExecLogFilter {
    ExecLogFilter {
      ret: sched,
      exec_count: 0,
      with_send: true,
    }
  }
}

struct ExecLogSink {
  ret: Schedule,
  exec_count: usize,
}

impl sink::Sink for ExecLogSink {
  type InputType = usize;

  fn process(
        &mut self,
        _input:   &mut Option<IdentifiedReceiver<Self::InputType>>)
      -> Schedule {
    self.exec_count += 1;
    println!("exec count: {}",self.exec_count);
    self.ret
  }
}

#[allow(dead_code)]
impl ExecLogSink {
  fn new(sched: Schedule) -> ExecLogSink {
    ExecLogSink {
      ret: sched,
      exec_count: 0,
    }
  }
}
