extern crate lossyq;

mod collector;
mod executor;
mod timer;
mod on_msg;
mod event;
mod loop_back;

use self::lossyq::spsc::{Sender, channel};
use super::common::{Task, Message, IdentifiedReceiver, Direction, new_id};
use super::elem::{gather, scatter, filter};
use super::connectable;

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
  loopback  : Box<Task+Send>,
  onmsg     : Box<Task+Send>,
  timer     : Box<Task+Send>,
  stopped   : Option<IdentifiedReceiver<Box<Task+Send>>>,

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
  use connectable::Connectable;  // for executor

  let (gate_tx, gate_rx) = channel(100);

  let (mut collector_task, mut collector_output) =
    gather::new(
      ".scheduler.collector",
      2,
      Box::new(collector::new()),
      1);

  let (mut executor_task, mut executor_outputs) =
    scatter::new(
      ".scheduler.executor",
      2,
      Box::new(executor::new()),
      1);

  // output 0: stopped tasks
  // output 1: looped back

  let (mut loopback_task, mut loopback_output) =
    filter::new(
      ".scheduler.loopback",
      2,
      Box::new(loop_back::new()));

  let (mut onmsg_task, mut onmsg_output) =
    filter::new(
      ".scheduler.onmsg",
      2,
      Box::new(on_msg::new()));

  let (mut timer_task, mut timer_output) =
    filter::new(
      ".scheduler.timer",
      2,
      Box::new(timer::new()));

  let mut gate_rx_opt = Some(
    IdentifiedReceiver{
      id:     new_id(String::from("Gate"), Direction::Out, 0),
      input:  gate_rx,
    }
  );

  let mut stopped : Option<IdentifiedReceiver<Box<Task+Send>>> = None;

  collector_task.connect(0, &mut gate_rx_opt).unwrap();
  collector_task.connect(1, &mut loopback_output).unwrap();
  collector_task.connect(2, &mut onmsg_output).unwrap();
  collector_task.connect(3, &mut timer_output).unwrap();

  executor_task.connect(&mut collector_output).unwrap();

  let sliced_executor_outputs = executor_outputs.as_mut_slice();
  connectable::connect_to(&mut stopped, &mut *sliced_executor_outputs[0]).unwrap();
  loopback_task.connect(&mut *sliced_executor_outputs[1]).unwrap();
  onmsg_task.connect(&mut *sliced_executor_outputs[2]).unwrap();
  timer_task.connect(&mut *sliced_executor_outputs[3]).unwrap();

  Scheduler{
    // looper     : spawn(|| { looper_entry(); }),
    gate         : gate_tx,
    collector    : collector_task,
    executor     : executor_task,
    loopback     : loopback_task,
    onmsg        : onmsg_task,
    timer        : timer_task,
    stopped      : stopped,
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
