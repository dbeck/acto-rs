use std::fmt;

#[derive(Copy,Clone,Debug)]
pub enum Direction {
  In,
  Out
}

#[derive(Clone,Debug)]
pub struct Id {
  task_name  : String,
  dir        : Direction,
  id         : usize,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id:({} {:?} {})", self.task_name, self.dir, self.id)
    }
}

pub fn new(name: String, dir: Direction, id: usize) -> Id {
  Id {
    task_name  : name,
    dir        : dir,
    id         : id,
  }
}
