extern crate acto_rs as actors;
extern crate lossyq;

#[cfg(feature = "bench")]
fn main() {
  use actors::bench;
  bench::run();
}

#[cfg(not(feature = "bench"))]
fn main() {
  println!("not running benchmarks. if you need them add --features \"bench\" flag");
}
