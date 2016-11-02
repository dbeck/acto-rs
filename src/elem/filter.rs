use lossyq::spsc::{Sender, channel};
use super::super::{Message, ChannelWrapper, SenderChannelId, ReceiverChannelId,
  ReceiverName, SenderName
};
use super::wrap::filter_wrap;

pub trait Filter {
  type InputValue   : Send;
  type InputError   : Send;
  type OutputValue  : Send;
  type OutputError  : Send;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    output:  &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop: &mut bool);
}

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    filter          : Box<Filter<InputValue=InputValue, InputError=InputError,
                                 OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<filter_wrap::FilterWrap<InputValue, InputError, OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      filter_wrap::new(
        name.clone(),
        filter,
        ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_tx
      )
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
