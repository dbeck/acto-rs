use super::super::{ChannelWrapper, ReceiverChannelId, ReceiverName};
use super::wrap::sink_wrap;

pub trait Sink {
  type InputValue : Send;
  type InputError : Send;

  fn process(
    &mut self,
    input:  &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    stop:   &mut bool);
}

pub fn new<InputValue: Send, InputError: Send>(
    name   : &str,
    sink   : Box<Sink<InputValue=InputValue, InputError=InputError>+Send>)
      -> Box<sink_wrap::SinkWrap<InputValue, InputError>>
{
  let name = String::from(name);
  Box::new(
    sink_wrap::new(
      name.clone(),
      sink,
      ChannelWrapper::ReceiverNotConnected(
        ReceiverChannelId(0),
        ReceiverName (name)
      )
    )
  )
}
