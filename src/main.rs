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

impl source::Source for SourceState {
  type OutputType = u64;

  fn process(
        &mut self,
        output: &mut Sender<Message<Self::OutputType>>)
      -> common::Schedule {
    output.put(|x| *x = Some(Message::Value(self.count)));
    self.count += 1;
    if self.count % 1000000 == 0 {
      let now = time::precise_time_ns();
      let diff_t = now-self.start;
      println!("diff t: {} {} {}/sec", diff_t/self.count,self.count,1_000_000_000/(diff_t/self.count));
    }
    common::Schedule::Loop
  }
}

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
  let mut sched = scheduler::new();
  sched.add_task(source_task_1);
  sched.add_task(source_task_2);
  sched.add_task(source_task_3);
  sched.add_task(source_task_4);
  sched.add_task(source_task_5);
  sched.add_task(source_task_6);
  sched.add_task(source_task_7);
  sched.add_task(source_task_8);
  sched.add_task(source_task_9);
  sched.stop();
}

fn time_baseline() {
  let start = time::precise_time_ns();
  let mut end = 0;
  for _i in 0..1_000_000 {
    end = time::precise_time_ns();
  }
  let diff = end - start;
  println!("timer overhead: {} ns", diff/1_000_000);
}

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
  for _i in 0..10_000_000i32 {
    collector_task.execute(&mut reporter);
  }
  let end = time::precise_time_ns();
  let diff = end - start;
  println!("collector execute: {} ns",diff/10_000_000);
}

fn main() {
  time_baseline();
  send_data();
  receive_data();
  source_send_data();
  collector_time();
  test_sched();
}
