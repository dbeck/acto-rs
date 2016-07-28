extern crate lossyq;

mod collector;

use super::common::{Task};
//use super::time_triggered;
use std::thread::{spawn, JoinHandle};
use std::collections::VecDeque;
use std::time::{Instant, Duration};
use std::sync::{Arc, Mutex, Condvar};

//
//
//let pair = Arc::new((Mutex::new(false), Condvar::new()));
//let pair2 = pair.clone();
//
//// Inside of our lock, spawn a new thread, and then wait for it to start
//thread::spawn(move|| {
//    let &(ref lock, ref cvar) = &*pair2;
//    let mut started = lock.lock().unwrap();
//    *started = true;
//    cvar.notify_one();
//});
//
//// wait for the thread to start up
//let &(ref lock, ref cvar) = &*pair;
//let mut started = lock.lock().unwrap();
//while !*started {
//    started = cvar.wait(started).unwrap();
//}

pub struct Scheduler {
  //looper    : JoinHandle<()>,
  //looping   : VecDeque<Box<Task+Send>>,
  //tasks     : HashMap<String, Box<Task>>,
  // looping Thread
  // - list, push back

  // on msg Thread
  // - map: ["name/type"] -> [ptr list]

  // scheduled Thread
  // - sorted multi map: [run_at] -> [ptr list]
  //timed : time_triggered::TimeTriggered,
}

impl Scheduler {

  pub fn add_task(&mut self, _task : Box<Task+Send>)
  {
    //use super::task::Task;
    //let n = task.name().clone();
    //self.tasks.insert(n, task);
    //self.looping.push_back(task);
    let _plus_10us = Instant::now() + Duration::new(0,1000);
    // self.timed.add(plus_10us, task);
  }
}

// Looping:
// - input from add_task
// - input from time_triggered
// - input from message_triggered
// - output to time_triggered
// - output to message_triggered
// - state

// Timed:
// - input from looping
// - output to looping
// - state

// Message TimeTriggered:
// - input from looping
// - output to looping
// - state

// Stoppped:
// - input from looping
// - state

fn looper_entry() {

}

pub fn new() -> Scheduler {
  let mut ret = Scheduler{
    //looper     : spawn(|| { looper_entry(); }),
    //looping    : VecDeque::new(),
    ////tasks      : HashMap::new(),
    //timed      : time_triggered::new(),
  };
  ret
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() {
  }
}
