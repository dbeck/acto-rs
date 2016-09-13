use std::time::{Duration, Instant};
use std::sync::Arc;
use parking_lot::{Mutex, Condvar};

#[derive(Clone)]
pub struct Event {
  condvar : Arc<(Mutex<u64>, Condvar)>,
}

impl Event {
  pub fn notify(&mut self) {
    let &(ref lock, ref cvar) = &*(self.condvar);
    let mut started = lock.lock();
    *started += 1;
    cvar.notify_one();
  }

  #[allow(dead_code)]
  pub fn ready(&self, ticket: u64) -> (bool, u64) {
    let &(ref lock, ref _cvar) = &*(self.condvar);
    let started = lock.lock();
    (*started > ticket, *started)
  }

  #[allow(dead_code)]
  pub fn wait(&mut self, ticket: u64, timeout_usec: u64) -> u64 {
    // wait for the thread to start up
    let start_at = Instant::now();
    let (ref lock, ref cvar) = *(self.condvar);
    let timeout = Duration::new(timeout_usec/1000_000, ((timeout_usec%1000_000) as u32) *1000);

    loop {
      let mut locked_ticket = lock.lock();
      if *locked_ticket > ticket {
        return *locked_ticket;
      }

      cvar.wait_for(&mut locked_ticket, timeout);

      if start_at.elapsed() >= timeout {
        return *locked_ticket;
      }
    }
  }
}

pub fn new() -> Event {
  Event {
    condvar: Arc::new((Mutex::new(0u64), Condvar::new())),
  }
}
