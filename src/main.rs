extern crate minions;
extern crate lossyq;

use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::{scheduler, source, filter, sink, ymerge, ysplit};
use minions::common;
use minions::common::Message;
use minions::task::Task;
use std::collections::HashMap;
use std::any::Any;

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

// Option<IdentifiedReceiver<Output>>

fn main() {

  let source_state: Box<source::Source<OutputType=i32>>                = Box::new(SourceState{state:0});
  let filter_state: Box<filter::Filter<InputType=i32,OutputType=i32>>  = Box::new(FilterState{state:0});
  let sink_state:   Box<sink::Sink<InputType=i32>>                     = Box::new(SinkState{state:0});
  let ysplit_state: Box<ysplit::YSplit<InputType=i32,OutputTypeA=i32,OutputTypeB=f64>>
    = Box::new(YSplitState{state_i:0, state_f:0.0});
  let ymerge_state: Box<ymerge::YMerge<InputTypeA=i32,InputTypeB=f64,OutputType=i32>>
    = Box::new(YMergeState{state_i:0, state_f:0.0});

  let (source_task, source_out)  = source::new( "Source", 2, source_state);
  let (mut filter_task, mut _filter_out)  = filter::new( "Filter", 2, filter_state);
  let (mut ysplit_task, mut _ysplit_out_a, mut _ysplit_out_b) = ysplit::new( "YSplit", 2, 2, ysplit_state);
  let (mut ymerge_task, mut _ymerge_out) = ymerge::new( "YMerge", 2, ymerge_state);
  let mut sink_task    = sink::new( "Sink", sink_state);

  let mut receivers : HashMap<String, Box<Any>> = HashMap::new();

  receivers.insert(source_task.name().clone(), Box::new(*source_out));

  {
    let _filter_in  = filter_task.input();
  }
  {
    let _ysplit_in   = ysplit_task.input();
  }
  {
    let _ymerge_in_a   = ymerge_task.input_a();
  }
  {
    let _ymerge_in_b   = ymerge_task.input_b();
  }
  {
    let _sink_in    = sink_task.input();
  }
  {
    let mut s = scheduler::new();
    let ss2: Box<source::Source<OutputType=i32>> = Box::new(SourceState{state:0});
    let (t, o)  = source::new( "Source", 2, ss2);
    s.add_source(t,o);
  }
}
