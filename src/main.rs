extern crate minions;
extern crate lossyq;

use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::{/*scheduler, */ source, filter, sink, ymerge, ysplit};
use minions::common;
use minions::common::Message;
//use minions::task::Task;

#[derive(Copy, Clone)]
struct SourceState {
  state : i32,
}

impl source::Source for SourceState {
  type OutputType = i32;

  fn process(
        &mut self,
        output: &mut Sender<Message<Self::OutputType>>)
      -> common::Schedule {
    output.put(|x| *x = Message::Value(self.state));
    self.state += 1;
    common::Schedule::Loop
  }
}

#[derive(Copy, Clone)]
struct FilterState {
  state : i32,
}

impl filter::Filter for FilterState {
  type InputType = i32;
  type OutputType = i32;

  fn process(
        &mut self,
        input: &mut Receiver<Message<Self::InputType>>,
        output: &mut Sender<Message<Self::OutputType>>)
      -> common::Schedule {
    for i in input.iter() {
      match i {
        Message::Value(v) => {
          self.state = v;
          output.put(|x| *x = Message::Value(self.state));
        }
        _ => { println!("Unknown value"); }
      }
    }
    common::Schedule::Loop
  }
}

#[derive(Copy, Clone)]
struct SinkState {
  state : i32,
}

impl sink::Sink for SinkState {
  type InputType = i32;

  fn process(
        &mut self,
        input: &mut Receiver<Message<Self::InputType>>)
      -> common::Schedule {
    for i in input.iter() {
      match i {
        Message::Value(v) => {
          self.state = v;
        }
        _ => { println!("Unknown value"); }
      }
    }
    common::Schedule::Loop
  }
}

#[derive(Copy, Clone)]
struct YSplitState {
  state_i : i32,
  state_f : f64,
}

impl ysplit::YSplit for YSplitState {
  type InputType    = i32;
  type OutputTypeA  = i32;
  type OutputTypeB  = f64;

  fn process(
        &mut self,
        input:     &mut Receiver<Message<Self::InputType>>,
        output_a:  &mut Sender<Message<Self::OutputTypeA>>,
        output_b:  &mut Sender<Message<Self::OutputTypeB>>) -> common::Schedule
  {
    for i in input.iter() {
      match i {
        Message::Value(v) => {
          self.state_i = v;
          self.state_f = v as f64;
          output_a.put(|x| *x = Message::Value(self.state_i));
          output_b.put(|x| *x = Message::Value(self.state_f));
        }
        _ => { println!("Unknown value"); }
      }
    }
    common::Schedule::Loop
  }
}

#[derive(Copy, Clone)]
struct YMergeState {
  state_i : i32,
  state_f : f64,
}

impl ymerge::YMerge for YMergeState {
  type InputTypeA  = i32;
  type InputTypeB  = f64;
  type OutputType  = i32;

  fn process(
        &mut self,
        input_a: &mut Receiver<Message<Self::InputTypeA>>,
        input_b: &mut Receiver<Message<Self::InputTypeB>>,
        output: &mut Sender<Message<Self::OutputType>>)
      -> common::Schedule {
    for i in input_a.iter() {
      match i {
        Message::Value(v) => {
          self.state_i = v;
          output.put(|x| *x = Message::Value(self.state_i));
        }
        _ => { println!("Unknown value"); }
      }
    }
    for i in input_b.iter() {
      match i {
        Message::Value(v) => {
          self.state_f = v as f64;
          output.put(|x| *x = Message::Value(self.state_f as i32));
        }
        _ => { println!("Unknown value"); }
      }
    }
    common::Schedule::Loop
  }
}

fn main() {
  use minions::connectable::{Connectable, ConnectableY};

  let (mut _source_task, mut source_out) = source::new( "Source", 2, Box::new(SourceState{state:0}));
  let (mut filter_task, mut filter_out) = filter::new( "Filter", 2, Box::new(FilterState{state:0}));
  let (mut ysplit_task, mut ysplit_out_a, mut ysplit_out_b) = ysplit::new( "YSplit", 2, 2, Box::new(YSplitState{state_i:0, state_f:0.0}));
  let (mut ymerge_task, mut ymerge_out) = ymerge::new( "YMerge", 2, Box::new(YMergeState{state_i:0, state_f:0.0}));
  let mut sink_task = sink::new( "Sink", Box::new(SinkState{state:0}));

  filter_task.connect(&mut source_out).unwrap();
  ysplit_task.connect(&mut filter_out).unwrap();
  ymerge_task.connect_a(&mut ysplit_out_a).unwrap();
  ymerge_task.connect_b(&mut ysplit_out_b).unwrap();
  sink_task.connect(&mut ymerge_out).unwrap();
}
