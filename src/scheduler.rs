extern crate lossyq;
use std::thread;
// use super::task::{Task};

/*
pub enum Schedule {
  Loop,
  OnMessage,
  Stop,
  EndPlusUSec(usize),
  StartPlusUSec(usize),
}
*/

pub struct Scheduler {
  threads : Vec<thread::JoinHandle<i32>>,
  // looping Thread
  // - list, push back

  // on msg Thread
  // - map: ["name/type"] -> [ptr list]

  // scheduled Thread
  // - sorted multi map: [run_at] -> [ptr list]
}

impl Scheduler {
  // add source
  // add sink
  // add filter
  // add ysplit
  // add ymerge
}

pub fn new() -> Scheduler {
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
  }
}
