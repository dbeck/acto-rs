extern crate minions;
extern crate lossyq;
use minions::{scheduler, worker};
use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::worker::{Request, Reply};

trait X {
}

struct S {
  a : i32,
}

impl X for S {}

fn any_test() {
  use std::any::Any;
  {
    let s = S{a:1};
    let mut v = vec![];
    v.push(&s as &Any);
    for i in v {
      match i.downcast_ref::<S>() {
        Some(as_s) => {
          println!("Converted: {:?}",as_s.a);
        },
        None => {
          println!("No conversion");
        },
      }
    }
  }
}

fn main() {
  any_test();
  scheduler::remove_me();
  /*
  {
    let (_worker, mut sender, _receiver) =
        worker::new(1,1,1,|_state, _receiver : Receiver<Request<i32>>| Reply::Value(1,1,1) );

    sender.put(|v| *v = Request::Value(1));
  }
  {
    let (_worker, mut _sender, _receiver) : (
      worker::Worker<_,_,_,_>,
      Sender<Request<_>>,
      Receiver<Reply<i32>>) =
        worker::new(1,1,1,|_state, _receiver : Receiver<Request<i32>>| Reply::Value(1,1,1) );
  }
  */
}
