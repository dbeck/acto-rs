# acto-rs library

This library is a proof of concept, never run in any production setup and is fairly untested. Use at your own risk. You were warned.

---

This library is a mixture of concepts to connect independent pieces together to form a data processing pipeline. These independent pieces can have:

- internal state
- typed channels to talk to others
- [scheduling rule](./src/lib.rs)

These pieces (actors) are managed by a [scheduler](./src/scheduler/mod.rs) which has a predefined number of threads to run them. The number of input and output channels are determined by the type of the actor. Possible types are:

- [Source](./src/elem/source.rs): 1 output
- [Sink](./src/elem/sink.rs): 1 input
- [Filter](./src/elem/filter.rs): 1 input, 1 output
- [Y-split](./src/elem/ysplit.rs): 1 input, 2 outputs of possible different types
- [Y-merge](./src/elem/ymerge.rs): 2 inputs of possible different types, 1 output
- [Scatter](./src/elem/scatter.rs): 1 input, multiple outputs of the same type
- [Gather](./src/elem/gather.rs): multiple inputs of the same type, 1 output

The scheduling rule determines when to run an actor:

- Loop - continously, round-robin with the other tasks of the scheduler
- OnMessage - when a message arrives to one of its input channels
- OnExternalEvent - when an external event is delivered via Scheduler::notify(..) (to integrate with MIO for example)
- Periodic(PeriodLengthInUsec) - periodically

## Usage

You need to design the topology of the components because the connections of the components need to be made before they are passed to the scheduler. The scheduler owns the components and you cannot change them afterwards from the outside.

When you pass the components to the scheduler you need to tell it how to schedule their execution based on one of the above rules. Finally you will need to start the scheduler. After you started the scheduler, you can still add new actors to it.

### The crate

```
[dependencies]
acto-rs = "0.5.0"
```

### Overview

- Implement the actors based on one of the elem traits
- Start/stop the scheduler
- Pass the actor instances to the scheduler

### Creating the actors

The actors need to implement one of the traits above. Examples:

- Source: [dummy source](/src/sample/dummy_source.rs)
- Filter: [dummy filter](/src/sample/dummy_source.rs)
- Sink: [dummy sink](/src/sample/dummy_source.rs)

#### Creating a source element

This is a somewhat more realistic element that reads UDP messages from the network and passes it forward to the next element in the topology.

```rust
use actors::*;
use std::net::{UdpSocket, SocketAddr, Ipv4Addr, SocketAddrV4};
use std::io;
use std::mem;

pub struct ReadBytes {
  socket: UdpSocket
}

//
// this item reads 1024 bytes on UDP and passes the data forward with
// the data size and the sender address. if an error happens, then the
// error goes forward instead.
//
impl source::Source for ReadBytes {

  type OutputValue = ([u8; 1024], (usize, SocketAddr));
  type OutputError = io::Error;

  fn process(&mut self,
             output: &mut Sender<Message<Self::OutputValue, Self::OutputError>>,
             _stop: &mut bool)
  {
    output.put(|value| {
      if let &mut Some(Message::Value(ref mut item)) = value {
        // re-use the preallocated space in the queue
        match self.socket.recv_from(&mut item.0) {
          Ok((read_bytes, from_addr)) => {
            item.1 = (read_bytes, from_addr);
          },
          Err(io_error) => {
            // swap in the error message
            let error_message = Some(Message::Error(ChannelPosition(output.seqno()), io_error));
            mem::swap(value, &mut error_message);
          }
        };
      } else {
        // allocate new buffer and swap it in
        let dummy_address  = Ipv4Addr::from(0);
        let dummy_sockaddr = SocketAddrV4::new(dummy_address, 1);
        let item = ([0; 1024],(0, SocketAddr::V4(dummy_sockaddr)));

        match self.socket.recv_from(&mut item.0) {
          Ok((read_bytes, from_addr)) => {
            item.1 = (read_bytes, from_addr);
            let message = Some(Message::Value(item));
            mem::swap(value, &mut message);
          },
          Err(io_error) => {
            // swap in the error message
            let error_message = Some(Message::Error(ChannelPosition(output.seqno()), io_error));
            mem::swap(value, &mut error_message);
          }
        };
      }
    });
  }
}
```

### Starting the scheduler

The scheduler allows adding new tasks while it is running or before it was started. The scheduler can only be started/stoped once. The tasks themselves decide when to stop and they will tell it to the scheduler via the `stop` flag passed to them at execution.

```rust
let mut sched1 = Scheduler::new();
sched1.start(); // this uses one single execution thread
sched1.stop();

// to use more threads, do:
let mut sched_multi = Scheduler::new();
sched_multi.start_with_threads(12);
sched_multi.stop();
```

### Pass the actors to the scheduler

```rust
let mut sched = Scheduler::new();
sched.start_with_threads(4);

// create two dummy tasks
let dummy_queue_size = 2_000;
let (source_task, mut source_out) = source::new( "Source", dummy_queue_size, Box::new(DummySource{}));
let mut sink_task = sink::new( "Sink", Box::new(DummySink{}));

// connect the sink to the source
sink_task.connect(&mut source_out).unwrap();

// add the source and the sink to the scheduler and tell
// the scheduler how to run them
let source_id = sched.add_task(source_task, SchedulingRule::OnExternalEvent).unwrap();
sched.add_task(sink_task, SchedulingRule::OnMessage).unwrap();

// send an example notification to the source task
sched.notify(&source_id).unwrap();

sched.stop();
```

### Project goals

The primary goal is predictable, low latency processing. I don't want to make any performance claims whatsoever. What I can tell is that I invested quite some time into measuring the latency of the components, the scheduler and the resulting pipeline.

Another goal is that I want to keep the overhead of message passing to the minimum. Both the time takes from the sender to do the send operation and also the end-to-end latency, between the start of the send, to the actual reception of the message. I found that the former is under estimated in an (unnamed) actor implementations. I want to keep this overhead to the tens of nanoseconds range.

I want this library to act sensibly under overload. This in practice means (possibly) dropping messages. The components talk using a bounded message queue. The sender can detect if it is about to overwrite a previous message and may act accordingly, but I do believe that this is not the right approach. If the message queue reader lags behind, then it means the system cannot cope with the load. In that case we shouldn't pile up messages, fill up all memory and let the system die.

### Project non-goals

I did take ideas from other actor models, but I don't want to follow them strictly. Erlang/Elixir actor model was a great source of inspiration and I admire their work.

## License

[MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE)
