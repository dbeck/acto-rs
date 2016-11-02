use lossyq::spsc::{Sender, channel};
use super::super::{Message, ChannelWrapper, SenderChannelId, ReceiverChannelId,
  ReceiverName, SenderName
};
use super::wrap::ysplit_wrap;

pub trait YSplit {
  type InputValue    : Send;
  type InputError    : Send;
  type OutputValueA  : Send;
  type OutputErrorA  : Send;
  type OutputValueB  : Send;
  type OutputErrorB  : Send;

  fn process(
    &mut self,
    input:     &mut ChannelWrapper<Self::InputValue, Self::InputError>,
    output_a:  &mut Sender<Message<Self::OutputValueA, Self::OutputErrorA>>,
    output_b:  &mut Sender<Message<Self::OutputValueB, Self::OutputErrorB>>,
    stop:      &mut bool);
}

pub fn new<InputValue: Send,   InputError: Send,
           OutputValueA: Send, OutputErrorA: Send,
           OutputValueB: Send, OutputErrorB: Send>(
    name              : &str,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputValue=InputValue, InputError=InputError,
                                   OutputValueA=OutputValueA, OutputErrorA=OutputErrorA,
                                   OutputValueB=OutputValueB, OutputErrorB=OutputErrorB>+Send>)
      -> (Box<ysplit_wrap::YSplitWrap<InputValue, InputError,
                                      OutputValueA, OutputErrorA,
                                      OutputValueB, OutputErrorB>>,
          Box<ChannelWrapper<OutputValueA, OutputErrorA>>,
          Box<ChannelWrapper<OutputValueB, OutputErrorB>>)
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size);
  let name = String::from(name);

  (
    Box::new(
      ysplit_wrap::new(
        name.clone(),
        ysplit,
        ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_a_tx,
        output_b_tx
      )
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(0),
        output_a_rx,
        SenderName(name.clone())
      )
    ),
    Box::new(
      ChannelWrapper::SenderNotConnected(
        SenderChannelId(1),
        output_b_rx,
        SenderName(name)
      )
    ),
  )
}
