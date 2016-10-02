
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Notification {
  pending:   AtomicUsize,
  delivered: AtomicUsize,
}

impl Notification {
  pub fn new() -> Notification {
    Notification {
      pending:    AtomicUsize::new(0),
      delivered:  AtomicUsize::new(0),
    }
  }

  pub fn notify(&mut self) -> usize {
    1 + self.pending.fetch_add(1, Ordering::AcqRel)
  }

  pub fn flush(&mut self) -> usize {
    let diff = self.pending.load(Ordering::Acquire) - self.delivered.load(Ordering::Acquire);
    if diff > 0 {
      self.delivered.fetch_add(diff, Ordering::AcqRel);
      diff
    } else {
      0
    }
  }
}
