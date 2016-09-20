
use super::super::elem::sink;
use super::super::{ChannelWrapper, Schedule, Message};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use super::tick::Tick;

pub struct MeasuredPipelineSink {
  latency:       u64,
  count:         u64,
  exec:          u64,
  start:         usize,
  spinned:       Arc<AtomicUsize>,
  elapsed:       Tick,
  others_spins:  u64,
  last_spin:     usize,
}

impl sink::Sink for MeasuredPipelineSink {
  type InputType = usize;

  fn process(&mut self, input: &mut ChannelWrapper<Self::InputType>) -> Schedule {
    if let &mut ChannelWrapper::ConnectedReceiver(ref mut channel_id,
                                                  ref mut receiver,
                                                  ref mut _sender_name) = input {
      let now = self.spinned.load(Ordering::Acquire);
      self.exec += 1;
      for m in receiver.iter() {
        if let Message::Value(tick) = m {
          self.latency  += (now - tick as usize) as u64;
          self.count    += 1;
        }
      }
      // only execute when there is a new message on the input channel
      let end = self.spinned.load(Ordering::Acquire);
      if self.last_spin != 0 {
        self.others_spins += (now - self.last_spin) as u64;
      }
      self.last_spin = end;
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
      latency:      0,
      exec:         0,
      count:        0,
      start:        spinned.load(Ordering::Acquire),
      spinned:      spinned,
      elapsed:      Tick::new(),
      others_spins: 0,
      last_spin:    0,
    }
  }
}

pub fn new(spinned:  Arc<AtomicUsize>) -> MeasuredPipelineSink {
  MeasuredPipelineSink::new(spinned)
}

impl Drop for MeasuredPipelineSink {
  fn drop(&mut self) {
    let ns = self.elapsed.elapsed_ns();
    let now = self.spinned.load(Ordering::Acquire);
    println!(" @drop MeasuredPipelineSink avg latency {} spins, count:{} ns/count:{} spin/count:{} exec:{} others_spins:{}",
      self.latency/self.count,
      self.count,
      ns/self.count,
      (now-self.start) as u64/self.count,
      self.exec,
      self.others_spins/self.exec
    );
  }
}
