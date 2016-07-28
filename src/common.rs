extern crate lossyq;
use self::lossyq::spsc::Receiver;
use std::fmt;

#[derive(Copy,Clone,Debug)]
pub enum Message<T: Copy+Send>
{
  Empty,                      //
  Value(T),                   //
  Ack(usize,usize),           // from-to
  Error(usize,&'static str),  // error at
}

#[derive(Debug)]
pub enum Schedule {
  Loop,
  OnMessage(usize),
  EndPlusUSec(usize),
  StartPlusUSec(usize),
  Stop,
}

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

pub struct IdentifiedReceiver<Input: Copy+Send> {
  pub id    : Id,
  pub input : Receiver<Message<Input>>,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id:({} {:?} {})", self.task_name, self.dir, self.id)
    }
}

pub fn new_id(name: String, dir: Direction, id: usize) -> Id {
  Id {
    task_name  : name,
    dir        : dir,
    id         : id,
  }
}
