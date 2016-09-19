
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule};
use super::super::scheduler::event;

pub struct MeasuredPipelineSink {
  on_exec:  event::Event,
  on_msg:   event::Event,
}

impl sink::Sink for MeasuredPipelineSink {
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

impl MeasuredPipelineSink {
  pub fn new(on_exec: event::Event, on_msg: event::Event) -> MeasuredPipelineSink {
    MeasuredPipelineSink{
      on_exec: on_exec,
      on_msg: on_msg,
    }
  }
}

pub fn new(on_exec: event::Event, on_msg: event::Event) -> MeasuredPipelineSink {
  MeasuredPipelineSink::new(on_exec, on_msg)
}

impl Drop for MeasuredPipelineSink {
  fn drop(&mut self) {
    let (_r, exec_count) = self.on_exec.ready(0);
    let (_r, msg_count)  = self.on_msg.ready(0);
    println!(" @drop MeasuredPipelineSink exec_count:{} msg_count:{}",exec_count, msg_count);
  }
}
