# acto-rs library

This library is a proof of concept, never run in any production setup and fairly untested. Use at your own risk. You were warned.

---

This library is a mixture of concepts to connect independent pieces together. These
independent pieces can have:

- internal state
- channels to talk to others
- [scheduling rule](./src/lib.rs)

These pieces (may be called actors) are managed by a [scheduler](./src/scheduler/mod.rs) which has a predefined number of threads to run them. The number of input and output channels are determined by the type of the actor. Possible types are:

- [Source](./src/elem/source.rs): 1 output
- [Sink](./src/elem/sink.rs): 1 input
- [Filter](./src/elem/filter.rs): 1 input, 1 output
- [Y-split](./src/elem/ysplit.rs): 1 input, 2 outputs of possible different types
- [Y-merge](./src/elem/ymerge.rs): 2 inputs of possible different types, 1 output
- [Scatter](./src/elem/scatter.rs): 1 input, multiple outputs of the same type
- [Gather](./src/elem/gather.rs): multiple inputs of the same type, 1 output

## License

[MIT](./LICENSE-MIT) or [Apache 2.0](./LICENSE-APACHE)
