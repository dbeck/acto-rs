extern crate minions;
extern crate lossyq;

use minions::{scheduler, worker};
use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::worker::{Request, Reply};

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
    output: &mut Sender<Reply<Self::ReplyType>>) -> worker::Result {
      for i in input.iter() {
        match i {
          worker::Request::Value(v) => {
            output.put(|x| *x = worker::Reply::Value(0,0,v));
          }
          _ => { println!("Unknown value"); }
        }
      }
      worker::Result::Ok
  }
}

fn main() {
  let mut state = WorkerState{state:0};

  let (mut ww, mut comm, mut req_tx, mut rep_rx) = worker::new(
    String::from("Hello"),
    2,
    2,
    &mut state as &mut worker::Worker<RequestType=i32,ReplyType=i32>);

  req_tx.put(|v| *v = worker::Request::Value(1));
  ww.process(&mut comm);
  for r in rep_rx.iter() {
    println!("{:?}",r);
  }

  scheduler::remove_me();
}
