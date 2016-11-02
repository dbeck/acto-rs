use lossyq::spsc::{Sender, channel};
use super::super::{Message, ChannelWrapper, SenderName, SenderChannelId,
  ReceiverChannelId, ReceiverName
};
use super::wrap::gather_wrap;

pub trait Gather {
  type InputValue   : Send;
  type InputError   : Send;
  type OutputValue  : Send;
  type OutputError  : Send;

  fn process(
    &mut self,
    input:   &mut Vec<ChannelWrapper<Self::InputValue, Self::InputError>>,
    output:  &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop:    &mut bool);
}

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    gather          : Box<Gather<InputValue=InputValue,   InputError=InputError,
                                 OutputValue=OutputValue, OutputError=OutputError>+Send>,
    n_channels      : usize)
      -> (Box<gather_wrap::GatherWrap<InputValue, InputError, OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);
  let mut inputs = vec![];
  for i in 0..n_channels {
    inputs.push(ChannelWrapper::ReceiverNotConnected(
      ReceiverChannelId(i),
      ReceiverName (name.clone())
    ));
  }

  (
    Box::new(gather_wrap::new( name.clone(), gather, inputs, output_tx)),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
