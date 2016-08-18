pub mod collector; // TODO: make non public later!!
mod executor;
mod timer;
mod on_msg;
mod event;
pub mod loop_back; // TODO: make non public later!!

use time;
use lossyq::spsc::{Sender, channel};
use super::common::{Task, Reporter, Message, Schedule, IdentifiedReceiver, Direction, new_id};
use super::elem::{gather, scatter, filter};
use super::connectable;

//use std::sync::{Arc};
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering, AtomicPtr};
use std::thread::{spawn, JoinHandle};

struct TaskWrap {
  task: Box<Task+Send>,
}

#[allow(dead_code)]
struct TaskArray {
  l2: Vec<AtomicPtr<TaskWrap>>,
}

#[allow(dead_code)]
pub struct Scheduler {
  max_id: AtomicUsize,
  l1: Vec<AtomicPtr<TaskArray>>,
}

// L1: 64k entries preallocated
// L2: 4k entries on-demand
impl Scheduler {

  fn add_l2_bucket(&mut self, idx: usize) {
    let l1_slice = self.l1.as_mut_slice();
    let mut bucket = Vec::with_capacity(4096);
    for _i in 0..(4096) {
      bucket.push(AtomicPtr::default());
    }
    let array = Box::new(TaskArray{ l2: bucket });
    l1_slice[idx].store(Box::into_raw(array), Ordering::SeqCst);
  }

  fn position(&self, idx: usize) -> (usize, usize) {
    (idx>>12, idx&0xfff)
  }

  pub fn add_task(&mut self, task: Box<Task+Send>) {
    let (l1, l2) = self.position(self.max_id.fetch_add(1, Ordering::SeqCst));
    if l2 == 0 {
      // make sure the next bucket exists when needed
      self.add_l2_bucket(l1+1);
    }
    //if l1 == 65534 {
      // TODO:
    //}
    unsafe {
      let l1_ptr = self.l1.get_unchecked_mut(l1).load(Ordering::SeqCst);
      if l1_ptr.is_null() == false {
        let wrap = Box::new(TaskWrap{task: task});
        let l2_atomic_ptr = (*l1_ptr).l2.get_unchecked_mut(l2);
        l2_atomic_ptr.store(Box::into_raw(wrap),Ordering::SeqCst);
      }
    }
  }

  pub fn start(&mut self) {
    for l1i in &mut self.l1 {
      let l1_ptr = l1i.load(Ordering::SeqCst);
      if l1_ptr.is_null() {
        break;
      } else {
        unsafe {
          let l2_vec = &mut (*l1_ptr).l2;
          for l2i in l2_vec {
            let wrk = l2i.swap(ptr::null_mut::<TaskWrap>(), Ordering::SeqCst);
            if wrk.is_null() == false {
              let mut reporter = CountingReporter{ count: 0 };
              (*wrk).task.execute(&mut reporter);
              l2i.swap(wrk, Ordering::SeqCst);
            }
          }
        }
      }
    }
  }
}

pub fn new() -> Scheduler {
  let mut ret = Scheduler{
    max_id: AtomicUsize::new(0),
    l1: Vec::with_capacity(65536),
  };
  // fill the l1 bucket
  for _i in 0..(64*1024) {
    ret.l1.push(AtomicPtr::default());
  }
  // add an initial l2 bucket
  ret.add_l2_bucket(0);
  ret
}

//////////////////////////////////////////////////////
// Old/Slow Implementation Below
//////////////////////////////////////////////////////

pub struct SchedulerOld {
  // gate to add new tasks:
  gate            : Sender<Message<Box<Task+Send>>>,
  process_event   : event::Event,
  process_thread  : JoinHandle<()>,
  //onmsg_event     : event::Event,
  onmsg_thread    : JoinHandle<()>,
  //timer_event     : event::Event,
  timer_thread    : JoinHandle<()>,
  //stopped_event   : event::Event,
  stopped_thread  : JoinHandle<()>,
}

impl SchedulerOld {

  pub fn add_task(&mut self, task: Box<Task+Send>)
  {
    use std::mem;
    let mut to_send : Option<Message<Box<Task+Send>>> = Some(Message::Value(task));
    self.gate.put( |v| mem::swap(&mut to_send, v) );
    // TODO : handle overflow here
    self.process_event.notify();
  }

  pub fn stop(self) {
    self.process_thread.join().unwrap();
    self.onmsg_thread.join().unwrap();
    self.timer_thread.join().unwrap();
    self.stopped_thread.join().unwrap();
  }
}

pub struct CountingReporter {
  pub count : usize,
}

impl Reporter for CountingReporter {
  fn message_sent(&mut self, _channel_id: usize, _last_msg_id: usize) {
    self.count += 1;
  }
}

pub struct MeasureTime {
  time_diff: u64,
  last: u64,
  count: u64,
}

impl MeasureTime {
  pub fn new() -> MeasureTime { MeasureTime{time_diff: 0, last: 0, count: 0} }
  pub fn start(&mut self) {
    self.last = time::precise_time_ns();
  }
  pub fn end(&mut self) {
    self.count += 1;
    self.time_diff += time::precise_time_ns() - self.last;
  }
  pub fn print(&mut self, prefix: &'static str) {
    if self.count % 1000000 == 0 {
      println!("{} diff t: {} {} {}/sec",prefix ,self.time_diff/self.count,self.count,1_000_000_000/(self.time_diff/self.count));
    }
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

  let mut m_c = MeasureTime::new();
  let mut m_e = MeasureTime::new();
  let mut m_l = MeasureTime::new();
  let mut m_x = MeasureTime::new();

  loop {
    let mut reporter = CountingReporter{ count: 0 };
    m_x.start();
    m_c.start();
    collector.execute(&mut reporter);
    m_c.end();
    m_e.start();
    executor.execute(&mut reporter);
    m_e.end();
    m_l.start();
    loopback.execute(&mut reporter);
    m_l.end();
    m_x.end();
    m_c.print("Collector thread: collector");
    m_e.print("Collector thread:  executor");
    m_l.print("Collector thread: loop_back");
    m_x.print("Collector thread: all");
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

pub fn new_old() -> SchedulerOld {

  use connectable::ConnectableN; // for collector
  use connectable::Connectable;  // for executor

  let (gate_tx, gate_rx) = channel(10000);

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

  SchedulerOld{
    gate            : gate_tx,
    process_event   : process_event.clone(),
    process_thread  : spawn(move || {
      process_entry(collector_task, executor_task, loopback_task, process_event);
    }),
    //onmsg_event     : onmsg_event.clone(),
    onmsg_thread    : spawn(move || {
      onmsg_entry(onmsg_task, onmsg_event);
    }),
    //timer_event     : timer_event.clone(),
    timer_thread    : spawn(move || {
      timer_entry(timer_task, timer_event, process_event_timer);
    }),
    //stopped_event   : stopped_event.clone() ,
    stopped_thread  : spawn(move || {
      stopped_entry(stopped.unwrap(), stopped_event);
    }),
  }
}

#[cfg(test)]
pub mod tests;
