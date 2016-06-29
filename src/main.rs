extern crate minions;
extern crate lossyq;

use minions::{scheduler, worker};
use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::common;
use minions::common::{Request, Reply};

#[derive(Copy, Clone)]
struct WorkerState {
  state : i32,
}

impl worker::Worker for WorkerState {
  type RequestType = i32;
  type ReplyType = i32;

  fn process(
    &mut self,
    input: &mut Receiver<Request<Self::RequestType>>,
    output: &mut Sender<Reply<Self::ReplyType>>) -> common::Result {
      for i in input.iter() {
        match i {
          Request::Value(v) => {
            output.put(|x| *x = Reply::Value(0,0,v));
          }
          _ => { println!("Unknown value"); }
        }
      }
      common::Result::Ok
  }
}

fn main() {
  let wrk: Box<worker::Worker<RequestType=i32,ReplyType=i32>> = Box::new(WorkerState{state:0});

  let (mut ww, mut req_tx, mut rep_rx) = worker::new( String::from("Hello"), 2, 2, wrk);

  req_tx.put(|v| *v = Request::Value(1));
  req_tx.put(|v| *v = Request::Value(2));
  ww.process();
  for r in rep_rx.iter() {
    println!("{:?}",r);
  }

  {
    let _s = scheduler::new(1);
  }
}
