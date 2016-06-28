/*
extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver};

pub struct Worker<In: Copy, Out: Copy, State> {
  input : Receiver<In>,
  output : Sender<Out>,
  state : State,
  fun : FnMut()
}

http://erlang.org/doc/man/gen_server.html

////////////////
Module:handle_call(Request, From, State) -> Result

Types:
Request = term()
From = {pid(),Tag}
State = term()
Result = {reply,Reply,NewState}
  | {reply,Reply,NewState,Timeout}
  | {reply,Reply,NewState,hibernate}
  | {noreply,NewState}
  | {noreply,NewState,Timeout}
  | {noreply,NewState,hibernate}
  | {stop,Reason,Reply,NewState}
  | {stop,Reason,NewState}
 Reply = term()
 NewState = term()
 Timeout = int()>=0 | infinity
 Reason = term()


 Module:handle_cast(Request, State) -> Result
 Types:

 Request = term()
 State = term()
 Result = {noreply,NewState}
   | {noreply,NewState,Timeout}
   | {noreply,NewState,hibernate}
   | {stop,Reason,NewState}
  NewState = term()
  Timeout = int()>=0 | infinity
  Reason = term()

  Module:handle_info(Info, State) -> Result
  Types:

  Info = timeout | term()
  State = term()
  Result = {noreply,NewState}
    | {noreply,NewState,Timeout}
    | {noreply,NewState,hibernate}
    | {stop,Reason,NewState}
   NewState = term()
   Timeout = int()>=0 | infinity
   Reason = normal | term()
*/

extern crate lossyq;
use self::lossyq::spsc::{Sender, Receiver, channel};

#[derive(Copy,Clone)]
pub enum Request<T: Copy+Send>
{
  Empty,
  Value(T)
}

#[derive(Copy,Clone)]
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

pub struct MyActor {
  state : i32,
}

pub trait Worker {
  type RequestType : Copy + Send;
  type ReplyType : Copy + Send;

  fn process(&mut self /*state*/,
    input: Receiver<Request<Self::RequestType>>,
    output: Sender<Reply<Self::ReplyType>>) -> Result;
}

pub trait Filter {
  type InputType : Copy + Send;
  type OutputType : Copy + Send;

  fn process(&mut self /*state*/,
    input: Receiver<Reply<Self::InputType>>,
    output: Sender<Reply<Self::OutputType>>) -> Result;
}

/*
pub struct Worker<State, Req: Copy+Send, Rep: Copy+Send, Fun> {
  input   : Receiver<Request<Req>>,
  output  : Sender<Reply<Rep>>,
  state   : State,
  process : Fun, // Fn(&mut State, Receiver<Request<Req>>) -> Reply<Rep>,
}

pub fn new<State, Req: Copy+Send, Rep: Copy+Send, F>(
    inq_size: usize,
    outq_size: usize,
    init_state: State,
    process_fn: F)
    -> ( Worker<State,Req,Rep,F>,
         Sender<Request<Req>>,
         Receiver<Reply<Rep>>)
    where F: Fn(&mut State, Receiver<Request<Req>>) -> Reply<Rep>
{
  let (tx_in,  rx_in) = channel(inq_size, Request::Empty);
  let (tx_out, rx_out) = channel(outq_size, Reply::Empty);
  ( Worker{
      input: rx_in,
      output: tx_out,
      state: init_state,
      process: process_fn},
    tx_in,
    rx_out )
}
*/

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() { }
}
