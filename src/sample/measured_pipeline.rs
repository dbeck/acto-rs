
use super::measured_pipeline_source::MeasuredPipelineSource;
use super::measured_pipeline_filter::MeasuredPipelineFilter;
use super::measured_pipeline_sink::MeasuredPipelineSink;
use super::super::elem::connectable::{Connectable};
use super::super::elem::{source, sink, filter};
use super::super::scheduler::{Scheduler, event};
use super::super::{TaskId};

pub struct MeasuredPipeline {
  sched:              Scheduler,
  sink_msg_evt:       event::Event,
  source_id:          TaskId,
  wait_ticket:        u64,
  sent:               u64,
}

impl MeasuredPipeline {
  pub fn new() -> MeasuredPipeline {
    let mut sched            = Scheduler::new();
    let source_exec_evt  = event::Event::new();
    let filter_exec_evt  = event::Event::new();
    let filter_msg_evt   = event::Event::new();
    let sink_exec_evt    = event::Event::new();
    let sink_msg_evt     = event::Event::new();

    let (source_task, mut source_out) =
      source::new( "Source", 20000000, Box::new(MeasuredPipelineSource::new(source_exec_evt.clone())));

    let (mut filter_task, mut filter_out) =
      filter::new( "Filter", 20000000, Box::new(MeasuredPipelineFilter::new(filter_exec_evt.clone(), filter_msg_evt.clone())));

    let mut sink_task =
      sink::new( "Sink", Box::new(MeasuredPipelineSink::new(sink_exec_evt.clone(), sink_msg_evt.clone())));

    filter_task.connect(&mut source_out).unwrap();
    sink_task.connect(&mut filter_out).unwrap();

    // reverse order to make sure, dependent task names cannot
    // be resolved immediately. this is to trigger the name
    // resolution code path
    let _sink_id = sched.add_task(sink_task);
    let _filter_id = sched.add_task(filter_task);
    let source_id = sched.add_task(source_task).unwrap();

    MeasuredPipeline {
      sched:              sched,
      sink_msg_evt:       sink_msg_evt,
      source_id:          source_id,
      wait_ticket:        0,
      sent:               0,
    }
  }

  pub fn start(&mut self) {
    self.sched.start_with_threads(1);
  }

  pub fn stop(&mut self) {
    self.sched.stop();
  }

  pub fn notify(&mut self) {
    let source_id = self.source_id;
    if let Err(e) = self.sched.notify(&source_id) {
      println!("couldn't notify {:?}, err: {:?}",source_id, e);
    } else {
      self.sent += 1;
    }
  }

  pub fn wait(&mut self) {
    /*
    use std::thread;
    let mut c = 0;
    loop {
      //let (_ready, res) = self.sink_msg_evt.ready(self.wait_ticket);
      let res = self.sink_msg_evt.wait(self.wait_ticket, 1_00);
      if res > self.wait_ticket {
        self.wait_ticket = res;
        break;
      } else {
        thread::yield_now();
        c += 1;
        if c > 10 {
          break;
        }
      }
    }
    */
    use std::thread;
    thread::yield_now();
  }
}

impl Drop for MeasuredPipeline {
  fn drop(&mut self) {
    let (_r, recvd_count) = self.sink_msg_evt.ready(0);
    println!(" @drop MeasuredPipeline recvd_count:{} tkt:{} sent:{}",
      recvd_count, self.wait_ticket, self.sent);
  }
}
