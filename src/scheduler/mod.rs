mod event;

use time;
//use lossyq::spsc::{Sender, channel};
use super::common::{Task, Reporter /*, Message, Schedule, IdentifiedReceiver, Direction, new_id*/ };
//use super::elem::{gather, scatter, filter};
//use super::connectable;


use std::ptr;
use std::cell::{UnsafeCell};
use std::sync::{Arc};
use std::sync::atomic::{AtomicUsize, Ordering, AtomicPtr, AtomicBool};
use std::thread::{spawn, JoinHandle};

struct TaskWrap {
  task: Box<Task+Send>,
}

struct TaskArray {
  l2: Vec<AtomicPtr<TaskWrap>>,
}

#[allow(dead_code)]
struct SchedulerData {
  max_id:   AtomicUsize,
  l1:       Vec<AtomicPtr<TaskArray>>,
  stop:     AtomicBool,
}

impl SchedulerData {
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

  fn new() -> SchedulerData {
    let mut data = SchedulerData{
      max_id:   AtomicUsize::new(0),
      l1:       Vec::with_capacity(65536),
      stop:     AtomicBool::new(false),
    };

    // fill the l1 bucket
    for _i in 0..(64*1024) {
      data.l1.push(AtomicPtr::default());
    }

    // add an initial l2 bucket
    data.add_l2_bucket(0);
    data
  }

  fn add_task(&mut self, task: Box<Task+Send>) {
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

  #[no_mangle]
  fn start_test(&mut self) {
    let mut reporter = CountingReporter{ count: 0 };
    let (l1, l2) = self.position(self.max_id.load(Ordering::SeqCst));
    let l1_slice = self.l1.as_mut_slice();
    for l1_idx in 0..(l1+1) {
      let l1_ptr = l1_slice[l1_idx].load(Ordering::SeqCst);
      let l2_vec = unsafe { &mut (*l1_ptr).l2 };
      let l2_slice = l2_vec.as_mut_slice();
      let mut l2_max_idx = 4095;
      if l1_idx == l1 {
        l2_max_idx = l2;
      }
      for l2idx in 0..(l2_max_idx+1) {
        let wrk = l2_slice[l2idx].swap(ptr::null_mut::<TaskWrap>(), Ordering::SeqCst);
        if wrk.is_null() == false {
          unsafe { (*wrk).task.execute(&mut reporter); }
          l2_slice[l2idx].store(wrk, Ordering::SeqCst);
        }
      }
    }
  }

  fn entry(&mut self, id: usize) {
    let mut exec_count : u64 = 0;
    let start = time::precise_time_ns();
    loop {
      if self.stop.load(Ordering::SeqCst) {
        break;
      }
      let (l1, l2) = self.position(self.max_id.load(Ordering::SeqCst));
      let l1_slice = self.l1.as_mut_slice();
      for l1_idx in 0..(l1+1) {
        let l1_ptr = l1_slice[l1_idx].load(Ordering::SeqCst);
        let l2_vec = unsafe { &mut (*l1_ptr).l2 };
        let l2_slice = l2_vec.as_mut_slice();
        let mut l2_max_idx = 4095;
        if l1_idx == l1 {
          l2_max_idx = l2;
        }
        let mut skip = id;
        let mut l2idx = 0;
        loop {
          if l2idx > l2_max_idx { break; }
          let wrk = l2_slice[l2idx].swap(ptr::null_mut::<TaskWrap>(), Ordering::SeqCst);
          if wrk.is_null() == false {
            let mut reporter = CountingReporter{ count: 0 };
            unsafe { (*wrk).task.execute(&mut reporter); }
            l2_slice[l2idx].store(wrk, Ordering::SeqCst);
            exec_count += 1;
            l2idx += 1;
          } else {
            l2idx += skip+1;
            skip += 1;
          }
        }
      }
    }
    let end = time::precise_time_ns();
    let diff = end - start;
    println!("thread: #{} exiting. #exec: {} exec-time: {} ns", id, exec_count, diff/exec_count);
  }

  fn stop(&mut self) {
    self.stop.store(true, Ordering::SeqCst);
  }
}

struct SchedulerDataHandle {
  handle: Arc<UnsafeCell<SchedulerData>>,
}

impl SchedulerDataHandle {
  fn clone(&mut self) -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: self.handle.clone(),
    }
  }

  fn new() -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: Arc::new(UnsafeCell::new(SchedulerData::new())),
    }
  }

  fn get(&mut self) -> &mut SchedulerData {
    unsafe { &mut (*self.handle.get()) }
  }
}

// all data access is atomic
unsafe impl Send for SchedulerDataHandle { }

#[allow(dead_code)]
pub struct Scheduler {
  data:     SchedulerDataHandle,
  threads:  Vec<JoinHandle<()>>,
}

// L1: 64k entries preallocated
// L2: 4k entries on-demand
impl Scheduler {

  pub fn add_task(&mut self, task: Box<Task+Send>) {
    (*self.data.get()).add_task(task);
  }

  pub fn start(&mut self) {
    self.start_with_threads(1);
  }

  pub fn start_with_threads(&mut self, n_threads: usize) {
    if n_threads == 0 {
      return;
    }
    for i in 0..n_threads {
      let mut handle = self.data.clone();
      let t = spawn(move || { handle.get().entry(i); });
      self.threads.push(t);
    }
  }

  pub fn stop(&mut self) {
    (*self.data.get()).stop();
    loop {
      match self.threads.pop() {
        Some(t) => {
          t.join().unwrap();
        },
        None => {
          break;
        }
      }
    }
  }

  pub fn start_test(&mut self) {
    (*self.data.get()).start_test();
  }
}

pub fn new() -> Scheduler {
  Scheduler{
    data:     SchedulerDataHandle::new(),
    threads:  Vec::new(),
  }
}

//////////////////////////////////////////////////////
// Old/Slow Implementation Below
//////////////////////////////////////////////////////

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

#[cfg(test)]
pub mod tests;
