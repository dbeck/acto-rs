extern crate minions;
extern crate lossyq;
extern crate parking_lot;
extern crate time;

//use lossyq::spsc::Receiver;
use lossyq::spsc::{channel, Sender};
use minions::scheduler;
use minions::elem::{source, /*, filter, sink, ymerge, ysplit*/ };
use minions::common;
use minions::common::{Message, Task};

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
      -> common::Schedule {
    output.put(|x| *x = Some(Message::Value(self.count)));
    self.count += 1;
    if self.count % 1_000_000 == 0 {
      let now = time::precise_time_ns();
      let diff_t = now-self.start;
      println!("diff t: {} {} {}/sec", diff_t/self.count,self.count,1_000_000_000/(diff_t/self.count));
    }
    common::Schedule::Loop
  }
}

#[allow(dead_code)]
fn test_sched() {
  //use minions::connectable::{Connectable, ConnectableY};
  let (source_task_1, mut _source_out) = source::new( "Source 1", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_2, mut _source_out) = source::new( "Source 2", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_3, mut _source_out) = source::new( "Source 3", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_4, mut _source_out) = source::new( "Source 4", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_5, mut _source_out) = source::new( "Source 5", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_6, mut _source_out) = source::new( "Source 6", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_7, mut _source_out) = source::new( "Source 7", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_8, mut _source_out) = source::new( "Source 8", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let (source_task_9, mut _source_out) = source::new( "Source 9", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let mut sched = scheduler::new_old();
  sched.add_task(source_task_1); // 189  243 166  -- -- --- --
  sched.add_task(source_task_2); // 244  369 221  -- 55 126 55
  sched.add_task(source_task_3); // 307  508 280  --163 139 59
  sched.add_task(source_task_4); // 363  645 335  -- 56 137 55
  sched.add_task(source_task_5); // 422  768 392  -- 59 123 57
  sched.add_task(source_task_6); // 482  910 451  -- 60 142 59
  sched.add_task(source_task_7); // 533 1033 504  -- 51 123 53
  sched.add_task(source_task_8); // 598 1170 562  -- 65 137 111
  sched.add_task(source_task_9); // 650 1286 615  -- 52 116 53
  sched.stop();
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
    source::new( "Source", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
  let mut reporter = scheduler::CountingReporter{ count: 0 };

  let start = time::precise_time_ns();
  for _i in 0..10_000_000i32 {
    source_task.execute(&mut reporter);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("source execute: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn collector_time() {
  use minions::elem::{gather, filter};
  use minions::connectable::ConnectableN; // for collector
  use minions::scheduler::{collector, loop_back};

  let (mut collector_task, mut _collector_output) =
    gather::new(".scheduler.collector", 1000, Box::new(collector::new()), 4);

  let (mut _loopback_task, mut loopback_output_1) =
    filter::new(".scheduler.loopback", 1000, Box::new(loop_back::new()));
  let (mut _loopback_task, mut loopback_output_2) =
    filter::new(".scheduler.loopback", 1000, Box::new(loop_back::new()));
  let (mut _loopback_task, mut loopback_output_3) =
    filter::new(".scheduler.loopback", 1000, Box::new(loop_back::new()));
  let (mut _loopback_task, mut loopback_output_4) =
    filter::new(".scheduler.loopback", 1000, Box::new(loop_back::new()));

  collector_task.connect(0, &mut loopback_output_1).unwrap();
  collector_task.connect(1, &mut loopback_output_2).unwrap();
  collector_task.connect(2, &mut loopback_output_3).unwrap();
  collector_task.connect(3, &mut loopback_output_4).unwrap();

  let mut reporter = scheduler::CountingReporter{ count: 0 };

  let start = time::precise_time_ns();
  for _i in 0..100_000_000i32 {
    collector_task.execute(&mut reporter);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("collector execute: {} ns",diff/100_000_000);
}

#[allow(dead_code)]
fn add_task_time() {
  let mut sources = vec![];
  for _i in 0..10_000_000i32 {
    let (source_task, mut _source_out) =
      source::new( "Source", 2, Box::new(SourceState{count:0, start:0}));
    sources.push(source_task);
  }
  let mut sched = scheduler::new();

  let start = time::precise_time_ns();
  for i in sources {
    sched.add_task(i);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("source add to sched: {} ns",diff/10_000_000);
}

#[allow(dead_code)]
fn sched_loop_time() {
  let mut sources = vec![];
  for _i in 0..50_000i32 {
    let (source_task, mut _source_out) =
      source::new( "Source", 2, Box::new(SourceState{count:0, start:time::precise_time_ns()}));
    sources.push(source_task);
  }
  let mut sched = scheduler::new();
  for i in sources {
    sched.add_task(i);
  }

  let start = time::precise_time_ns();
  for _i in 0..5_000 {
    sched.start();
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("sched loop: {} ns => {} ns",diff/5_000,diff/5_000/50_000);
}

use std::sync::atomic::{AtomicPtr, Ordering};
use std::ptr;

fn main() {

  let mut v = Box::new(9 as usize);
  let nil = AtomicPtr::new(ptr::null_mut::<usize>());
  let p1  = AtomicPtr::new(v.as_mut());
  let p2  = AtomicPtr::new(ptr::null_mut::<usize>());

  println!("nil {:?}", nil.load(Ordering::SeqCst));
  println!("p1 {:?}",  p1.load(Ordering::SeqCst));
  println!("p2 {:?}",  p2.load(Ordering::SeqCst));

  assert!(p1.load(Ordering::SeqCst) != nil.load(Ordering::SeqCst));
  assert!(p2.load(Ordering::SeqCst) == nil.load(Ordering::SeqCst));

  //time_baseline();
  //send_data();
  //indirect_send_data();
  //locked_send_data();
  //lotted_send_data();
  //mpsc_send_data();
  //receive_data();
  //source_send_data();
  //collector_time();
  //add_task_time();
  //sched_loop_time();
  //test_sched();
}
