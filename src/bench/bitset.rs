use super::bench_200ms;

struct BitSet {
  bits:     Vec<u64>,
  count64:  usize,
}

impl BitSet {
  fn new(count: usize) -> BitSet {
    let count64 = 2+(count/64);

    BitSet {
      bits:     vec![0; count64],
      count64:  count64,
    }
  }

  fn set(&mut self, at: usize) {
    let segment = at/64;
    let offset  = at%64;
    let mut slice = self.bits.as_mut_slice();
    slice[segment] |= 1<<offset;
  }

  fn clear_all(&mut self) {
    for i in &mut self.bits {
      *i = 0;
    }
  }
}

pub fn run() {
  let mut bs = BitSet::new(4096);
  bench_200ms("bit-set", |i| {
    bs.set((i%4096) as usize);
  });

  bench_200ms("new-bitset", |_i| {
    bs = BitSet::new(4096);
  });

  bench_200ms("clear-bitset", |_i| {
    bs.clear_all();
  });
}
