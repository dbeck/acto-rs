
use super::super::{TaskId};

pub struct Private {
}

impl Private {
  pub fn new() -> Private {
    Private {
    }
  }

  pub fn ensure_size(&mut self, _size: usize) {
  }

  pub fn save_notification(&mut self,
                           _to: TaskId)
  {
  }
}
