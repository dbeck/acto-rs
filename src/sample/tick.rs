
use std::time::Instant;

#[derive(Copy, Clone)]
pub struct Tick {
  id:   u64,
  time: Instant,
}

impl Tick {
  pub fn new() -> Tick {
    Tick {
      id:   0,
      time: Instant::now(),
    }
  }

  pub fn elapsed_ns(&self) -> u64 {
    let ela = self.time.elapsed();
    ela.as_secs() * 1_000_000_000 + ela.subsec_nanos() as u64
  }
}
