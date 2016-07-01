extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};
use super::common::{Request, Reply, Result};

pub trait Worker {
  type RequestType  : Copy+Send;
  type ReplyType    : Copy+Send;

  fn process(
    &mut self,
    input:   &mut Receiver<Request<Self::RequestType>>,
    output:  &mut Sender<Reply<Self::ReplyType>>) -> Result;
}

pub struct WorkerWrap<Req: Copy+Send, Rep: Copy+Send> {
  name        : String,
  worker      : Box<Worker<RequestType=Req,ReplyType=Rep>>,
  request_rx  : Receiver<Request<Req>>,
  reply_tx    : Sender<Reply<Rep>>,
}

impl<Req : Copy+Send, Rep : Copy+Send> WorkerWrap<Req, Rep> {
  pub fn process(&mut self) -> Result {
    self.worker.process(&mut self.request_rx, &mut self.reply_tx)
  }
}

pub fn new<'a, Req: Copy+Send, Rep: Copy+Send>(
    name            : String,
    request_q_size  : usize,
    reply_q_size    : usize,
    worker          : Box<Worker<RequestType=Req,ReplyType=Rep>>) ->
    ( WorkerWrap<Req, Rep>,
      Sender<Request<Req>>,
      Receiver<Reply<Rep>> )
{
  let (request_tx, request_rx) = channel(request_q_size, Request::Empty);
  let (reply_tx, reply_rx) = channel(reply_q_size, Reply::Empty);
  (
    WorkerWrap{
      name        : name,
      worker      : worker,
      request_rx  : request_rx,
      reply_tx    : reply_tx,
    },
    request_tx,
    reply_rx
  )
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
