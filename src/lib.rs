extern crate lossyq;
extern crate parking_lot;

pub mod scheduler;
pub mod common;
pub mod connectable;
pub mod elem;

#[cfg(test)]
mod tests {
  //use super::*;

  #[test]
  fn dummyx() {
    use std::thread;
    use std::sync::Arc;

    use parking_lot::{Mutex, Condvar};
    let pair = Arc::new((Mutex::new(false), Condvar::new()));
    let pair2 = pair.clone();

    let t = thread::spawn(move|| {
      let &(ref lock, ref cvar) = &*pair2;
      let mut started = lock.lock();
      *started = true;
      cvar.notify_one();
    });

    // wait for the thread to start up
    let &(ref lock, ref cvar) = &*pair;
    let mut started = lock.lock();
    while !*started {
        cvar.wait(&mut started);
    }
    t.join().unwrap();

  }
}
