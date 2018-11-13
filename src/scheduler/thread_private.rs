
use super::super::{TaskId};

pub struct ThreadPrivate {
  to_trigger: Vec<TaskId>,
  next_wakeup: u64,
}

impl ThreadPrivate {
  pub fn new() -> ThreadPrivate {
    ThreadPrivate {
      to_trigger: Vec::with_capacity(10),
      next_wakeup: 0xffff_ffff_ffff_ffff,
    }
  }

  #[inline]
  pub fn save_trigger(&mut self,
                      to: TaskId)
  {
    self.to_trigger.push(to);
  }

  pub fn to_trigger(&self) -> &Vec<TaskId> {
    &self.to_trigger
  }

  pub fn clear(&mut self) {
    self.to_trigger.clear();
    self.next_wakeup = 0xffff_ffff_ffff_ffff;
  }

  pub fn adjust_wakeup(&mut self, val: u64) {
    if self.next_wakeup > val {
      self.next_wakeup = val;
    }
  }

  pub fn get_wakeup(&self) -> u64 {
    self.next_wakeup
  }
}
