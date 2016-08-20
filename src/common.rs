use lossyq::spsc::Receiver;
use std::fmt;

#[derive(Copy,Clone,Debug)]
pub enum Message<T: Send>
{
  Empty,                      //
  Value(T),                   //
  Ack(usize,usize),           // from-to
  Error(usize,&'static str),  // error at
}

#[derive(Debug)]
pub enum Schedule {
  Loop,
  OnMessage(u64),
  EndPlusUSec(u64),
  StartPlusUSec(u64),
  OnExternalEvent,
  Stop,
}

pub trait Reporter {
  fn message_sent(&mut self, channel_id: usize, last_msg_id: usize);
}

pub trait Task {
  fn execute(&mut self, reporter: &mut Reporter) -> Schedule;
  fn name(&self)  -> &String;
  fn input_count(&self) -> usize;
  fn output_count(&self) -> usize;
}

#[derive(Clone,Debug)]
pub struct Id {
  task_name  : String,
  id         : usize,
}

pub struct IdentifiedReceiver<Input: Send> {
  pub id    : Id,
  pub input : Receiver<Message<Input>>,
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Id:({} {})", self.task_name, self.id)
    }
}

pub fn new_id(name: String, id: usize) -> Id {
  Id {
    task_name  : name,
    id         : id,
  }
}
