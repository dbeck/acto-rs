use super::bench_200ms;
use super::super::sample::measured_pipeline::MeasuredPipeline;
use super::spinner::Spinner;
use libc;

fn latency(stop_delay: u32, dummies: usize) {
  let spinner = Spinner::new();
  let mut pipe = MeasuredPipeline::new(spinner.get(), dummies);
  pipe.start();
  let printout = format!("pipe-latency-{}-{}",stop_delay,dummies);
  bench_200ms(printout.as_str(), |_v| {
    pipe.notify();
    pipe.wait();
  });
  unsafe { libc::usleep(stop_delay); }
  pipe.stop();
  spinner.stop();
}

pub fn run() {
  for i in 1..10 {
    latency(1,20*i);
    latency(100_000,20*i);
    latency(5_000_000,20*i);
  }
}
