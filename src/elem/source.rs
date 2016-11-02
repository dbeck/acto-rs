use lossyq::spsc::{Sender, channel};
use super::super::{Message, SenderName, ChannelWrapper, SenderChannelId};
use super::wrap::source_wrap;

pub trait Source {
  type OutputValue : Send;
  type OutputError : Send;

  fn process(
    &mut self,
    output: &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop: &mut bool);
}

pub fn new<OutputValue: Send, OutputError: Send>(
    name            : &str,
    output_q_size   : usize,
    source          : Box<Source<OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<source_wrap::SourceWrap<OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx, output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(source_wrap::new(name.clone(), source, output_tx)),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_rx,
        SenderName(name)
      )
    )
  )
}
