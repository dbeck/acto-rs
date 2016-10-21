# acto-rs library

This library is a proof of concept, never run in any production setup and fairly untested. Use at your own risk. You were warned.

---

This library is a mixture of concepts to connect independent pieces together. These independent pieces can have:

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

### The crate

```
[dependencies]
acto-rs = "0.4.0"
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

### Starting the scheduler

The scheduler allows adding new tasks while it is running or before it was started. The scheduler can only be started/stoped once. The tasks themselves decide when to stop and they will tell it to the scheduler via the `stop` flag passed to them at execution.

```rust
let mut sched = Scheduler::new();
sched.start();
sched.stop();
```

## License

[MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE)
