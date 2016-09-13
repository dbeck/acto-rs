
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule};
use super::super::scheduler::event;

#[allow(dead_code)]
pub struct EvtSink {
  on_exec:  event::Event,
  on_msg:   event::Event,
}

impl sink::Sink for EvtSink {
  type InputType = usize;

  fn process(&mut self, input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    self.on_exec.notify();
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut _channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for _m in receiver.iter() {
        self.on_msg.notify();
      }
    }
    Schedule::Loop
  }
}

pub fn new(on_exec: event::Event, on_msg: event::Event) -> EvtSink {
  EvtSink{
    on_exec: on_exec,
    on_msg: on_msg,
  }
}
