
use std::sync::atomic::{AtomicPtr, Ordering, AtomicUsize};
use super::super::{Task, SchedulingRule, TaskId, ChannelId};
use super::exec_info::{ExecInfo};
use super::prv::{Private};
use super::{wrap};
use std::ptr;

struct ExecFlags (AtomicUsize);

pub struct TaskPage {
  data:    Vec<(AtomicPtr<wrap::TaskWrap>, ExecFlags)>,
  id:      usize,
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
  }

  pub fn init_info(&mut self,
                   idx: usize,
                   output_count: usize,
                   rule: SchedulingRule)
  {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    // TODO :
    //data_ref.1.init(output_count, rule);
  }

  pub fn register_dependents(&mut self,
                             idx: usize,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[idx];
    // data_ref.1.register_dependents(deps);
  }

  pub fn eval(&mut self,
                 l2_max_idx: usize,
                 exec_id: usize,
                 private_data: &mut Private,
                 time_us: &AtomicUsize)
  {
    let mut skip    = exec_id;
    let mut l2_idx  = 0;
    //let info_slice  = self.info.as_mut_slice();
    let mut now     = time_us.load(Ordering::Acquire);

    for i in &mut self.data {
      if l2_idx >= l2_max_idx { break; }
      //if i.1.next_execution_at() <= now {
        let wrk = i.0.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
        if !wrk.is_null() {
          unsafe {
            let mut stop = false;
            (*wrk).execute(false, &mut stop);
            let end = time_us.load(Ordering::Acquire);
            now = end;
          }
          i.0.store(wrk, Ordering::Release);
        } else {
          l2_idx += skip;
          skip += exec_id;
        }
      //}
      l2_idx += 1;
    }

          /*
    loop {
      if l2_idx >= l2_max_idx { break; }


      let info_ref    = &mut info_slice[l2_idx];

      let next_at     = info_ref.next_execution_at();


      if next_at <= now {
        let wrk_ref = unsafe { self.l2.get_unchecked_mut(l2_idx) };
        let wrk = wrk_ref.swap(ptr::null_mut::<wrap::TaskWrap>(), Ordering::AcqRel);
        if wrk.is_null() == false {
          unsafe {
            let _result = (*wrk).execute();
            let end = time_us.load(Ordering::Acquire);
            now = end;
          }
          wrk_ref.store(wrk, Ordering::Release);
        } else {
          l2_idx += skip;
          skip += id;
        }
      }


      l2_idx += 1;
    }
          */
  }

  pub fn notify(&mut self, l2_idx: usize) {
    let slice = self.data.as_mut_slice();
    let data_ref = &mut slice[l2_idx];
    // TODO
    // data_ref.1.ext_notify();
  }

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {}

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new(id: usize) -> TaskPage {
  let sz               = max_idx()+1;
  let mut data         = Vec::with_capacity(sz);

  for _i in 0..sz {
    let f = ExecFlags(AtomicUsize::default());
    data.push( (AtomicPtr::default(), f) );
  }

  TaskPage{
    data:   data,
    id:     id,
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
