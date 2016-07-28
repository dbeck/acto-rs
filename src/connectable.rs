use super::common::IdentifiedReceiver;

fn connect_to<Input: Copy+Send>(me : &mut Option<IdentifiedReceiver<Input>>,
                                to : &mut Option<IdentifiedReceiver<Input>>)
                                -> Result<(),String>
{
  use std::mem;
  if me.is_some() {
    let ch : &Option<IdentifiedReceiver<Input>> = me;
    match ch {
      &Some(ref v) => {
        Err(format!("already connected to {}",v.id))
      },
      &None => {
        Err(String::from("cannot happen"))
      }
    }
  }
  else if to.is_none() {
    Err(format!("already connected"))
  }
  else {
    mem::swap(me,to);
    Ok(())
  }
}

fn disconnect_from<Input: Copy+Send>(me   : &mut Option<IdentifiedReceiver<Input>>,
                                    from : &mut Option<IdentifiedReceiver<Input>>)
                                    -> Result<(),String>
{
  use std::mem;
  if me.is_none() {
    Err(format!("not connected"))
  }
  else if from.is_some() {
    match from {
      &mut Some(ref v) => {
        Err(format!("not yet connected. has channel: {}", v.id)
        )
      },
      &mut None => {
        Err(String::from("cannot happen"))
      }
    }
  }
  else {
    mem::swap(me,from);
    Ok(())
  }
}

pub trait Connectable {
  type Input: Copy+Send;

  fn input(&mut self) -> &mut Option<IdentifiedReceiver<Self::Input>>;

  fn connect(&mut self, other: &mut Option<IdentifiedReceiver<Self::Input>>)
      -> Result<(),String> {
    connect_to(self.input(), other)
  }

  fn disconnect(&mut self, other: &mut Option<IdentifiedReceiver<Self::Input>>)
      -> Result<(),String> {
    disconnect_from(self.input(), other)
  }
}

pub trait ConnectableY {
  type InputA: Copy+Send;
  type InputB: Copy+Send;

  fn input_a(&mut self) -> &mut Option<IdentifiedReceiver<Self::InputA>>;
  fn input_b(&mut self) -> &mut Option<IdentifiedReceiver<Self::InputB>>;

  fn connect_a(&mut self, other: &mut Option<IdentifiedReceiver<Self::InputA>>)
      -> Result<(),String> {
    connect_to(self.input_a(), other)
  }

  fn connect_b(&mut self, other: &mut Option<IdentifiedReceiver<Self::InputB>>)
      -> Result<(),String> {
    connect_to(self.input_b(), other)
  }

  fn disconnect_a(&mut self, other: &mut Option<IdentifiedReceiver<Self::InputA>>)
      -> Result<(),String> {
    disconnect_from(self.input_a(), other)
  }

  fn disconnect_b(&mut self, other: &mut Option<IdentifiedReceiver<Self::InputB>>)
      -> Result<(),String> {
    disconnect_from(self.input_b(), other)
  }
}

pub trait ConnectableN {
  type Input: Copy+Send;

  fn input(&mut self, n: usize) -> &mut Option<IdentifiedReceiver<Self::Input>>;

  fn connect(&mut self, n: usize, other: &mut Option<IdentifiedReceiver<Self::Input>>)
      -> Result<(),String> {
    connect_to(self.input(n), other)
  }

  fn disconnect(&mut self, n: usize, other: &mut Option<IdentifiedReceiver<Self::Input>>)
      -> Result<(),String> {
    disconnect_from(self.input(n), other)
  }
}
