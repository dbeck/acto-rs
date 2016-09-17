pub mod basics;
pub mod ext_pipeline_latency;

use std::time::{Instant};

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
    repeat = 1+(remaining/diff_ns/10*9);
  }
  println!("bench_200ms  {}  avg {} ns, iter {}, {}m iter/s {}k iter/s",
    name,
    diff_ns/iteration,
    iteration,
    1_000*iteration/diff_ns,
    iteration/(diff_ns/1_000_000));
}

pub fn run() {
  basics::run();
  ext_pipeline_latency::run();
}
