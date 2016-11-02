use lossyq::spsc::{Sender, channel};
use super::super::{Message, ChannelWrapper, SenderName, SenderChannelId,
  ReceiverChannelId, ReceiverName
};
use super::wrap::scatter_wrap;

pub trait Scatter {
  type InputValue   : Send;
  type InputError   : Send;
  type OutputValue  : Send;
  type OutputError  : Send;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    output:  &mut Vec<Sender<Message<Self::OutputValue, Self::OutputError>>>,
    stop:    &mut bool);
}

pub fn new<InputValue: Send, InputError: Send, OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    scatter         : Box<Scatter<InputValue=InputValue, InputError=InputError,
                                  OutputValue=OutputValue, OutputError=OutputError>+Send>,
    n_channels      : usize)
      -> (Box<scatter_wrap::ScatterWrap<InputValue, InputError, OutputValue, OutputError>>,
          Vec<Box<ChannelWrapper<OutputValue, OutputError>>>)
{
  let mut tx_vec = Vec::with_capacity(n_channels);
  let mut rx_vec = Vec::with_capacity(n_channels);
  let name = String::from(name);

  for i in 0..n_channels {
    let (output_tx, output_rx) = channel(output_q_size);
    tx_vec.push(output_tx);
    rx_vec.push(
      Box::new(
        ChannelWrapper::SenderNotConnected(
          SenderChannelId(i),
          output_rx,
          SenderName(name.clone())
        )
      )
    );
  }

  (
    Box::new(
      scatter_wrap::new(
        name.clone(),
        scatter,
        ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        tx_vec,
      )
    ),
    rx_vec
  )
}
