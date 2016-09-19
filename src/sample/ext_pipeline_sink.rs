
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule};
use super::super::scheduler::event;

#[allow(dead_code)]
pub struct ExtPipelineSink {
  on_exec:  event::Event,
  on_msg:   event::Event,
}

impl sink::Sink for ExtPipelineSink {
  type InputType = usize;

  fn process(&mut self, input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    self.on_exec.notify();
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for _m in receiver.iter() {
        self.on_msg.notify();
      }
      // only execute when there is a new message on the input channel
      Schedule::OnMessage(*channel_id)
    } else {
      Schedule::Stop
    }
  }
}

impl ExtPipelineSink {
  pub fn new(on_exec: event::Event, on_msg: event::Event) -> ExtPipelineSink {
    ExtPipelineSink{
      on_exec: on_exec,
      on_msg: on_msg,
    }
  }
}

pub fn new(on_exec: event::Event, on_msg: event::Event) -> ExtPipelineSink {
  ExtPipelineSink::new(on_exec, on_msg)
}
