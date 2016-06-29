extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};

#[derive(Copy,Clone,Debug)]
pub enum Request<T: Copy+Send>
{
  Empty,
  Value(T)
}

#[derive(Copy,Clone,Debug)]
pub enum Reply<T: Copy+Send>
{
  Empty,                      // no msg
  Error(usize,&'static str),  // msg error
  Ack(usize,usize),           // from-to
  Value(usize,usize,T)        // value from-to
}

#[derive(Debug)]
pub enum Result {
  Ok,
  Stop,
}

pub trait Worker {
  type RequestType : Copy+Send;
  type ReplyType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Request<Self::RequestType>>,
    output: &mut Sender<Reply<Self::ReplyType>>) -> Result;
}

pub trait Filter {
  type InputType : Copy+Send;
  type OutputType : Copy+Send;

  fn process(
    &mut self,
    input: &mut Receiver<Reply<Self::InputType>>,
    output: &mut Sender<Reply<Self::OutputType>>) -> Result;
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
