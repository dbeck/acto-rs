
use std::time::Instant;

pub struct Tick {
  time: Instant,
}

impl Tick {
  pub fn new() -> Tick {
    Tick {
      time: Instant::now(),
    }
  }

  pub fn elapsed_ns(&self) -> u64 {
    let ela = self.time.elapsed();
    ela.as_secs() * 1_000_000_000 + ela.subsec_nanos() as u64
  }
}
