
#[derive(Copy,Clone,Debug,PartialEq)]
pub struct TaskId {
  id: usize,
}

impl TaskId {
  pub fn id(&self) -> usize {
    self.id
  }
}

pub fn new(id: usize) -> TaskId {
  TaskId{ id: id }
}
