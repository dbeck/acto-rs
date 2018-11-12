
use std::sync::Arc;
use std::cell::UnsafeCell;
use scheduler::scheduler_impl;

pub struct SchedulerDataHandle {
  handle: Arc<UnsafeCell<scheduler_impl::SchedulerImpl>>,
}

impl SchedulerDataHandle {
  pub fn clone(&mut self) -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: self.handle.clone(),
    }
  }

  fn new() -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: Arc::new(UnsafeCell::new(scheduler_impl::new())),
    }
  }

  pub fn get(&mut self) -> &mut scheduler_impl::SchedulerImpl {
    unsafe { &mut (*self.handle.get()) }
  }
}

// all data access is atomic
unsafe impl Send for SchedulerDataHandle { }

pub fn new() -> SchedulerDataHandle {
  SchedulerDataHandle::new()
}
