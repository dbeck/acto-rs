extern crate parking_lot;

use std::sync::Arc;
use self::parking_lot::{Mutex, Condvar};

pub struct Event {
  condvar : Arc<(Mutex<u64>, Condvar)>,
}

impl Event {
  pub fn notify(&mut self) {

  }

  pub fn wait(&mut self, timeout_usec: usize) {
  }
}

pub fn new() -> Event {
  Event {
    condvar: Arc::new((Mutex::new(0u64), Condvar::new())),
  }
}

#[cfg(test)]
mod tests {
  //use super::*;

  #[test]
  fn dummy() { }
}
