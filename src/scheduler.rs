extern crate lossyq;

use super::task::{Task};
use super::time_triggered;
use std::thread::{spawn, JoinHandle};
use std::collections::VecDeque;
use std::time::{Instant, Duration};

pub struct Scheduler {
  threads   : Vec<JoinHandle<i32>>,
  looping   : VecDeque<Box<Task>>,
  //tasks     : HashMap<String, Box<Task>>,
  // looping Thread
  // - list, push back

  // on msg Thread
  // - map: ["name/type"] -> [ptr list]

  // scheduled Thread
  // - sorted multi map: [run_at] -> [ptr list]
  timed : time_triggered::TimeTriggered,
}

impl Scheduler {

  pub fn add_task(&mut self, task : Box<Task>)
  {
    //use super::task::Task;
    //let n = task.name().clone();
    //self.tasks.insert(n, task);
    //self.looping.push_back(task);
    let plus_10us = Instant::now() + Duration::new(0,1000);
    self.timed.add(plus_10us, task);
  }
}

pub fn new() -> Scheduler {
  let mut ret = Scheduler{
    threads    : vec![],
    looping    : VecDeque::new(),
    //tasks      : HashMap::new(),
    timed      : time_triggered::new(),
  };
  let t = spawn(|| {
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
