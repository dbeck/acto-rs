extern crate acto_rs as actors;
extern crate lossyq;
extern crate parking_lot;
extern crate time;
extern crate libc;

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
    println!("CountingSource executed {} times",self.count);
  }
}

#[derive(Copy, Clone)]
struct SourceState {
  count      : u64,
  start      : u64,
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
      let now = time::precise_time_ns();
      if self.start == 0 {
        self.start = now;
      } else {
        let diff_t = now-self.start;
        println!("diff t: {} {} {}/sec", diff_t/self.count,self.count,10_000_000_000/(diff_t/self.count));
      }
    }
    self.count += 1;
    Schedule::Loop
  }
}

#[allow(dead_code)]
fn time_baseline() {
  let start = time::precise_time_ns();
  let mut end = 0;
  for _i in 0..1_000_000 {
    end = time::precise_time_ns();
  }
  let diff = end - start;
  println!("timer overhead: {} ns", diff/1_000_000);
}

#[allow(dead_code)]
fn send_data() {
  let start = time::precise_time_ns();
  let (mut tx, _rx) = channel(100);
  for i in 0..10_000_000i32 {
    tx.put(|v| *v = Some(i));
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("send i32 overhead: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn indirect_send_data() {
  let start = time::precise_time_ns();
  let (mut tx, _rx) = channel(100);
  for i in 0..10_000_000i32 {
    let sender = |val: i32, chan: &mut lossyq::spsc::Sender<i32>| {
      chan.put(|v: &mut Option<i32>| *v = Some(val));
    };
    sender(i, &mut tx);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("indirect i32 overhead: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn locked_send_data() {
  use std::sync::{Arc, Mutex};
  let start = time::precise_time_ns();
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  for i in 0..10_000_000i32 {
    let mut x = locked.lock().unwrap();
    x.put(|v| *v = Some(i));
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("locked send i32 overhead: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn lotted_send_data() {
  use std::sync::{Arc};
  use parking_lot::Mutex;
  let start = time::precise_time_ns();
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  for i in 0..10_000_000i32 {
    let mut x = locked.lock();
    x.put(|v| *v = Some(i));
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("lotted send i32 overhead: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn mpsc_send_data() {
  use std::sync::{Arc,mpsc};
  let start = time::precise_time_ns();
  let (tx, rx) = mpsc::channel();
  let atx = Arc::new(tx);
  for i in 0..10_000_000i32 {
    atx.send(i).unwrap();
    rx.recv().unwrap();
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("mpsc send i32 overhead: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn receive_data() {
  let start = time::precise_time_ns();
  let (_tx, mut rx) = channel(100);
  let mut sum = 0;
  for _i in 0..10_000_000i32 {
    for i in rx.iter() {
      sum += i;
    }
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("receive i32 overhead: {} ns  sum:{}",diff/10_000_000,sum);
}

#[allow(dead_code)]
fn source_send_data() {
  let (mut source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(SourceState{count:0, start:0}));

  let start = time::precise_time_ns();
  for _i in 0..10_000_000 {
    source_task.execute();
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("source execute: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn source_send_data_with_swap() {
  use std::sync::atomic::{AtomicPtr, Ordering};
  use std::ptr;

  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(SourceState{count:0, start:0}));
  let source_ptr = AtomicPtr::new(Box::into_raw(source_task));

  let start = time::precise_time_ns();
  for _i in 0..10_000_000 {
    let old_ptr = source_ptr.swap(ptr::null_mut(), Ordering::AcqRel);
    unsafe { (*old_ptr).execute(); }
    source_ptr.swap(old_ptr,  Ordering::AcqRel);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("source execute: {} ns (w/ swap)",diff/10_000_000);
}

#[allow(dead_code)]
fn add_task_time() {
  let mut sources = vec![];
  for i in 0..3_000_000i32 {
    let name = format!("Source {}",i);
    let (source_task, mut _source_out) =
      source::new( name.as_str(), 2, Box::new(SourceState{count:0, start:0}));
    sources.push(source_task);
  }
  let mut sched = scheduler::new();

  let start = time::precise_time_ns();
  for i in sources {
    match sched.add_task(i) {
      Ok(_) => {}
      Err(e) => { println!("cannot add because of: {:?}", e); }
    }
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("source add to sched: {} ns",diff/3_000_000);
}

#[allow(dead_code)]
fn start_stop() {
  let mut sched = scheduler::new();
  for i in 0..10_000 {
    let name = format!("Source {}",i);
    let (source_task, mut _source_out) =
      source::new( name.as_str(), 2, Box::new(SourceState{count:0, start:0}));
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
  let v = vec![1, 2, 5, 10, 100, 1000, 5_000, 10_000, 30_000, 50_000, 100_000, 200_000, 500_000];
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

fn main() {
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
