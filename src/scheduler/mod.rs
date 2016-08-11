extern crate lossyq;
extern crate time;

mod collector;
mod executor;
mod timer;
mod on_msg;
mod event;
mod loop_back;
mod lossless;

use self::lossyq::spsc::{Sender, channel};
use super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::elem::{gather, scatter, filter};
use super::connectable;

use std::thread::{spawn, JoinHandle};
//use std::collections::VecDeque;
//use std::time::{Instant, Duration};
//use std::sync::{Arc, Mutex, Condvar};

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
  // gate to add new tasks:
  gate            : Sender<Message<Box<Task+Send>>>,
  process_event   : event::Event,
  process_thread  : JoinHandle<()>,
  onmsg_event     : event::Event,
  onmsg_thread    : JoinHandle<()>,
  timer_event     : event::Event,
  timer_thread    : JoinHandle<()>,
  stopped_event   : event::Event,
  stopped_thread  : JoinHandle<()>,

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

struct CountingReporter {
  pub count : usize,
}

impl Reporter for CountingReporter {
  fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize) {
    self.count += 1;
  }
}

fn process_entry(collector : Box<Task+Send>,
                 executor : Box<Task+Send>,
                 loopback : Box<Task+Send>,
                 event    : event::Event) {
  // rebind mutable parameters
  let mut collector = collector;
  let mut executor  = executor;
  let mut loopback  = loopback;
  let mut event     = event;
  //
  let mut spin   = 0;
  let mut ticket = 0;

  loop {
    let mut reporter = CountingReporter{ count: 0 };
    collector.execute(&mut reporter);
    executor.execute(&mut reporter);
    loopback.execute(&mut reporter);
    if reporter.count == 0 {
      if spin > 100 {
        ticket = event.wait(ticket, spin);
      }
      spin += 1;
    } else {
      spin = 0;
    }
  }
}

fn timer_entry(timer : Box<Task+Send>,
               timer_event : event::Event,
               process_event : event::Event) {
  // rebind mutable parameters
  let mut timer         = timer;
  let mut timer_event   = timer_event;
  let mut process_event = process_event;
  //
  let mut ticket = 0;

  loop {
    let mut reporter = CountingReporter{ count: 0 };
    let start_ns = time::precise_time_ns();
    let result = timer.execute(&mut reporter);
    if reporter.count > 0 {
      // tell process event that the timer emitted a task
      process_event.notify();
    }
    match result {
      Schedule::Loop => {
        // continue execution
      },
      Schedule::OnMessage(_) => {
        // this is not handled
      },
      Schedule::EndPlusUSec(us) => {
        ticket = timer_event.wait(ticket, us as u64);
      },
      Schedule::StartPlusUSec(us) => {
        let diff_ns = time::precise_time_ns() - start_ns;
        let us_u64 : u64 = us as u64;
        if diff_ns/1000 < us_u64 {
          ticket = timer_event.wait(ticket, us_u64 - (diff_ns/1000));
        }
      },
      Schedule::Stop => {
        // ignore for the time being
      }
    }
  }
}

fn onmsg_entry(_onmsg : Box<Task+Send>,
               _event : event::Event) {
}

fn stopped_entry(_stopped : IdentifiedReceiver<executor::TaskResults>,
                 _event   : event::Event) {
}

pub fn new() -> Scheduler {

  use connectable::ConnectableN; // for collector
  use connectable::Connectable;  // for executor

  let (gate_tx, gate_rx) = channel(1000);

  let (mut collector_task, mut collector_output) =
    gather::new(".scheduler.collector", 1000, Box::new(collector::new()), 4);

  let (mut executor_task, mut executor_outputs) =
    scatter::new(".scheduler.executor", 1000, Box::new(executor::new()), 4);

  let (mut loopback_task, mut loopback_output) =
    filter::new(".scheduler.loopback", 1000, Box::new(loop_back::new()));

  let (mut onmsg_task, mut onmsg_output) =
    filter::new(".scheduler.onmsg", 1000, Box::new(on_msg::new()));

  let (mut timer_task, mut timer_output) =
    filter::new( ".scheduler.timer", 1000, Box::new(timer::new()));

  let mut gate_rx_opt = Some(
    IdentifiedReceiver{
      id:     new_id(String::from("Gate"), Direction::Out, 0),
      input:  gate_rx,
    }
  );

  let mut stopped : Option<IdentifiedReceiver<executor::TaskResults>> = None;

  // 0: new tasks through the gate
  // 1: loop_back
  // 2: on_msg
  // 3: timer
  collector_task.connect(0, &mut gate_rx_opt).unwrap();
  collector_task.connect(1, &mut loopback_output).unwrap();
  collector_task.connect(2, &mut onmsg_output).unwrap();
  collector_task.connect(3, &mut timer_output).unwrap();

  executor_task.connect(&mut collector_output).unwrap();
  // 0: stopped
  // 1: loop_back
  // 2: on_msg
  // 3: timer
  let sliced_executor_outputs = executor_outputs.as_mut_slice();
  connectable::connect_to(&mut stopped, &mut *sliced_executor_outputs[0]).unwrap();
  loopback_task.connect(&mut *sliced_executor_outputs[1]).unwrap();
  onmsg_task.connect(&mut *sliced_executor_outputs[2]).unwrap();
  timer_task.connect(&mut *sliced_executor_outputs[3]).unwrap();

  let process_event = event::new();
  let onmsg_event   = event::new();
  let timer_event   = event::new();
  let stopped_event = event::new();

  // cloned events
  let process_event_timer = process_event.clone();

  Scheduler{
    gate            : gate_tx,
    process_event   : process_event.clone(),
    process_thread  : spawn(move || {
      process_entry(collector_task, executor_task, loopback_task, process_event);
    }),
    onmsg_event     : onmsg_event.clone(),
    onmsg_thread    : spawn(move || {
      onmsg_entry(onmsg_task, onmsg_event);
    }),
    timer_event     : timer_event.clone(),
    timer_thread    : spawn(move || {
      timer_entry(timer_task, timer_event, process_event_timer);
    }),
    stopped_event   : stopped_event.clone() ,
    stopped_thread  : spawn(move || {
      stopped_entry(stopped.unwrap(), stopped_event);
    }),
  }
}

#[cfg(test)]
pub mod tests;
