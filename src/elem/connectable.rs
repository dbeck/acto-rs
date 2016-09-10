use super::super::{ChannelWrapper, ChannelId, ReceiverChannelId};

pub fn connect_receiver_to_sender<Input: Send>(rcv : &mut ChannelWrapper<Input>,
                                               snd : &mut ChannelWrapper<Input>)
    -> Result<(),String>
{
  use std::mem;

  let (channel_id, mut tmp_sender) = match rcv {
    &mut ChannelWrapper::ReceiverNotConnected(ref mut receiver_channel_id, ref mut receiver_name) => {
      match snd {
        &mut ChannelWrapper::SenderNotConnected(ref mut sender_channel_id, ref mut _receiver, ref mut _sender_name) => {
          let channel_id = ChannelId{sender_id: *sender_channel_id, receiver_id: *receiver_channel_id};
          (channel_id, ChannelWrapper::ConnectedSender::<Input>(channel_id.clone(), receiver_name.clone()))
        }
        _ => {
          return Err(String::from("internal error: expected SenderNotConnected here"));
        }
      }
    },
    _ => {
      return Err(String::from("internal error: expected ReceiverNotConnected here"));
    }
  };

  mem::swap(&mut tmp_sender, snd);

  match tmp_sender {
    ChannelWrapper::SenderNotConnected(_sender_channel_id, receiver, sender_name) => {
      let mut new_receiver = ChannelWrapper::ConnectedReceiver::<Input>(channel_id, receiver, sender_name.clone());
      mem::swap(&mut new_receiver, rcv);
    },
    _ => {
      return Err(String::from("internal error: expected SenderNotConnected here"));
    }
  };

  Ok(())
}

pub fn disconnect_receiver_from_sender<Input: Send>(rcv : &mut ChannelWrapper<Input>,
                                                    snd : &mut ChannelWrapper<Input>)
    -> Result<(),String>
{
  use std::mem;

  let mut tmp_receiver = match snd {
    &mut ChannelWrapper::ConnectedSender(ref mut channel_id_rcv, ref mut receiver_name) => {
      match snd {
        &mut ChannelWrapper::ConnectedReceiver(ref mut _channel_id_snd, ref mut _receiver, ref mut _sender_name) => {
          ChannelWrapper::ReceiverNotConnected::<Input>(channel_id_rcv.receiver_id,receiver_name.clone())
        }
        _ => {
          return Err(String::from("internal error: expected ConnectedReceiver here"));
        }
      }
    },
    _ => {
      return Err(String::from("internal error: expected ConnectedSender here"));
    }
  };

  mem::swap(&mut tmp_receiver, rcv);

  match tmp_receiver {
    ChannelWrapper::ConnectedReceiver(channel_id_snd, receiver, sender_name) => {
      let mut new_sender = ChannelWrapper::SenderNotConnected::<Input>(
        channel_id_snd.sender_id, receiver, sender_name.clone());
      mem::swap(&mut new_sender, snd);
    },
    _ => {
      return Err(String::from("internal error: expected ConnectedReceiver here"));
    }
  };

  Ok(())
}


pub fn connect_to<Input: Send>(me : &mut ChannelWrapper<Input>,
                               to : &mut ChannelWrapper<Input>)
    -> Result<(),String>
{
  match me {
    &mut ChannelWrapper::ReceiverNotConnected(..) => {
      match to {
        &mut ChannelWrapper::SenderNotConnected(..) => {
          connect_receiver_to_sender(me, to)
        },
        &mut ChannelWrapper::ConnectedSender(ref _channel_id, ref receiver_name) => {
          Err(format!("sender already connected to {:?}", receiver_name))
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(format!("invalid state, cannot connect. - ReceiverNotConnected/ReceiverNotConnected"))
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(format!("invalid state, cannot connect. - ReceiverNotConnected/ConnectedReceiver"))
        }
      }
    },
    &mut ChannelWrapper::ConnectedReceiver(ref _channel_id, ref _receiver, ref sender_name) => {
      Err(format!("receiver already connected to {:?}", sender_name))
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      match to {
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          connect_receiver_to_sender(to, me)
        },
        &mut ChannelWrapper::ConnectedSender(_channel_id, ref receiver_name) => {
          Err(format!("sender already connected to {:?}", receiver_name))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(format!("invalid state, cannot connect. - SenderNotConnected/SenderNotConnected"))
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(format!("invalid state, cannot connect. - SenderNotConnected/ConnectedReceiver"))
        }
      }
    },
    &mut ChannelWrapper::ConnectedSender(..) => {
      Err(format!("invalid state LHS/me - ConnectedSender"))
    },
  }
}

pub fn disconnect_from<Input: Send>(me   : &mut ChannelWrapper<Input>,
                                    from : &mut ChannelWrapper<Input>)
    -> Result<(),String>
{
  match me {
    &mut ChannelWrapper::ConnectedReceiver(..) => {
      match from {
        &mut ChannelWrapper::ConnectedSender(..) => {
          disconnect_receiver_from_sender(me, from)
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedReceiver/ConnectedReceiver"))
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedReceiver/ReceiverNotConnected"))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedReceiver/SenderNotConnected"))
        }
      }
    },
    &mut ChannelWrapper::ConnectedSender(..) => {
      match from {
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          disconnect_receiver_from_sender(from, me)
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedSender/ReceiverNotConnected"))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedSender/SenderNotConnected"))
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          Err(format!("cannot disconnect, invalid state: from state: ConnectedSender/ConnectedSender"))
        }
      }
    },
    &mut ChannelWrapper::ReceiverNotConnected(..) => {
      Err(format!("cannot disconnect from state: ReceiverNotConnected"))
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      Err(format!("cannot disconnect from state: SenderNotConnected"))
    },
  }
}

pub trait Connectable {
  type Input: Send;

  fn input(&mut self) -> &mut ChannelWrapper<Self::Input>;

  fn connect(&mut self, other: &mut ChannelWrapper<Self::Input>)
      -> Result<(),String> {
    connect_to(self.input(), other)
  }

  fn disconnect(&mut self, other: &mut ChannelWrapper<Self::Input>)
      -> Result<(),String> {
    disconnect_from(self.input(), other)
  }
}

pub trait ConnectableY {
  type InputA: Send;
  type InputB: Send;

  fn input_a(&mut self) -> &mut ChannelWrapper<Self::InputA>;
  fn input_b(&mut self) -> &mut ChannelWrapper<Self::InputB>;

  fn connect_a(&mut self, other: &mut ChannelWrapper<Self::InputA>)
      -> Result<(),String> {
    connect_to(self.input_a(), other)
  }

  fn connect_b(&mut self, other: &mut ChannelWrapper<Self::InputB>)
      -> Result<(),String> {
    connect_to(self.input_b(), other)
  }

  fn disconnect_a(&mut self, other: &mut ChannelWrapper<Self::InputA>)
      -> Result<(),String> {
    disconnect_from(self.input_a(), other)
  }

  fn disconnect_b(&mut self, other: &mut ChannelWrapper<Self::InputB>)
      -> Result<(),String> {
    disconnect_from(self.input_b(), other)
  }
}

pub trait ConnectableN {
  type Input: Send;

  fn input(&mut self, n: ReceiverChannelId) -> &mut ChannelWrapper<Self::Input>;

  fn connect(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::Input>)
      -> Result<(),String> {
    connect_to(self.input(n), other)
  }

  fn disconnect(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::Input>)
      -> Result<(),String> {
    disconnect_from(self.input(n), other)
  }
}

pub trait ConnectableYN {
  type InputA: Send;
  type InputB: Send;

  fn input_a(&mut self, n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputA>;
  fn input_b(&mut self, n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputB>;

  fn connect_a(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::InputA>)
      -> Result<(),String> {
    connect_to(self.input_a(n), other)
  }

  fn connect_b(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::InputB>)
      -> Result<(),String> {
    connect_to(self.input_b(n), other)
  }

  fn disconnect_a(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::InputA>)
      -> Result<(),String> {
    disconnect_from(self.input_a(n), other)
  }

  fn disconnect_b(&mut self, n: ReceiverChannelId, other: &mut ChannelWrapper<Self::InputB>)
      -> Result<(),String> {
    disconnect_from(self.input_b(n), other)
  }
}
