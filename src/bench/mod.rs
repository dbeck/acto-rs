pub mod basics;
pub mod ext_pipeline_latency;

use std::time::{Instant};
use libc;
use super::scheduler;
use super::elem::{source, /*, filter, sink, ymerge, ysplit*/ };
use super::sample::dummy_source::{DummySource};

fn bench_200ms<F>(name: &str, fun: F) where F : FnMut(u64) {
  let start = Instant::now();
  let mut diff;
  let mut diff_ns;
  let mut iteration = 0u64;
  let mut fun = fun;
  let mut repeat = 1;
  loop {
    // unrolled loop to reduce the number of timer calls and also amortize
    // the cost of the initial branch int the for's condition
    for _i in 0..repeat {
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
    }
    diff = start.elapsed();
    diff_ns = diff.as_secs() * 1000_000_000 + diff.subsec_nanos() as u64;
    if diff_ns > 200_000_000 {
      break;
    }
    // calculate how much it can run without calling the timer function
    // again
    let remaining = 200_000_000 - diff_ns;
    repeat = 1+(remaining/diff_ns);
  }
  println!("bench_200ms  {}  avg {} ns, iter {}, {}m iter/s",
    name, diff_ns/iteration, iteration, 1_000*iteration/diff_ns );
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
  basics::run();
  ext_pipeline_latency::run();
  add_task_time();
  dummy_start_stop();
}
