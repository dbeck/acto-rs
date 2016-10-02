
use std::collections::{HashMap};
use std::sync::atomic::{AtomicUsize, AtomicBool, AtomicPtr, Ordering};
use super::super::{Task, Error, TaskState, TaskId,
  ReceiverChannelId, ChannelId, SchedulingRule
};
use super::{page, prv};
use super::observer::{TaskObserver};
use super::event;
use parking_lot::{Mutex};
use std::ptr;
use std::time::{Instant};
use libc;

pub struct SchedulerData {
  // ticker only:
  start:       Instant,
  // shared between threads
  // everything below has to be thread safe:
  max_id:      AtomicUsize,
  l1:          Vec<AtomicPtr<page::TaskPage>>,
  stop:        AtomicBool,
  time_us:     AtomicUsize,
  ids:         Mutex<HashMap<String, usize>>,
  unresolved:  Mutex<HashMap<String, HashMap<TaskId,Vec<ChannelId>>>>,
  evt:         event::Event,
}

impl SchedulerData {
  fn add_l2_page(&mut self, idx: usize) {
    let array = Box::new(page::new());
    let len = self.l1.len();
    if idx >= len-1 {
      // extend slice
      for _i in 0..initial_capacity() {
        self.l1.push(AtomicPtr::default());
      }
    }
    let l1_slice = self.l1.as_mut_slice();
    l1_slice[idx].store(Box::into_raw(array), Ordering::Release);
  }

  fn new() -> SchedulerData {
    let l1_size = initial_capacity();
    let mut data = SchedulerData{
      start:       Instant::now(),
      max_id:      AtomicUsize::new(0),
      l1:          Vec::with_capacity(l1_size),
      stop:        AtomicBool::new(false),
      time_us:     AtomicUsize::new(0),
      ids:         Mutex::new(HashMap::new()),
      unresolved:  Mutex::new(HashMap::new()),
      evt:         event::new(),
    };

    // fill the l1 bucket
    for _i in 0..l1_size {
      data.l1.push(AtomicPtr::default());
    }

    // add an initial l2 page
    data.add_l2_page(0);
    data
  }

  pub fn add_task(&mut self, task: Box<Task+Send>, _rule: SchedulingRule) -> Result<TaskId, Error> {
    let ret_id : usize;
    let input_count = task.input_count();
    let mut input_task_ids : Vec<Option<usize>> = Vec::with_capacity(input_count);

    // check if name exists, and register if not. also register
    // unresolved sender ids if any.
    {
      let mut ids = self.ids.lock();
      if ids.contains_key(task.name()) {
        return Result::Err(Error::AlreadyExists);
      } else {
        ret_id = self.max_id.fetch_add(1, Ordering::AcqRel);
        ids.insert(task.name().clone(), ret_id);

        // resolve input task ids
        for i in 0..input_count {
          match task.input_id(ReceiverChannelId(i)) {
            Some(ref ch_id_sender_name) => {
              let ref ch_id_id   = ch_id_sender_name.0;
              let ref ch_id_name = ch_id_sender_name.1;
              match ids.get(&ch_id_name.0) {
                Some(&id)  => input_task_ids.push(Some(id)),
                None       => {
                  let mut unresolved = self.unresolved.lock();
                  // register that this task needs the task id of the sender
                  // - based on the sender name and chanel id
                  let dependents = unresolved.entry(ch_id_name.0.clone()).or_insert(HashMap::new());
                  let channels = dependents.entry(TaskId(ret_id)).or_insert(Vec::new());
                  channels.push(*ch_id_id);
                  input_task_ids.push(None)
                }
              }
            },
            _ => input_task_ids.push(None)
          }
        }
      }
    }

    let task_name = task.name().clone();
    {
      // store task
      let (l1, l2) = page::position(ret_id);
      if l2 == 0 {
        // make sure the next bucket exists when needed
        self.add_l2_page(l1+1);
      }
      unsafe {
        let l1_ptr = self.l1.get_unchecked_mut(l1).load(Ordering::Acquire);
        if l1_ptr.is_null() == false {
          (*l1_ptr).store(l2, task, TaskId(ret_id), input_task_ids);
        }
      }
    }

    {
      // resolved ids if any, for other tasks
      let mut unresolved = self.unresolved.lock();
      let mut remove = false;
      if let Some(dependents) = unresolved.get(&task_name) {
        remove = true;
        for (dep_id, channels) in dependents.iter() {
          for ch in channels.iter() {
            // resolve input id
            /* XXX
            self.apply_page(dep_id.0, |idx, page| {
              unsafe { (*page).resolve_input_task_id(idx, &TaskId(ret_id), &ch) };
            });
            */
          }
        }
      }
      if remove {
        unresolved.remove(&task_name);
      }
    }
    Result::Ok(TaskId(ret_id))
  }

  pub fn ticker(&mut self) {
    let mut last_event_at = 0;
    loop {
      unsafe { libc::usleep(10); }
      let diff = self.start.elapsed();
      let diff_us = diff.as_secs() as usize * 1000_000 + diff.subsec_nanos() as usize / 1000;
      self.time_us.store(diff_us, Ordering::Release);
      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
      // tick evt every second
      if diff_us - last_event_at > 1_000_000 {
        last_event_at = diff_us;
        self.evt.notify();
      }
    }
  }

  fn post_process_tasks(&mut self, observer: &TaskObserver) {
    // process msg wait dependencies
    /* XXX
    for w in observer.msg_waits() {
      let &(task_id, state) = w;
      match state {
        TaskState::MessageWait(sender_id, channel_id) => {
          // register dependencies
          self.apply_page(sender_id.0, |idx, page| {
            unsafe { (*page).register_dependent(idx, channel_id, task_id) };
          });
        }
        _ => {},
      }
    }
    */

    /* XXX
    {
      let to_trigger : &Vec<TaskId> = observer.msg_triggers();
      if to_trigger.len() > 0 {
        for &t_task_id in observer.msg_triggers() {
          self.msg_trigger(t_task_id);
        }
      }
    }
    */
  }

  pub fn entry(&mut self, id: usize) {
    //let mut pp_time = 0u64;
    //let mut pp_count = 0u64;

    let start = Instant::now();
    let mut no_exec = 0u64;
    let mut exec = 0u64;
    let mut iter = 0u64;

    let mut private_data = prv::Private::new();

    let l2_max = page::max_idx();
    loop {
      let max_id = self.max_id.load(Ordering::Acquire);
      private_data.ensure_size(max_id);
      
      let mut reporter = TaskObserver::new(max_id);
      let (l1, l2) = page::position(max_id);
      {
        let l1_slice = self.l1.as_mut_slice();

        // go through all fully filled l2 buckets
        let mut l2_max_idx = l2_max;
        for l1_idx in 0..l1 {
          let l1_ptr = l1_slice[l1_idx].load(Ordering::Acquire);
          unsafe {
            (*l1_ptr).eval(l2_max_idx, id, &mut reporter, &self.time_us);
          }
        }

        // take care of the last, partially filled bucket
        l2_max_idx = l2;
        for l1_idx in l1..(l1+1) {
          let l1_ptr = l1_slice[l1_idx].load(Ordering::Acquire);
          unsafe {
            (*l1_ptr).eval(l2_max_idx, id, &mut reporter, &self.time_us);
          }
        }
      }

      self.post_process_tasks(&reporter);

      if reporter.exec_count() == 0 {
        no_exec += 1;
      } else {
        exec += 1;
      }

      iter += 1;

      // check stop state
      if self.stop.load(Ordering::Acquire) {
        break;
      }
    }

    let diff = start.elapsed();
    let diff_ns = diff.as_secs() * 1_000_000_000 + diff.subsec_nanos() as u64;
    let ns_iter = diff_ns/iter;

    println!("loop_count: {} {} ns/iter :: no_exec: {}  exec_loop: {}",
      iter,
      ns_iter,
      no_exec,
      exec
    );
  }

  /* XXX !!!
  fn msg_trigger(&self, task_id: TaskId)  {
    if task_id.0 < self.max_id.load(Ordering::Acquire) {
      self.apply_page(task_id.0, |idx, page| {
        unsafe { (*page).msg_trigger(idx) };
      });
    }
  }
  */

  pub fn notify(&mut self, id: &TaskId) -> Result<usize, Error> {
    if self.stop.load(Ordering::Acquire) {
      return Result::Err(Error::Stopping);
    }
    let max = self.max_id.load(Ordering::Acquire);
    if id.0 >= max {
      return Result::Err(Error::NonExistent);
    }
    let (l1, l2) = page::position(id.0);
    let l1_slice = self.l1.as_mut_slice();
    let l1_ptr = l1_slice[l1].load(Ordering::Acquire);
    if l1_ptr.is_null() {
      return Result::Err(Error::NonExistent);
    }
    unsafe { Ok((*l1_ptr).notify(l2)) }
  }

  pub fn stop(&mut self) {
    self.stop.store(true, Ordering::Release);
  }

  fn apply_page<F>(&self, task_id: usize, mut fun: F)
    where F : FnMut(usize, *mut page::TaskPage)
  {
    let (l1, l2) = page::position(task_id);
    let l1_slice = self.l1.as_slice();
    let l1_ptr = l1_slice[l1].load(Ordering::Acquire);
    //let l1_ptr = self.l1.get_unchecked(l1).load(Ordering::Acquire);
    if l1_ptr.is_null() == false {
      fun(l2, l1_ptr);
    }
  }

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {}

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new() -> SchedulerData {
  SchedulerData::new()
}

pub fn initial_capacity() -> usize {
  1024*1024
}

impl Drop for SchedulerData {
  fn drop(&mut self) {
    self.print_stats();
    let len = self.l1.len();
    let l1_slice = self.l1.as_mut_slice();
    for i in 0..len {
      let l1_atomic_ptr = &mut l1_slice[i];
      let ptr = l1_atomic_ptr.swap(ptr::null_mut::<page::TaskPage>(), Ordering::AcqRel);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      } else {
        break;
      }
    }
  }
}
