use lossyq::spsc::{Sender, channel};
use super::super::{Task, Message, Schedule, ChannelWrapper, ChannelId,
  SenderChannelId, ReceiverChannelId, ReceiverName, SenderName
};
use super::connectable::{Connectable};
use super::identified_input::{IdentifiedInput};
use super::output_counter::{OutputCounter};

pub trait YSplit {
  type InputType    : Send;
  type OutputTypeA  : Send;
  type OutputTypeB  : Send;

  fn process(
    &mut self,
    input:     &mut ChannelWrapper<Self::InputType>,
    output_a:  &mut Sender<Message<Self::OutputTypeA>>,
    output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> Schedule;
}

pub struct YSplitWrap<Input: Send, OutputA: Send, OutputB: Send> {
  name          : String,
  state         : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>,
  input_rx      : ChannelWrapper<Input>,
  output_a_tx   : Sender<Message<OutputA>>,
  output_b_tx   : Sender<Message<OutputB>>,
}

impl<Input: Send, OutputA: Send, OutputB: Send> IdentifiedInput for YSplitWrap<Input, OutputA, OutputB> {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    if ch_id.0 != 0 {
      None
    } else {
      match &self.input_rx {
        &ChannelWrapper::ConnectedReceiver(ref channel_id, ref _receiver, ref sender_name) => {
          Some((*channel_id, sender_name.clone()))
        },
        _ => None,
      }
    }
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> OutputCounter for  YSplitWrap<Input, OutputA, OutputB> {
  fn get_tx_count(&self, ch_id: usize) -> usize {
    if ch_id == 0 {
      self.output_a_tx.seqno()
    } else if ch_id == 1 {
      self.output_b_tx.seqno()
    } else {
      0
    }
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> Connectable for YSplitWrap<Input, OutputA, OutputB> {
  type Input = Input;

  fn input(&mut self) -> &mut ChannelWrapper<Input> {
    &mut self.input_rx
  }
}

impl<Input: Send, OutputA: Send, OutputB: Send> Task for YSplitWrap<Input, OutputA, OutputB> {
  fn execute(&mut self) -> Schedule {
    self.state.process(&mut self.input_rx,
                       &mut self.output_a_tx,
                       &mut self.output_b_tx)
  }
  fn name(&self) -> &String { &self.name }
  fn input_count(&self) -> usize { 1 }
  fn output_count(&self) -> usize { 2 }
  fn input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)> {
    self.get_input_id(ch_id)
  }
}

pub fn new<Input: Send, OutputA: Send, OutputB: Send>(
    name              : &str,
    output_a_q_size   : usize,
    output_b_q_size   : usize,
    ysplit            : Box<YSplit<InputType=Input, OutputTypeA=OutputA, OutputTypeB=OutputB>+Send>)
      -> (Box<YSplitWrap<Input,OutputA,OutputB>>,
          Box<ChannelWrapper<OutputA>>,
          Box<ChannelWrapper<OutputB>>)
{
  let (output_a_tx, output_a_rx) = channel(output_a_q_size);
  let (output_b_tx, output_b_rx) = channel(output_b_q_size);
  let name = String::from(name);

  (
    Box::new(
      YSplitWrap{
        name          : name.clone(),
        state         : ysplit,
        input_rx      : ChannelWrapper::ReceiverNotConnected(
          ReceiverChannelId(0),
          ReceiverName (name.clone())
        ),
        output_a_tx   : output_a_tx,
        output_b_tx   : output_b_tx,
      }
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
