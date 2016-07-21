extern crate lossyq;

use super::source::SourceWrap;
use super::filter::FilterWrap;
use super::sink::SinkWrap;
use super::task::{Task};
use std::thread::{spawn, JoinHandle};
use std::collections::{HashMap};

pub struct Scheduler {
  threads   : Vec<JoinHandle<i32>>,
  tasks     : HashMap<String, Box<Task>>,
  // looping Thread
  // - list, push back

  // on msg Thread
  // - map: ["name/type"] -> [ptr list]

  // scheduled Thread
  // - sorted multi map: [run_at] -> [ptr list]
}

impl Scheduler {

  pub fn add_source<Output: 'static+Copy+Send>(
      &mut self,
      task : Box<SourceWrap<Output>>)
  {
    self.tasks.insert(String::from("x"), task);
  }

  pub fn add_filter<Input: 'static+Copy+Send, Output: 'static+Copy+Send>(
      &mut self,
      task   : Box<FilterWrap<Input, Output>>)
  {
    self.tasks.insert(String::from("x"), task);
  }

  pub fn add_sink<Input: 'static+Copy+Send>(
      &mut self,
      task   : Box<SinkWrap<Input>>)
  {
    self.tasks.insert(String::from("x"), task);
  }

  // add ysplit
  // add ymerge
}

pub fn new() -> Scheduler {
  let mut ret = Scheduler{
    threads    : vec![],
    tasks      : HashMap::new(),
  };
  let t = spawn(|| {
    1 as i32
    });
  ret.threads.push(t);
  ret
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() {
  }
}
