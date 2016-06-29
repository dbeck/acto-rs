extern crate lossyq;

pub struct Scheduler {
  alma : i32,
}

impl Scheduler {

  // start/add thread
  // "start" loop()

  // stop() "all"

  // spawn "runnable" <- worker/supervisor
  //  -- add(name,runnable)

  // send msg(name[], msg)

  // str???
}

pub fn new() -> Scheduler {
  Scheduler{
    alma : 0,
  }
}


#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn dummy() {
    remove_me();
  }
}
