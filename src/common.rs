#[derive(Copy,Clone,Debug)]
pub enum Message<T: Copy+Send>
{
  Empty,                      //
  Value(T),                   //
  Ack(usize,usize),           // from-to
  Error(usize,&'static str),  // error at
}

#[derive(Debug)]
pub enum Schedule {
  Loop,
  OnMessage,
  EndPlusMs(usize),
  StartPlusMs(usize),
  Stop,
}
