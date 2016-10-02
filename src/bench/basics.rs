use super::bench_200ms;

use lossyq::spsc::{channel, Sender};
use super::super::elem::{source, /*, filter, sink, ymerge, ysplit*/ };
use super::super::{Task};
use super::super::sample::dummy_source::{DummySource};
use super::super::scheduler::event::{Event};
use super::spinner;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

fn time_baseline() {
  bench_200ms("time-baseline", |_v| {} );
}

fn spinner() {
  let sp = spinner::Spinner::new();
  let counter = sp.get();
  bench_200ms("spinner", |_v| {
    let _x = counter.load(Ordering::Acquire);
  });
  sp.stop();
}

fn atomic_ops() {
  let val = AtomicUsize::new(0);
  bench_200ms("fetch-add-relaxed", |_v| {
    val.fetch_add(1, Ordering::Relaxed);
  });
  bench_200ms("fetch-add-seqcst", |_v| {
    val.fetch_add(1, Ordering::SeqCst);
  });
  bench_200ms("fetch-add-acqrel", |_v| {
    val.fetch_add(1, Ordering::AcqRel);
  });
}

fn hash_map_10() {
  let mut m = HashMap::new();
  for i in 0..10u64 {
    m.insert(i, i);
  }
  bench_200ms("hash-map-nonex", |v| {
    let _x = m.get(&v);
  });
  bench_200ms("hash-map-seq", |v| {
    let k = v%10;
    let _x = m.get(&k);
  });
}

fn lossyq_send() {
  let (mut tx, _rx) = channel(100);
  bench_200ms("lossyq-send", |i| {
    tx.put(|v| *v = Some(i));
  });
}

fn lossyq_recv() {
  let (mut _tx, mut rx) = channel::<u64>(100);
  bench_200ms("lossyq-recv", |_i| {
    for _ii in rx.iter() {
    }
  });
}

fn lossyq_send_recv() {
  let (mut tx, mut rx) = channel(100);
  bench_200ms("lossyq-send-recv", |i| {
    tx.put(|v| *v = Some(i));
    for _i in rx.iter() {
    }
  });
}

fn lossyq_send_recv_3() {
  let (mut tx, mut rx) = channel(100);
  bench_200ms("lossyq-send-recv3", |i| {
    tx.put(|v| *v = Some(i));
    tx.put(|v| *v = Some(i));
    tx.put(|v| *v = Some(i));
    for _i in rx.iter() {
    }
  });
}


fn mpsc_send() {
  use std::sync::mpsc;
  let (tx, _rx) = mpsc::channel();
  bench_200ms("mpsc-send", |i| {
    tx.send(i).unwrap();
  });
}

fn mpsc_recv() {
  use std::sync::mpsc;
  let (tx, rx) = mpsc::channel();
  tx.send(0u64).unwrap();
  bench_200ms("mpsc-recv", |_i| {
    let _tr = rx.try_recv();
  });
}

fn mpsc_send_recv() {
  use std::sync::mpsc;
  let (tx, rx) = mpsc::channel();
  bench_200ms("mpsc-send-recv", |i| {
    tx.send(i).unwrap();
    rx.recv().unwrap();
  });
}

fn mpsc_send_recv_3() {
  use std::sync::mpsc;
  let (tx, rx) = mpsc::channel();
  bench_200ms("mpsc-send-recv3", |i| {
    tx.send(i).unwrap();
    tx.send(i).unwrap();
    tx.send(i).unwrap();
    for _i in 0..3 {
      let _x = rx.try_recv();
    }
  });
}

fn indirect_send_data() {
  let (mut tx, _rx) = channel(100);
  let sender = |val: u64, chan: &mut Sender<u64>| {
    chan.put(|v: &mut Option<u64>| *v = Some(val));
  };
  bench_200ms("indirect-send", |i| { sender(i, &mut tx); });
}

fn locked_data() {
  use std::sync::{Arc, Mutex};
  let locked = Arc::new(Mutex::new(0u64));
  bench_200ms("std::mutex", |_i| {
    let mut _x = locked.lock().unwrap();
  });
}

fn locked_send_data() {
  use std::sync::{Arc, Mutex};
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  bench_200ms("std::mutex+send", |i| {
    let mut x = locked.lock().unwrap();
    x.put(|v| *v = Some(i));
  });
}

fn lotted_data() {
  use std::sync::{Arc};
  use parking_lot::Mutex;
  let locked = Arc::new(Mutex::new(0u64));
  bench_200ms("parking_lot", |_i| {
    let mut _x = locked.lock();
  });
}

fn lotted_send_data() {
  use std::sync::{Arc};
  use parking_lot::Mutex;
  let (tx, _rx) = channel(100);
  let locked = Arc::new(Mutex::new(tx));
  bench_200ms("parking_lot+send", |i| {
    let mut x = locked.lock();
    x.put(|v| *v = Some(i));
  });
}

fn source_execute() {
  let (mut source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(DummySource{}));

  bench_200ms("source-execute", |_i| {
    source_task.execute().unwrap();
  });
}

fn source_execute_with_swap() {
  use std::sync::atomic::{AtomicPtr, Ordering};
  use std::ptr;

  let (source_task, mut _source_out) =
    source::new( "Source", 2, Box::new(DummySource{}));
  let source_ptr = AtomicPtr::new(Box::into_raw(source_task));

  bench_200ms("source-execute-w-swap", |_i| {
    let old_ptr = source_ptr.swap(ptr::null_mut(), Ordering::AcqRel);
    unsafe { (*old_ptr).execute().unwrap(); }
    source_ptr.swap(old_ptr,  Ordering::AcqRel);
  });

  let _bx = unsafe { Box::from_raw(source_ptr.swap(ptr::null_mut(), Ordering::AcqRel)) };
}

fn event_notify() {
  let mut evt = Event::new();
  bench_200ms("event-notify", |_i| {
    evt.notify();
  });
}

fn event_notify_wait() {
  let mut evt = Event::new();
  let mut ticket = 0u64;
  bench_200ms("event-notify-wait", |_i| {
    evt.notify();
    ticket = evt.wait(ticket, 10_000_000_000);
  });
}

fn event_mt_notify() {
  let mut wait_us = 10_000_000;
  let mut ticket = 0u64;
  // event to measure
  let mut evt = Event::new();
  let mut evt_reader = evt.clone();
  // synchronize start
  let mut start_evt = Event::new();
  let mut start_sig = start_evt.clone();
  // synchronize end
  let mut end_evt = Event::new();
  let mut end_sig = end_evt.clone();

  let notifier = thread::spawn(move|| {
    start_sig.wait(0, 10_000_000);
    bench_200ms("event-mt-notify", |_i| {
      evt.notify();
    });
    end_evt.notify();
  });

  start_evt.notify();
  loop {
    let res = evt_reader.wait(ticket, wait_us);
    if ticket == res {
      if end_sig.wait(0, 1) > 0 {
        break;
      }
    }
    ticket = res;
    wait_us = 1;
  }
  notifier.join().unwrap();
}

pub fn run() {
  time_baseline();
  spinner();
  atomic_ops();
  hash_map_10();
  lossyq_send();
  lossyq_recv();
  lossyq_send_recv();
  lossyq_send_recv_3();
  mpsc_send();
  mpsc_recv();
  mpsc_send_recv();
  mpsc_send_recv_3();
  indirect_send_data();
  locked_data();
  locked_send_data();
  lotted_data();
  lotted_send_data();
  source_execute();
  source_execute_with_swap();
  event_notify();
  event_notify_wait();
  event_mt_notify();
}
