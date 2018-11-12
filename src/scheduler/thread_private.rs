
use super::super::{TaskId};

pub struct ThreadPrivate {
  to_trigger: Vec<TaskId>,
}

impl ThreadPrivate {
  pub fn new() -> ThreadPrivate {
    ThreadPrivate {
      to_trigger: Vec::with_capacity(10),
    }
  }

  pub fn ensure_size(&mut self, _size: usize) {
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
  }
}
