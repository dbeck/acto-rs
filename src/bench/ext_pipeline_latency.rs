use super::bench_200ms;
use super::super::sample::measured_pipeline::MeasuredPipeline;
use super::spinner::Spinner;

fn latency_1() {
  let spinner = Spinner::new();
  let mut pipe = MeasuredPipeline::new(spinner.get());
  pipe.start();
  bench_200ms("pipe-latency", |_v| {
    pipe.notify();
    pipe.wait();
  });
  pipe.stop();
  spinner.stop();
}

pub fn run() {
  latency_1();
}
