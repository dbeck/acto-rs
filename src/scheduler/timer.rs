extern crate lossyq;

use self::lossyq::spsc::{Sender, Receiver};
use super::super::common::{Task, Message};
use super::super::elem::filter::Filter;
use super::super::common::Schedule;
use super::executor::TaskResults;

//use std::collections::BinaryHeap;
//use std::time::{Instant};
//use std::cmp::{PartialOrd, PartialEq, Eq, Ordering};
//use std::collections::VecDeque;
//use std::mem;

pub struct Timer {
  // dummyx: usize,
}

impl Filter for Timer {
  type InputType  = TaskResults;
  type OutputType = Box<Task + Send>;

  fn process(
          &mut self,
          _input:   &mut Receiver<Message<Self::InputType>>,
          _output:  &mut Sender<Message<Self::OutputType>>) -> Schedule {
    Schedule::Loop
  }
}

pub fn new() -> Timer {
  Timer {
    // dummyx: 0,
  }
}

// add task:
// - task
// - at micro sec

// current time micro sec:
// ->

// callback:
// - add to looped tasks

/*
struct Item {
  at   : Instant,
  task : Box<Task>,
}

impl Ord for Item {
  fn cmp(&self, other:&Self) -> Ordering {
    if self.at < other.at {
      Ordering::Less
    } else if self.at > other.at {
      Ordering::Greater
    } else {
      Ordering::Equal
    }
  }
}

impl PartialOrd for Item {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
      Some(self.cmp(other))
  }
}

impl PartialEq for Item {
  fn eq(&self, other: &Self) -> bool {
    if self.at == other.at {
      true
    } else {
      false
    }
  }
}

impl Eq for Item { }

pub struct TimeTriggered {
  items : BinaryHeap<Item>,
}

impl TimeTriggered {
  pub fn add(&mut self, at: Instant, task: Box<Task>) {
    self.items.push(Item{at: at, task: task});
  }
}
*/

/*
pub fn new() -> TimeTriggered {
  TimeTriggered {
    items : BinaryHeap::new(),
  }
}
*/
