
use std::sync::Arc;
use std::cell::UnsafeCell;
use super::data;

pub struct SchedulerDataHandle {
  handle: Arc<UnsafeCell<data::SchedulerData>>,
}

impl SchedulerDataHandle {
  pub fn clone(&mut self) -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: self.handle.clone(),
    }
  }

  fn new() -> SchedulerDataHandle {
    SchedulerDataHandle{
      handle: Arc::new(UnsafeCell::new(data::new())),
    }
  }

  pub fn get(&mut self) -> &mut data::SchedulerData {
    unsafe { &mut (*self.handle.get()) }
  }
}

// all data access is atomic
unsafe impl Send for SchedulerDataHandle { }

pub fn new() -> SchedulerDataHandle {
  SchedulerDataHandle::new()
}
