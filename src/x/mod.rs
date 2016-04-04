
pub fn testme() {

  use std::fmt;

  trait Info {
    fn sz(&self) -> usize;
    fn print(&self);
  }

  struct Buffer<T> {
    buf: Vec<T>,
  }

  impl<T: fmt::Debug> Info for Buffer<T> {
    fn sz(&self) -> usize {
      self.buf.len()
    }

    fn print(&self) {
      for i in &self.buf {
        println!("trait: {:?}", i);
      }
    }
  }

  let int_buf    = Buffer { buf: vec![1,   2,   3] };
  let float_buf  = Buffer { buf: vec![1.1, 2.1, 3.1] };

  let buffers = vec![ &int_buf   as &Info,
                      &float_buf as &Info ];

  for i in &buffers {
    println!("sz: {}",i.sz());
    i.print();
  }
}

pub fn testme2() {
  use std::sync::atomic;
  let ausz = atomic::AtomicUsize::new(42);
  let a : [atomic::AtomicUsize ;1] = [ausz]; // vec![ausz];
  for x in &a {
    println!("42={:?}", x.load(atomic::Ordering::Relaxed) );
  }
}

#[test]
pub fn it_works() {
  testme();
  testme2();
}
