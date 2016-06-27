extern crate lossyq;

use std::vec::Vec;
use self::lossyq::spsc::{Sender,channel};

trait SenderWrap {
  fn fun(&mut self, val: i32) -> ();
}

struct Item {
  q : Box<SenderWrap>
}

impl SenderWrap for Sender<i32> {
  fn fun(&mut self, val: i32) -> () {
    let v = self as &mut Sender<i32>;
    v.put(|x| *x = val);
  }
}

pub struct Scheduler {
  items : Vec<Item>
}

impl Scheduler {

  // start/add thread
  // "start" loop()

  // stop() "all"

  // spawn "runnable" <- worker/supervisor
  //  -- add(name,runnable)

  // send msg(name[], msg)

  // str???

  pub fn add_queue(&mut self, _name: &str, size: usize) {
    let (tx, mut _rx) = channel(size, 0 as i32);
    // let val = Box::new(&mut Item{q: tx} as &mut Valami);
    //self.items = Box::new(&mut Item{q: tx} as &mut Valami);
    // self.items.push(val); //.append(val as &mut Valami)
    self.items.push(Item{q: Box::new(tx)});
  }

  pub fn broadcast(&mut self, val: i32) {
    let slice = self.items.as_mut_slice();
    for i in slice {
      i.q.fun(val);
    }
  }
}

pub fn new() -> Scheduler {
  Scheduler{
    items : vec![]
  }
}


// -- TODO : test interface here --
pub fn remove_me() {
  let mut s = new();
  s.add_queue("hello", 10);
  s.add_queue("hello", 10);
  s.add_queue("hello", 10);
  s.broadcast(1);
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() {
    remove_me();
  }
}
