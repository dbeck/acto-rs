
use lossyq::spsc::Sender;
use super::super::elem::filter;
use super::super::{ChannelWrapper, Message, Schedule};
use super::super::scheduler::event;

#[allow(dead_code)]
pub struct EvtFilter {
  on_exec:  event::Event,
  on_msg:   event::Event,
}

impl filter::Filter for EvtFilter {
  type InputType = usize;
  type OutputType = usize;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputType>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule
  {
    self.on_exec.notify();
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut _channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for m in receiver.iter() {
        self.on_msg.notify();
        output.put(|v| *v = Some(m));
      }
    }
    Schedule::Loop
  }
}

pub fn new(on_exec: event::Event, on_msg: event::Event) -> EvtFilter {
  EvtFilter{
    on_exec: on_exec,
    on_msg: on_msg,
  }
}
