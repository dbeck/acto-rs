extern crate acto_rs as actors;
extern crate lossyq;
extern crate parking_lot;
extern crate libc;

use std::time::{Duration, Instant};
//use lossyq::spsc::Receiver;
use lossyq::spsc::{channel, Sender};
use actors::scheduler;
use actors::elem::{source, /*, filter, sink, ymerge, ysplit*/ };
use actors::{Message, Task, Schedule};

#[allow(dead_code)]
struct DummySource {
}

impl source::Source for DummySource {
  type OutputType = u64;

  fn process(
        &mut self,
        _output: &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    Schedule::Loop
  }
}

#[allow(dead_code)]
struct CountingSource {
  count: usize,
}

impl source::Source for CountingSource {
  type OutputType = u64;

  fn process(
        &mut self,
        _output: &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    self.count += 1;
    Schedule::Loop
  }
}

impl Drop for CountingSource {
  fn drop(&mut self) {
    println!("CountingSource executed {} times", self.count);
  }
}

#[derive(Copy, Clone)]
struct SourceState {
  count      : u64,
  start      : Option<Instant>,
}

#[allow(dead_code)]
impl source::Source for SourceState {
  type OutputType = u64;

  fn process(
        &mut self,
        output: &mut Sender<Message<Self::OutputType>>)
      -> Schedule {
    output.put(|x| *x = Some(Message::Value(self.count)));
    if self.count % 10_000_000 == 0 {
      if let Some(start) = self.start {
        let diff_t = start.elapsed();
        let diff_ns = diff_t.as_secs() * 1000_000_000 + diff_t.subsec_nanos() as u64;
        println!("diff t: {} {} {}/sec", diff_ns/self.count,self.count,10_000_000_000/(diff_ns/self.count));
      } else {
        self.start = Some(Instant::now());
      }
    }
    self.count += 1;
    Schedule::Loop
  }
}

#[allow(dead_code)]
fn time_baseline() {
  let start = Instant::now();
  let mut diff = Duration::new(0, 0);
  for _i in 0..1_000_000 {
    diff = start.elapsed();
  }
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("timer overhead: {} ns", diff_ns/1_000_000);
}

#[allow(dead_code)]
fn send_data() {
  let start = Instant::now();
  let (mut tx, _rx) = channel(100);
  for i in 0..10_000_000i32 {
    tx.put(|v| *v = Some(i));
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("send i32 overhead: {} ns", diff_ns/10_000_000);
}

#[allow(dead_code)]
fn indirect_send_data() {
  let start = Instant::now();
  let (mut tx, _rx) = channel(100);
  for i in 0..10_000_000i32 {
    let sender = |val: i32, chan: &mut lossyq::spsc::Sender<i32>| {
      chan.put(|v: &mut Option<i32>| *v = Some(val));
    };
    sender(i, &mut tx);
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("indirect i32 overhead: {} ns", diff_ns/10_000_000);
}

#[allow(dead_code)]
fn locked_send_data() {
  use std::sync::{Arc, Mutex};
  let start = Instant::now();
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  for i in 0..10_000_000i32 {
    let mut x = locked.lock().unwrap();
    x.put(|v| *v = Some(i));
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("locked send i32 overhead: {} ns",diff_ns/10_000_000);
}

#[allow(dead_code)]
fn lotted_send_data() {
  use std::sync::{Arc};
  use parking_lot::Mutex;
  let start = Instant::now();
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  for i in 0..10_000_000i32 {
    let mut x = locked.lock();
    x.put(|v| *v = Some(i));
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("lotted send i32 overhead: {} ns",diff_ns/10_000_000);
}

#[allow(dead_code)]
fn mpsc_send_data() {
  use std::sync::{Arc,mpsc};
  let start = Instant::now();
  let (tx, rx) = mpsc::channel();
  let atx = Arc::new(tx);
  for i in 0..10_000_000i32 {
    atx.send(i).unwrap();
    rx.recv().unwrap();
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("mpsc send i32 overhead: {} ns",diff_ns/10_000_000);
}

#[allow(dead_code)]
fn receive_data() {
  let start = Instant::now();
  let (_tx, mut rx) = channel(100);
  let mut sum = 0;
  for _i in 0..10_000_000i32 {
    for i in rx.iter() {
      sum += i;
    }
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("receive i32 overhead: {} ns (sum {})",diff_ns/10_000_000, sum);
}

#[allow(dead_code)]
fn source_send_data() {
  let (mut source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(SourceState{count:0, start:None}));

  let start = Instant::now();
  for _i in 0..10_000_000 {
    source_task.execute();
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("source execute: {} ns",diff_ns/10_000_000);
}

#[allow(dead_code)]
fn source_send_data_with_swap() {
  use std::sync::atomic::{AtomicPtr, Ordering};
  use std::ptr;

  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(SourceState{count:0, start:None}));
  let source_ptr = AtomicPtr::new(Box::into_raw(source_task));

  let start = Instant::now();
  for _i in 0..10_000_000 {
    let old_ptr = source_ptr.swap(ptr::null_mut(), Ordering::AcqRel);
    unsafe { (*old_ptr).execute(); }
    source_ptr.swap(old_ptr,  Ordering::AcqRel);
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("source execute: {} ns (w/ swap)",diff_ns/10_000_000);
}

#[allow(dead_code)]
fn add_task_time() {
  let mut sources = vec![];
  for i in 0..3_000_000i32 {
    let name = format!("Source {}",i);
    let (source_task, mut _source_out) =
      source::new( name.as_str(), 2, Box::new(SourceState{count:0, start:None}));
    sources.push(source_task);
  }
  let mut sched = scheduler::new();

  let start = Instant::now();
  for i in sources {
    match sched.add_task(i) {
      Ok(_) => {}
      Err(e) => { println!("cannot add because of: {:?}", e); }
    }
  }
  let diff = start.elapsed();
  let diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
  println!("source add to sched: {} ns",diff_ns/3_000_000);
}

#[allow(dead_code)]
fn start_stop() {
  let mut sched = scheduler::new();
  for i in 0..10_000 {
    let name = format!("Source {}",i);
    let (source_task, mut _source_out) =
      source::new( name.as_str(), 2, Box::new(SourceState{count:0, start:None}));
    match sched.add_task(source_task) {
      Ok(_) => {}
      Err(e) => { println!("cannot add: {} because of: {:?} idx: {}", name, e, i); }
    }
  }
  sched.start_with_threads(1);
  unsafe { libc::sleep(5); }
  sched.stop();
}

#[allow(dead_code)]
fn dummy_start_stop() {
  let v = vec![1, 2, 5, 10, 16, 32, 64, 100, 128, 256,
    1000, 5_000, 10_000, 30_000, 50_000, 100_000, 200_000, 500_000];
  for x in v {
    println!("add {} tasks", x);
    let mut sched = scheduler::new();
    for i in 0..x {
      let name = format!("Source {}",i);
      let (source_task, mut _source_out) =
        source::new( name.as_str(), 2, Box::new(DummySource{}));
      match sched.add_task(source_task) {
        Ok(_) => {}
        Err(e) => { println!("cannot add: {} because of: {:?} idx: {}", name, e, i); }
      }
    }
    sched.start_with_threads(1);
    unsafe { libc::sleep(2); }
    sched.stop();
  }
}

#[cfg(feature = "bench")]
use actors::bench;

#[cfg(feature = "bench")]
pub fn run_benchmarks() {
  println!("hello from bench");
}

#[cfg(not(feature = "bench"))]
pub fn run_benchmarks() {
  println!("not running benchmarks. if you need them add --features \"bench\" flag");
}

fn main() {
  run_benchmarks();
  time_baseline();
  send_data();
  indirect_send_data();
  locked_send_data();
  lotted_send_data();
  mpsc_send_data();
  receive_data();
  source_send_data();
  source_send_data_with_swap();
  add_task_time();
  start_stop();
  dummy_start_stop();
}
