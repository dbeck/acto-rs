extern crate lossyq;
use std::thread;

pub struct Scheduler {
  threads : Vec<thread::JoinHandle<i32>>,
  // free inputs
  // free outputs
}

impl Scheduler {
  // add generator
  // add sink
  // add worker
  // add filter
}

pub fn new(_n_threads : usize) -> Scheduler {
  let mut ret = Scheduler{
    threads : vec![],
  };
  let t = thread::spawn(|| {
    1 as i32
    });
  ret.threads.push(t);
  ret
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() {
    remove_me();
  }
}
