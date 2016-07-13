
pub enum Direction {
  In,
  Out
}

pub struct Id {
  task_name  : String,
  dir        : Direction,
  id         : usize,
}

pub fn new(name: String, dir: Direction, id: usize) -> Id {
  Id {
    task_name  : name,
    dir        : dir,
    id         : id,
  }
}
