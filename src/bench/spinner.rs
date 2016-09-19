
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, AtomicBool, Ordering};
use std::thread;

pub struct Spinner {
  start:    Arc<AtomicUsize>,
  stop:     Arc<AtomicBool>,
  handle:   thread::JoinHandle<()>,
}

impl Spinner {
  pub fn new() -> Spinner {
    let start = Arc::new(AtomicUsize::new(0));
    let stop  = Arc::new(AtomicBool::new(false));
    Spinner {
      start:  start.clone(),
      stop:   stop.clone(),
      handle: thread::spawn(move || Spinner::entry(start, stop)),
    }
  }

  pub fn get(&self) -> Arc<AtomicUsize> {
    self.start.clone()
  }

  pub fn stop(self) {
    self.stop.store(true, Ordering::Release);
    self.handle.join().unwrap();
  }

  fn entry(start: Arc<AtomicUsize>, stop: Arc<AtomicBool>) {
    loop {
      if stop.load(Ordering::Acquire) == true {
        break;
      }
      start.fetch_add(1, Ordering::AcqRel);
    }
  }
}
