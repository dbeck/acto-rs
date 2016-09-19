
use lossyq::spsc::Sender;
use super::super::elem::filter;
use super::super::{ChannelWrapper, Message, Schedule};
use super::super::scheduler::event;
use super::tick::{Tick};

pub struct MeasuredPipelineFilter {
  on_exec:  event::Event,
  on_msg:   event::Event,
}

impl filter::Filter for MeasuredPipelineFilter {
  type InputType  = Tick;
  type OutputType = Tick;

  fn process(
    &mut self,
    input:   &mut ChannelWrapper<Self::InputType>,
    output:  &mut Sender<Message<Self::OutputType>>) -> Schedule
  {
    self.on_exec.notify();
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for m in receiver.iter() {
        self.on_msg.notify();
        output.put(|v| *v = Some(m));
      }
      // only execute when there is a new message on the input channel
      Schedule::OnMessage(*channel_id)
    } else {
      Schedule::Stop
    }
  }
}

impl MeasuredPipelineFilter {
  pub fn new(on_exec: event::Event, on_msg: event::Event) -> MeasuredPipelineFilter {
    MeasuredPipelineFilter{
      on_exec: on_exec,
      on_msg: on_msg,
    }
  }
}

pub fn new(on_exec: event::Event, on_msg: event::Event) -> MeasuredPipelineFilter {
  MeasuredPipelineFilter::new(on_exec, on_msg)
}

impl Drop for MeasuredPipelineFilter {
  fn drop(&mut self) {
    let (_r, exec_count) = self.on_exec.ready(0);
    let (_r, msg_count)  = self.on_msg.ready(0);
    println!(" @drop MeasuredPipelineFilter exec_count:{} msg_count:{}",exec_count, msg_count);
  }
}
