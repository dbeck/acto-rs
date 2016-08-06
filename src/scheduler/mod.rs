extern crate lossyq;

mod collector;
mod executor;
mod timer;
mod on_msg;
mod stopped;

use self::lossyq::spsc::{Sender, channel};
use super::common::{Task, Message, IdentifiedReceiver, Direction, new_id};
use super::elem::{gather, scatter, sink};
//use std::thread::{spawn, JoinHandle};
//use std::collections::VecDeque;
//use std::time::{Instant, Duration};
//use std::sync::{Arc, Mutex, Condvar};

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
  // looper    : JoinHandle<()>,
  gate      : Sender<Message<Box<Task+Send>>>,
  collector : Box<Task+Send>,
  executor  : Box<Task+Send>,
  stopped   : Box<Task+Send>,

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

  pub fn add_task(&mut self, task: Box<Task+Send>)
  {
    use std::mem;
    //use super::task::Task;
    //let n = task.name().clone();
    //self.tasks.insert(n, task);
    //self.looping.push_back(task);
    // let _plus_10us = Instant::now() + Duration::new(0,1000);
    // self.timed.add(plus_10us, task);

    let mut to_send : Option<Message<Box<Task+Send>>> = Some(Message::Value(task));
    self.gate.put( |v| mem::swap(&mut to_send, v) );
    // TODO : handle overflow here
  }
}

//fn looper_entry() {}

pub fn new() -> Scheduler {

  use connectable::ConnectableN; // for collector
  let (gate_tx, gate_rx) = channel(100);

  let (mut collector_task, mut _collector_output) =
    gather::new(
      ".scheduler.collector",
      2,
      Box::new(collector::new()),
      1);

  let (mut executor_task, mut _executor_outputs) =
    scatter::new(
      ".scheduler.executor",
      2,
      Box::new(executor::new()),
      1);

  let (mut executor_task, mut _executor_outputs) =
    scatter::new(
      ".scheduler.executor",
      2,
      Box::new(executor::new()),
      1);

  let mut stopped_task = sink::new( ".scheduler.stopped", Box::new(stopped::new()));

  let mut gate_rx_opt = Some(
    IdentifiedReceiver{
      id:     new_id(String::from("Gate"), Direction::Out, 0),
      input:  gate_rx,
    }
  );

  collector_task.connect(0, &mut gate_rx_opt).unwrap();

  Scheduler{
    // looper     : spawn(|| { looper_entry(); }),
    collector    : collector_task,
    gate         : gate_tx,
    executor     : executor_task,
    stopped      : stopped_task,
    ////tasks      : HashMap::new(),
    //timed      : time_triggered::new(),
  }
}

#[cfg(test)]
mod tests {
  // use super::*;

  #[test]
  fn dummy() {
  }
}
