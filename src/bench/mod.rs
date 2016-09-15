
use std::time::{Instant};
use lossyq::spsc::{channel, Sender};
use libc;
use super::scheduler;
use super::elem::{source, /*, filter, sink, ymerge, ysplit*/ };
use super::{Task};
use super::sample::dummy_source::{DummySource};

fn bench_200ms<F>(name: &str, fun: F) where F : FnMut(u64) {
  let start = Instant::now();
  let mut diff;
  let mut diff_ns;
  let mut iteration = 0u64;
  let mut fun = fun;
  loop {
    fun(iteration);    fun(iteration+1);  fun(iteration+2);  fun(iteration+3);
    fun(iteration+4);  fun(iteration+5);  fun(iteration+6);  fun(iteration+7);
    fun(iteration+8);  fun(iteration+9);  fun(iteration+10); fun(iteration+11);
    fun(iteration+12); fun(iteration+13); fun(iteration+14); fun(iteration+15);
    fun(iteration+16); fun(iteration+17); fun(iteration+18); fun(iteration+19);
    fun(iteration+20); fun(iteration+21); fun(iteration+22); fun(iteration+23);
    fun(iteration+24); fun(iteration+25); fun(iteration+26); fun(iteration+27);
    fun(iteration+28); fun(iteration+29); fun(iteration+30); fun(iteration+31);
    fun(iteration+32); fun(iteration+33); fun(iteration+34); fun(iteration+35);
    fun(iteration+36); fun(iteration+37); fun(iteration+38); fun(iteration+39);
    fun(iteration+40); fun(iteration+41); fun(iteration+42); fun(iteration+43);
    fun(iteration+44); fun(iteration+45); fun(iteration+46); fun(iteration+47);
    fun(iteration+48); fun(iteration+49); fun(iteration+50); fun(iteration+51);
    fun(iteration+52); fun(iteration+53); fun(iteration+54); fun(iteration+55);
    fun(iteration+56); fun(iteration+57); fun(iteration+58); fun(iteration+59);
    fun(iteration+60); fun(iteration+61); fun(iteration+62); fun(iteration+63);

    iteration += 64;
    diff = start.elapsed();
    diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
    if diff_ns > 200_000_000 {
      break;
    }
  }
  println!("bench_200ms  {}  avg {} ns, iter {}",name, diff_ns/iteration, iteration);
}

fn time_baseline() {
  bench_200ms("time-baseline", |_v| {} );
}

fn lossyq_send() {
  let (mut tx, _rx) = channel(100);
  bench_200ms("lossyq-send", |i| {
    tx.put(|v| *v = Some(i));
  });
}

fn lossyq_send_recv() {
  let (mut tx, mut rx) = channel(100);
  bench_200ms("lossyq-send-recv", |i| {
    tx.put(|v| *v = Some(i));
    for _i in rx.iter() {
    }
  });
}

fn lossyq_send_recv_1() {
  let (mut tx, mut rx) = channel(100);
  bench_200ms("lossyq-send-recv1", |i| {
    tx.put(|v| *v = Some(i));
    rx.iter().next();
  });
}

fn lossyq_recv() {
  let (mut _tx, mut rx) = channel::<u64>(100);
  bench_200ms("lossyq-recv", |_i| {
    for _ii in rx.iter() {
    }
  });
}

fn mpsc_send() {
  use std::sync::mpsc;
  let (tx, _rx) = mpsc::channel();
  bench_200ms("mpsc-send", |i| {
    tx.send(i).unwrap();
  });
}

fn mpsc_send_recv() {
  use std::sync::mpsc;
  let (tx, rx) = mpsc::channel();
  bench_200ms("mpsc-send-recv", |i| {
    tx.send(i).unwrap();
    rx.recv().unwrap();
  });
}

fn indirect_send_data() {
  let (mut tx, _rx) = channel(100);
  let sender = |val: u64, chan: &mut Sender<u64>| {
    chan.put(|v: &mut Option<u64>| *v = Some(val));
  };
  bench_200ms("indirect-send", |i| { sender(i, &mut tx); });
}

fn locked_send_data() {
  use std::sync::{Arc, Mutex};
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  bench_200ms("std::mutex+send", |i| {
    let mut x = locked.lock().unwrap();
    x.put(|v| *v = Some(i));
  });
}

fn lotted_send_data() {
  use std::sync::{Arc};
  use parking_lot::Mutex;
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  bench_200ms("parking_lot+send", |i| {
    let mut x = locked.lock();
    x.put(|v| *v = Some(i));
  });
}

fn source_execute() {
  let (mut source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(DummySource{}));

  bench_200ms("source-execute", |_i| {
    source_task.execute();
  });
}

fn source_execute_with_swap() {
  use std::sync::atomic::{AtomicPtr, Ordering};
  use std::ptr;

  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(DummySource{}));
  let source_ptr = AtomicPtr::new(Box::into_raw(source_task));

  bench_200ms("source-execute-w-swap", |_i| {
    let old_ptr = source_ptr.swap(ptr::null_mut(), Ordering::AcqRel);
    unsafe { (*old_ptr).execute(); }
    source_ptr.swap(old_ptr,  Ordering::AcqRel);
  });

  let _bx = unsafe { Box::from_raw(source_ptr.swap(ptr::null_mut(), Ordering::AcqRel)) };
}

// TODO : remove later
fn add_task_time() {
  let mut sources = Vec::with_capacity(1_000_000);
  for i in 0..1_000_000i32 {
    let name = format!("Source {}",i);
    let (source_task, mut _source_out) =
      source::new( name.as_str(), 2, Box::new(DummySource{}));
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
  println!("source add to sched: {} ns",diff_ns/1_000_000);
}

// TODO : remove later
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

pub fn run() {
  time_baseline();
  lossyq_send();
  lossyq_send_recv();
  lossyq_send_recv_1();
  lossyq_recv();
  mpsc_send();
  mpsc_send_recv();
  indirect_send_data();
  locked_send_data();
  lotted_send_data();
  source_execute();
  source_execute_with_swap();
  add_task_time();
  dummy_start_stop();
}
