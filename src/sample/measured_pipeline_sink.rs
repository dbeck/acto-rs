
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule, Message};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use super::tick::Tick;

pub struct MeasuredPipelineSink {
  latency:  u64,
  count:    u64,
  spinned:  Arc<AtomicUsize>,
  elapsed:  Tick,
}

impl sink::Sink for MeasuredPipelineSink {
  type InputType = usize;

  fn process(&mut self, input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      for m in receiver.iter() {
        if let Message::Value(tick) = m {
          let now = self.spinned.load(Ordering::Acquire);
          self.latency  += (now - tick as usize) as u64;
          self.count    += 1;
        }
      }
      // only execute when there is a new message on the input channel
      Schedule::OnMessage(*channel_id)
      //Schedule::Loop
    } else {
      Schedule::Stop
    }
  }
}

impl MeasuredPipelineSink {
  pub fn new(spinned:  Arc<AtomicUsize>) -> MeasuredPipelineSink {
    MeasuredPipelineSink{
      latency:  0,
      count:    0,
      spinned:  spinned,
      elapsed:  Tick::new(),
    }
  }
}

pub fn new(spinned:  Arc<AtomicUsize>) -> MeasuredPipelineSink {
  MeasuredPipelineSink::new(spinned)
}

impl Drop for MeasuredPipelineSink {
  fn drop(&mut self) {
    let ns = self.elapsed.elapsed_ns();
    println!(" @drop MeasuredPipelineSink avg latency {} spins, count:{} ns/count:{} spin/count:{}",
      self.latency/self.count,
      self.count,
      ns/self.count,
      self.spinned.load(Ordering::Acquire) as u64/self.count
    );
  }
}
