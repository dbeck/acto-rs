extern crate minions;
extern crate lossyq;

use minions::{scheduler, source, filter, sink};
use lossyq::spsc::Receiver;
use lossyq::spsc::Sender;
use minions::common;
use minions::common::Message;

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

fn main() {
  let source_state: Box<source::Source<OutputType=i32>>                = Box::new(SourceState{state:0});
  let filter_state: Box<filter::Filter<InputType=i32,OutputType=i32>>  = Box::new(FilterState{state:0});
  let sink_state:   Box<sink::Sink<InputType=i32>>                     = Box::new(SinkState{state:0});

  let mut source_task  = source::new( "Source", 2, source_state);
  let mut filter_task  = filter::new( "Filter", 2, filter_state);
  let mut sink_task    = sink::new( "Sink", sink_state);

  {
    let _source_out = source_task.output();
    let _filter_in  = filter_task.input();
  }
  {
    let _filter_out = filter_task.output();
    let _sink_in    = sink_task.input();
  }
  {
    let _s = scheduler::new();
  }
}
