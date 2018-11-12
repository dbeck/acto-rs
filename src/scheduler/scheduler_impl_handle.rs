
use std::sync::Arc;
use std::cell::UnsafeCell;
use scheduler::scheduler_impl;

pub struct SchedulerImplHandle {
  handle: Arc<UnsafeCell<scheduler_impl::SchedulerImpl>>,
}

impl SchedulerImplHandle {
  pub fn clone(&mut self) -> SchedulerImplHandle {
    SchedulerImplHandle{
      handle: self.handle.clone(),
    }
  }

  fn new() -> SchedulerImplHandle {
    SchedulerImplHandle{
      handle: Arc::new(UnsafeCell::new(scheduler_impl::new())),
    }
  }

  pub fn get(&mut self) -> &mut scheduler_impl::SchedulerImpl {
    unsafe { &mut (*self.handle.get()) }
  }
}

// all data access is atomic
unsafe impl Send for SchedulerImplHandle { }

pub fn new() -> SchedulerImplHandle {
  SchedulerImplHandle::new()
}
