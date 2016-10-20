use super::bench_200ms;

struct BitSet {
  bits:  Vec<u64>,
}

impl BitSet {
  fn new(count: usize) -> BitSet {
    let count64 = 1 + (count/64);

    BitSet {
      bits: vec![0; count64],
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

  fn for_each<F>(&mut self, mut fun: F)
    where F: FnMut(usize, u64)
  {
    let max = self.bits.len();
    let slice = self.bits.as_mut_slice();
    for i in 0..max {
      fun(i, slice[i]);
    }
  }
}

pub fn run() {
  let mut bs = BitSet::new(4096);
  bench_200ms("set-bitset", |i| {
    bs.set((i%4096) as usize);
  });

  bench_200ms("clear-bitset", |_i| {
    bs.clear_all();
  });

  bench_200ms("foreach-bitset", |_i| {
    bs.for_each(|_pos, _val| {} );
  });

  bench_200ms("new-bitset", |_i| {
    bs = BitSet::new(4096);
  });

}
