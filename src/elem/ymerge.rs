use lossyq::spsc::{Sender, channel};
use super::super::{Message, ChannelWrapper, SenderName, ReceiverChannelId,
  ReceiverName, SenderChannelId
};
use super::wrap::ymerge_wrap;

pub trait YMerge {
  type InputValueA   : Send;
  type InputErrorA   : Send;
  type InputValueB   : Send;
  type InputErrorB   : Send;
  type OutputValue   : Send;
  type OutputError   : Send;

  fn process(
    &mut self,
    input_a:  &mut ChannelWrapper<Self::InputValueA, Self::InputErrorA>,
    input_b:  &mut ChannelWrapper<Self::InputValueB, Self::InputErrorB>,
    output:   &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
    stop:     &mut bool);
}

pub fn new<InputValueA: Send, InputErrorA: Send,
           InputValueB: Send, InputErrorB: Send,
           OutputValue: Send, OutputError: Send>(
    name             : &str,
    output_q_size    : usize,
    ymerge           : Box<YMerge<InputValueA=InputValueA, InputErrorA=InputErrorA,
                                  InputValueB=InputValueB, InputErrorB=InputErrorB,
                                  OutputValue=OutputValue, OutputError=OutputError>+Send>)
      -> (Box<ymerge_wrap::YMergeWrap<InputValueA, InputErrorA,
                                      InputValueB, InputErrorB,
                                      OutputValue, OutputError>>,
          Box<ChannelWrapper<OutputValue, OutputError>>)
{
  let (output_tx,  output_rx) = channel(output_q_size);
  let name = String::from(name);

  (
    Box::new(
      ymerge_wrap::new(
        name.clone(),
        ymerge,
        ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(1),
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
