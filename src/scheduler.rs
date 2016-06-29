extern crate lossyq;
use std::thread;

pub struct Scheduler {
  threads : Vec<thread::JoinHandle<i32>>,
}

impl Scheduler {

  // start/add thread
  // "start" loop()

  // stop() "all"

  // spawn "runnable" <- worker/supervisor
  //  -- add(name,runnable)

  // send msg(name[], msg)

  // str???
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
