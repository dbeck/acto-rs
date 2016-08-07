extern crate parking_lot;

use std::sync::Arc;
use self::parking_lot::{Mutex, Condvar};

pub struct Event {
  condvar : Arc<(Mutex<u64>, Condvar)>,
}

impl Event {
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
