
use super::ext_pipeline_source::ExtPipelineSource;
use super::ext_pipeline_filter::ExtPipelineFilter;
use super::ext_pipeline_sink::ExtPipelineSink;
use super::super::elem::connectable::{Connectable};
use super::super::elem::{source, sink, filter};
use super::super::scheduler::{Scheduler, event};
use super::super::{TaskId};

pub struct ExtPipeline {
  sched:              Scheduler,
  sink_msg_evt:       event::Event,
  source_id:          TaskId,
  wait_ticket:        u64,
}

impl ExtPipeline {
  pub fn new() -> ExtPipeline {
    let mut sched            = Scheduler::new();
    let source_exec_evt  = event::Event::new();
    let filter_exec_evt  = event::Event::new();
    let filter_msg_evt   = event::Event::new();
    let sink_exec_evt    = event::Event::new();
    let sink_msg_evt     = event::Event::new();

    let (source_task, mut source_out) =
      source::new( "Source", 20, Box::new(ExtPipelineSource::new(source_exec_evt.clone())));

    let (mut filter_task, mut filter_out) =
      filter::new( "Filter", 20, Box::new(ExtPipelineFilter::new(filter_exec_evt.clone(), filter_msg_evt.clone())));

    let mut sink_task =
      sink::new( "Sink", Box::new(ExtPipelineSink::new(sink_exec_evt.clone(), sink_msg_evt.clone())));

    filter_task.connect(&mut source_out).unwrap();
    sink_task.connect(&mut filter_out).unwrap();

    let source_id = sched.add_task(source_task).unwrap();
    let _filter_id = sched.add_task(filter_task);
    let _sink_id = sched.add_task(sink_task);

    ExtPipeline {
      sched:              sched,
      sink_msg_evt:       sink_msg_evt,
      source_id:          source_id,
      wait_ticket:        0,
    }
  }

  pub fn start(&mut self) {
    self.sched.start();
  }

  pub fn stop(&mut self) {
    self.sched.stop();
  }

  pub fn notify(&mut self) {
    loop {
      let source_id = self.source_id;
      if let Ok(_result) = self.sched.notify(&source_id) {
        break;
      }
    }
  }

  pub fn wait(&mut self) {
    loop {
      let res = self.sink_msg_evt.wait(self.wait_ticket, 10_000_000);
      if res > self.wait_ticket {
        self.wait_ticket = res;
        break;
      }
    }
  }
}
