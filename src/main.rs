extern crate acto_rs as actors;
extern crate lossyq;
extern crate parking_lot;
extern crate libc;

#[cfg(feature = "bench")]
fn main() {
  use actors::bench;
  bench::run();
  println!("hello from bench");
}

#[cfg(not(feature = "bench"))]
fn main() {
  println!("not running benchmarks. if you need them add --features \"bench\" flag");
}
