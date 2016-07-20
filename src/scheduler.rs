extern crate lossyq;

use super::source::SourceWrap;
use super::filter::FilterWrap;
use super::sink::SinkWrap;
use super::channel_id;
use super::task::{Task};
use std::thread::{spawn, JoinHandle};
use std::any::{Any};
use std::collections::{HashMap};

pub struct Scheduler {
  threads   : Vec<JoinHandle<i32>>,
  tasks     : HashMap<String, Box<Task>>,
  receivers : HashMap<String, Box<Any>>,
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
      task : Box<SourceWrap<Output>>,
      output : Box<Any>)
  {
    self.tasks.insert(String::from("x"), task);
    self.receivers.insert(String::from("x"), output);
  }

  pub fn add_filter<Input: 'static+Copy+Send, Output: 'static+Copy+Send>(
      &mut self,
      task   : Box<FilterWrap<Input, Output>>,
      output : Box<Any>,
      _input  : channel_id::Id)
  {
    self.tasks.insert(String::from("x"), task);
    self.receivers.insert(String::from("x"), output);
  }

  pub fn add_sink<Input: 'static+Copy+Send>(
      &mut self,
      task   : Box<SinkWrap<Input>>,
      _input  : channel_id::Id)
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
    receivers  : HashMap::new(),
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
