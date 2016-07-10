extern crate minions;
extern crate lossyq;

use minions::{scheduler, filter};
use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::common;
use minions::common::Message;

#[derive(Copy, Clone)]
struct FilterState {
  state : i32,
}

impl filter::Filter for FilterState {
  type InputType = i32;
  type OutputType = i32;

  fn process(
    &mut self,
    input: &mut Receiver<Message<Self::InputType>>,
    output: &mut Sender<Message<Self::OutputType>>) -> common::Schedule {
      for i in input.iter() {
        match i {
          Message::Value(v) => {
            output.put(|x| *x = Message::Value(v));
          }
          _ => { println!("Unknown value"); }
        }
      }
      common::Schedule::Loop
  }
}

fn main() {
  let fl: Box<filter::Filter<InputType=i32,OutputType=i32>> = Box::new(FilterState{state:0});

  let (mut ww, mut req_tx, mut rep_rx) = filter::new( String::from("Hello"), 2, 2, fl);

  req_tx.put(|v| *v = Message::Value(1));
  req_tx.put(|v| *v = Message::Value(2));
  ww.process();
  for r in rep_rx.iter() {
    println!("{:?}",r);
  }

  {
    let _s = scheduler::new();
  }
}
