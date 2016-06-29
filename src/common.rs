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
