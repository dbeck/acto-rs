use std::time::Duration;
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};
use time;

#[derive(Clone)]
pub struct Event {
  condvar : Arc<(Mutex<u64>, Condvar)>,
}

#[allow(dead_code)]
impl Event {
  pub fn notify(&mut self) {
    let &(ref lock, ref cvar) = &*(self.condvar);
    let mut started = lock.lock();
    *started += 1;
    cvar.notify_one();
  }

  pub fn wait(&mut self, ticket: u64, timeout_usec: u64) -> u64 {
    // wait for the thread to start up
    let start_at = time::precise_time_ns();
    let &(ref lock, ref cvar) = &*(self.condvar);

    loop {
      let mut locked_ticket = lock.lock();
      if *locked_ticket > ticket {
        return *locked_ticket;
      }

      cvar.wait_for(&mut locked_ticket, Duration::new(timeout_usec/1000_000, ((timeout_usec%1000_000) as u32) *1000 ));

      let now = time::precise_time_ns();
      if now > start_at+(1000*timeout_usec) {
        return *locked_ticket;
      }
    }
  }
}

#[allow(dead_code)]
pub fn new() -> Event {
  Event {
    condvar: Arc::new((Mutex::new(0u64), Condvar::new())),
  }
}
