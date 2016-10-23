use super::super::{ChannelWrapper, ChannelId, ReceiverChannelId,
  ExpectedChannelState, ActualChannelState, ChannelState
};
use super::super::Error as ActorError;

pub fn connect_receiver_to_sender<Value: Send, Error: Send>(rcv : &mut ChannelWrapper<Value, Error>,
                                                            snd : &mut ChannelWrapper<Value, Error>)
    -> Result<(), ActorError>
{
  use std::mem;

  let (channel_id, mut tmp_sender) = match rcv {
    &mut ChannelWrapper::ReceiverNotConnected(ref mut receiver_channel_id, ref mut receiver_name) => {
      match snd {
        &mut ChannelWrapper::SenderNotConnected(ref mut sender_channel_id, ref mut _receiver, ref mut _sender_name) => {
          let channel_id = ChannelId{sender_id: *sender_channel_id, receiver_id: *receiver_channel_id};
          (channel_id, ChannelWrapper::ConnectedSender::<Value, Error>(channel_id.clone(), receiver_name.clone()))
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ReceiverNotConnected)));
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ConnectedReceiver)));
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ConnectedSender)));
        },
      }
    },
    &mut ChannelWrapper::ConnectedReceiver(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ReceiverNotConnected),
        ActualChannelState(ChannelState::ConnectedReceiver)));
    },
    &mut ChannelWrapper::ConnectedSender(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ReceiverNotConnected),
        ActualChannelState(ChannelState::ConnectedSender)));
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ReceiverNotConnected),
        ActualChannelState(ChannelState::SenderNotConnected)));
    },
  };

  mem::swap(&mut tmp_sender, snd);

  match tmp_sender {
    ChannelWrapper::SenderNotConnected(_sender_channel_id, receiver, sender_name) => {
      let mut new_receiver = ChannelWrapper::ConnectedReceiver::<Value, Error>(channel_id, receiver, sender_name.clone());
      mem::swap(&mut new_receiver, rcv);
    },
    ChannelWrapper::ReceiverNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::SenderNotConnected),
        ActualChannelState(ChannelState::ReceiverNotConnected)));
    },
    ChannelWrapper::ConnectedReceiver(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::SenderNotConnected),
        ActualChannelState(ChannelState::ConnectedReceiver)));
    },
    ChannelWrapper::ConnectedSender(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::SenderNotConnected),
        ActualChannelState(ChannelState::ConnectedSender)));
    },
  };

  Ok(())
}

pub fn disconnect_receiver_from_sender<Value: Send, Error: Send>(rcv : &mut ChannelWrapper<Value, Error>,
                                                                 snd : &mut ChannelWrapper<Value, Error>)
    -> Result<(), ActorError>
{
  use std::mem;

  let mut tmp_receiver = match snd {
    &mut ChannelWrapper::ConnectedSender(ref mut channel_id_rcv, ref mut receiver_name) => {
      match rcv {
        &mut ChannelWrapper::ConnectedReceiver(ref mut _channel_id_snd, ref mut _receiver, ref mut _sender_name) => {
          ChannelWrapper::ReceiverNotConnected::<Value, Error>(channel_id_rcv.receiver_id,receiver_name.clone())
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedReceiver),
            ActualChannelState(ChannelState::ReceiverNotConnected)));
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedReceiver),
            ActualChannelState(ChannelState::ConnectedSender)));
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          return Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedReceiver),
            ActualChannelState(ChannelState::SenderNotConnected)));
        },
      }
    },
    &mut ChannelWrapper::ReceiverNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedSender),
        ActualChannelState(ChannelState::ReceiverNotConnected)));
    },

    &mut ChannelWrapper::ConnectedReceiver(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedSender),
        ActualChannelState(ChannelState::ConnectedReceiver)));
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedSender),
        ActualChannelState(ChannelState::SenderNotConnected)));
    },
  };

  mem::swap(&mut tmp_receiver, rcv);

  match tmp_receiver {
    ChannelWrapper::ConnectedReceiver(channel_id_snd, receiver, sender_name) => {
      let mut new_sender = ChannelWrapper::SenderNotConnected::<Value, Error>(
        channel_id_snd.sender_id, receiver, sender_name.clone());
      mem::swap(&mut new_sender, snd);
    },
    ChannelWrapper::ReceiverNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedReceiver),
        ActualChannelState(ChannelState::ReceiverNotConnected)));
    },
    ChannelWrapper::ConnectedSender(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedReceiver),
        ActualChannelState(ChannelState::ConnectedSender)));
    },
    ChannelWrapper::SenderNotConnected(..) => {
      return Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedReceiver),
        ActualChannelState(ChannelState::SenderNotConnected)));
    },
  };
  Ok(())
}

pub fn connect_to<Value: Send, Error: Send>(me : &mut ChannelWrapper<Value, Error>,
                                            to : &mut ChannelWrapper<Value, Error>)
    -> Result<(), ActorError>
{
  match me {
    &mut ChannelWrapper::ReceiverNotConnected(..) => {
      match to {
        &mut ChannelWrapper::SenderNotConnected(..) => {
          connect_receiver_to_sender(me, to)
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ConnectedSender)))
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ReceiverNotConnected)))
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::SenderNotConnected),
            ActualChannelState(ChannelState::ConnectedReceiver)))
        }
      }
    },
    &mut ChannelWrapper::ConnectedReceiver(..) => {
      Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ReceiverNotConnected),
        ActualChannelState(ChannelState::ConnectedReceiver)))
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      match to {
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          connect_receiver_to_sender(to, me)
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ReceiverNotConnected),
            ActualChannelState(ChannelState::ConnectedSender)))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ReceiverNotConnected),
            ActualChannelState(ChannelState::SenderNotConnected)))
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ReceiverNotConnected),
            ActualChannelState(ChannelState::ConnectedReceiver)))
        }
      }
    },
    &mut ChannelWrapper::ConnectedSender(..) => {
      Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::SenderNotConnected),
        ActualChannelState(ChannelState::ConnectedSender)))
    },
  }
}

pub fn disconnect_from<Value: Send, Error: Send>(me   : &mut ChannelWrapper<Value, Error>,
                                                 from : &mut ChannelWrapper<Value, Error>)
    -> Result<(), ActorError>
{
  match me {
    &mut ChannelWrapper::ConnectedReceiver(..) => {
      match from {
        &mut ChannelWrapper::ConnectedSender(..) => {
          disconnect_receiver_from_sender(me, from)
        },
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedSender),
            ActualChannelState(ChannelState::ConnectedReceiver)))
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedSender),
            ActualChannelState(ChannelState::ReceiverNotConnected)))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedSender),
            ActualChannelState(ChannelState::SenderNotConnected)))
        }
      }
    },
    &mut ChannelWrapper::ConnectedSender(..) => {
      match from {
        &mut ChannelWrapper::ConnectedReceiver(..) => {
          disconnect_receiver_from_sender(from, me)
        },
        &mut ChannelWrapper::ReceiverNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedReceiver),
            ActualChannelState(ChannelState::ReceiverNotConnected)))
        },
        &mut ChannelWrapper::SenderNotConnected(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedReceiver),
            ActualChannelState(ChannelState::SenderNotConnected)))
        },
        &mut ChannelWrapper::ConnectedSender(..) => {
          Err(ActorError::InvalidChannelState(
            ExpectedChannelState(ChannelState::ConnectedSender),
            ActualChannelState(ChannelState::SenderNotConnected)))
        }
      }
    },
    &mut ChannelWrapper::ReceiverNotConnected(..) => {
      Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedReceiver),
        ActualChannelState(ChannelState::ReceiverNotConnected)))
    },
    &mut ChannelWrapper::SenderNotConnected(..) => {
      Err(ActorError::InvalidChannelState(
        ExpectedChannelState(ChannelState::ConnectedSender),
        ActualChannelState(ChannelState::SenderNotConnected)))
    },
  }
}

pub trait Connectable {
  type InputValue: Send;
  type InputError: Send;

  fn input(&mut self) -> &mut ChannelWrapper<Self::InputValue, Self::InputError>;

  fn connect(&mut self,
             other: &mut ChannelWrapper<Self::InputValue, Self::InputError>)
      -> Result<(), ActorError>
  {
    connect_to(self.input(), other)
  }

  fn disconnect(&mut self,
                other: &mut ChannelWrapper<Self::InputValue, Self::InputError>)
      -> Result<(), ActorError> {
    disconnect_from(self.input(), other)
  }
}

pub trait ConnectableY {
  type InputValueA: Send;
  type InputErrorA: Send;
  type InputValueB: Send;
  type InputErrorB: Send;

  fn input_a(&mut self) -> &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>;
  fn input_b(&mut self) -> &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>;

  fn connect_a(&mut self,
               other: &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>)
      -> Result<(), ActorError>
  {
    connect_to(self.input_a(), other)
  }

  fn connect_b(&mut self,
              other: &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>)
      -> Result<(), ActorError>
  {
    connect_to(self.input_b(), other)
  }

  fn disconnect_a(&mut self,
                  other: &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>)
      -> Result<(), ActorError>
  {
    disconnect_from(self.input_a(), other)
  }

  fn disconnect_b(&mut self,
                  other: &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>)
      -> Result<(), ActorError>
  {
    disconnect_from(self.input_b(), other)
  }
}

pub trait ConnectableN {
  type InputValue: Send;
  type InputError: Send;

  fn input(&mut self,
           n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputValue, Self::InputError>;

  fn connect(&mut self,
             n: ReceiverChannelId,
             other: &mut ChannelWrapper<Self::InputValue, Self::InputError>)
      -> Result<(), ActorError>
  {
    connect_to(self.input(n), other)
  }

  fn disconnect(&mut self,
                n: ReceiverChannelId,
                other: &mut ChannelWrapper<Self::InputValue, Self::InputError>)
      -> Result<(), ActorError>
  {
    disconnect_from(self.input(n), other)
  }
}

pub trait ConnectableYN {
  type InputValueA: Send;
  type InputErrorA: Send;
  type InputValueB: Send;
  type InputErrorB: Send;

  fn input_a(&mut self,
             n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>;

  fn input_b(&mut self,
             n: ReceiverChannelId) -> &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>;

  fn connect_a(&mut self,
               n: ReceiverChannelId,
               other: &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>)
      -> Result<(), ActorError>
  {
    connect_to(self.input_a(n), other)
  }

  fn connect_b(&mut self,
               n: ReceiverChannelId,
               other: &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>)
      -> Result<(), ActorError>
  {
    connect_to(self.input_b(n), other)
  }

  fn disconnect_a(&mut self,
                  n: ReceiverChannelId,
                  other: &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>)
      -> Result<(), ActorError>
  {
    disconnect_from(self.input_a(n), other)
  }

  fn disconnect_b(&mut self,
                  n: ReceiverChannelId,
                  other: &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>)
      -> Result<(), ActorError>
  {
    disconnect_from(self.input_b(n), other)
  }
}
