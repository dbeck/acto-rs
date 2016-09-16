use super::bench_200ms;
use super::super::sample::ext_pipeline::ExtPipeline;

fn latency_1() {
  let mut pipe = ExtPipeline::new();
  pipe.start();
  bench_200ms("pipe-latency", |_v| {
    pipe.notify();
    pipe.wait();  
  });
  pipe.stop();
}

pub fn run() {
  latency_1();
}
