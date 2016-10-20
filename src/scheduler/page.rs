
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, ChannelId, TaskId, PeriodLengthInUsec};
use super::prv::{Private};
use super::{wrap};
use std::ptr;

struct ExecFlags (AtomicUsize);

pub struct TaskPage {
  data:    Vec<(AtomicPtr<wrap::TaskWrap>, ExecFlags, PeriodLengthInUsec)>,
}

pub fn max_idx() -> usize {
  // note: this must be aligned with position(idx)
  4095
}

pub fn position(idx: usize) -> (usize, usize) {
  // note: this depends on max_idx !!!
  (idx>>12, idx&0xfff)
}

impl TaskPage {
  pub fn store(&mut self,
               idx: usize,
               task: Box<Task+Send>)
  {
    let wrap = Box::new(wrap::new(task));
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let old = data_ref.0.swap(Box::into_raw(wrap), Ordering::AcqRel);
    if old.is_null() == false {
      // make sure we drop old pointers when swapped, although
      // this shouldn't happen since the SchedulerData must take care
      // of atomically increasing indices
      let _b = unsafe { Box::from_raw(old) };
    }
    // clear the stopped flag too
    (data_ref.1).0.fetch_and(63-16, Ordering::AcqRel);
  }

  pub fn set_dependents_flag(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    (data_ref.1).0.fetch_or(1, Ordering::Release);
  }

  pub fn set_delayed_exec(&mut self, idx: usize, period: PeriodLengthInUsec) {
    let slice = self.data.as_mut_slice();
    let mut data_ref = &mut slice[idx];
    (data_ref.1).0.fetch_or(4, Ordering::Release);
    data_ref.2 = period;
  }

  pub fn set_conditional_exec_flag(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    (data_ref.1).0.fetch_or(32, Ordering::Release);
  }

  pub fn schedule_exec(&mut self, idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    // clear exec time
    (data_ref.1).0.fetch_and(63, Ordering::Acquire);
  }

  // has_dependents: 1
  // ???: 2
  // delayed: 4
  // ???: 8
  // stopped-flag: 16
  // conditiona: 32

  pub fn register_dependents(&mut self,
                             idx: usize,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    let atomic_flags = &mut data_ref.1;
    let delay_exec : u64 = 0xffffffffffffff << 6;
    let flags = atomic_flags.0.fetch_or(delay_exec as usize, Ordering::Acquire);
    loop {
      let wrk = data_ref.0.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if !wrk.is_null() {
        unsafe { (*wrk).register_dependents(deps); }
        data_ref.0.store(wrk, Ordering::Release);
        atomic_flags.0.store(flags, Ordering::Release);
        break;
      }
      atomic_flags.0.fetch_or(delay_exec as usize, Ordering::Acquire);
    }
  }

  #[inline(always)]
  pub fn eval(&mut self,
              l2_max_idx: usize,
              exec_thread_id: usize,
              private_data: &mut Private,
              time_us: &AtomicUsize)
  {
    let mut skip    = exec_thread_id;
    let mut l2_idx  = 0;
    let mut now     = time_us.load(Ordering::Acquire);

    for act_data in &mut self.data {
      if l2_idx >= l2_max_idx { break; }
      let flags = (act_data.1).0.load(Ordering::Acquire);
      let stopped = flags&16;
      // execute if not stopped and time is OK
      if stopped == 0 {
        let next_execution_at = flags >> 6;
        if next_execution_at <= now {
          let wrk = act_data.0.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
          if !wrk.is_null() {

            let mut stop = false;
            unsafe {
              // flags&1 is the dependents flag
              (*wrk).execute(flags&1 == 1, &mut stop, private_data);
            }

            let atomic_flags = &mut (act_data.1).0;
            let end = time_us.load(Ordering::Acquire);

            if stop {
              // the task said to be stopped, so set the stop bit 
              atomic_flags.fetch_or(16, Ordering::Release);
            } else if flags&32 == 32 {
              // for conditionally executed tasks that:
              // 1, wait for external notification
              // 2, wait for message
              // -> set exec time to 10s ahead
              // -> add back original flags
              let new_flags : usize = (end+10_000_000)<<6 | (flags&63);
              atomic_flags.store(new_flags, Ordering::Release);
            } else if flags&4 == 4 {
              // flags&4 is the delay flag. the third component of the
              // data/act_data is the delay amount: i.e. i.2
              let new_flags : usize = (now+(act_data.2).0)<<6 | (flags&63);
              atomic_flags.store(new_flags, Ordering::Release);
            }
            now = end;
            act_data.0.store(wrk, Ordering::Release);
          } else {
            l2_idx += skip;
            skip += exec_thread_id;
          }
        }
      }
      l2_idx += 1;
    }
  }

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {
    /*
    let mut pos = 0;
    for i in &self.data {
      let ptr = i.0.load(Ordering::Acquire);
      if !ptr.is_null() {
        let flags = (i.1).0.load(Ordering::Acquire);
        let has_dependents = flags&1 == 1;
        let delayed = flags&4 == 4;
        let stopped = flags&16 == 16;
        let conditional = flags&32 == 32;
        let next_execution_at = flags>>6;
        println!("#{} has-dep:{:?} delayed:{:?}/{:?} stop:{:?} cond:{:?} next:{}",
          pos, has_dependents, delayed, (i.2).0,
          stopped, conditional,
          next_execution_at);
      }
      pos += 1;
    }
    */
  }

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new(_id: usize) -> TaskPage {
  let sz               = max_idx()+1;
  let mut data         = Vec::with_capacity(sz);

  for _i in 0..sz {
    // default flag is stopped: 16
    let f = ExecFlags(AtomicUsize::new(16));
    data.push( (AtomicPtr::default(), f, PeriodLengthInUsec(0)) );
  }

  TaskPage{
    data:   data,
  }
}

impl Drop for TaskPage {
  fn drop(&mut self) {
    self.print_stats();
    let slice = self.data.as_mut_slice();
    for i in 0..(1+max_idx()) {
      let data_ref = &mut slice[i];
      let ptr = data_ref.0.swap(
        ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
      if ptr.is_null() == false {
        // make sure we drop the pointers
        let _b = unsafe { Box::from_raw(ptr) };
      }
    }
  }
}
