
pub fn testme2() {
  use std::sync::atomic;
  use std::mem;
  let ausz = atomic::AtomicUsize::new(42);
  let a : [atomic::AtomicUsize ;1] = [ausz]; // vec![ausz];
  a[0].store(9, atomic::Ordering::Release);
  //let a = [ausz]; // vec![ausz];
  for x in &a {
    println!("42={:?} {:?} {:?}",
      x.load(atomic::Ordering::Acquire),
      mem::align_of_val(x),
      mem::align_of_val(&a) );
  }
}

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

pub fn testme3() {
  use std::sync::atomic::{AtomicUsize, Ordering};

  struct CircularBuffer<T : Copy> {
    seqno : AtomicUsize,
    size  : u16,
    data  : Vec<T>,
    flags : Vec<AtomicUsize>,
    //
    writer_buffer  : u32,
    reader_data    : Vec<u32>,
  }

  impl <T : Copy> CircularBuffer<T> {
    fn new(size : u16, default_value : T) -> CircularBuffer<T> {
      // TODO: check size here!!!
      let mut ret = CircularBuffer {
        seqno : AtomicUsize::new(0),
        size  : size,
        data  : vec![],
        flags : vec![],
        writer_buffer : 0,
        reader_data   : vec![]
      };

      // fill data with default_value 1+(size*2)
      ret.data.reserve(1+(size*2) as usize);
      for _i in 0..(1+(size*2)) {
        ret.data.push(default_value);
      }

      // the initial writer buffer is the last element in data[]
      ret.writer_buffer = (2*size) as u32;

      // fill flags and reader_data
      ret.flags.reserve(size as usize);
      ret.reader_data.reserve(size as usize);
      for i in 0..size {
        ret.flags.push(AtomicUsize::new(i as usize));
        ret.reader_data.push((i + size) as u32)
      }

      ret
    }

    fn put<F>(&mut self, setter: F) -> usize
    where F : Fn(&mut T) {
      // fill the writer_buffer here
      {
        let mut opt : Option<&mut T> = self.data.get_mut(self.writer_buffer as usize);
        match opt.as_mut() {
          Some(v) => setter(v),
          None    => {}
        }
      }
      // check what the old sequence number was
      let old_seqno = self.seqno.load(Ordering::Acquire);

      // calculate new seqno
      let new_seqno : u32 = (old_seqno + 1) as u32;

      // calculate new position
      let new_pos = new_seqno % (self.size as u32);

      // swap the writer_buffer with the new location and calculate new flags
      {
        let old_flag_opts : Option<&mut AtomicUsize> = self.flags.get_mut(new_pos as usize); //.unwrap().load(Ordering::Acquire);
        match old_flag_opts {
          Some(v) => {
            let old_flags = v.load(Ordering::Acquire);
            let old_buffer : u32 = (old_flags as u32) & 0xffff;
            let new_flags  : u64 = (new_seqno << 16) as u64 + (self.writer_buffer as u64);
            v.store(new_flags as usize, Ordering::Relaxed);
            self.writer_buffer = old_buffer;
          },
          None => {}
        };
      }

      self.seqno.store(new_seqno as usize, Ordering::Release);
      old_seqno + 1
    }
  }

  let mut mycb : CircularBuffer<i32> = CircularBuffer::new(100, 0);

  for i in 0..10 {
    println!("out: {} {}",mycb.put(|v| *v = i*1000), mycb.size);
  }
}

pub fn all_tests() {
  testme();
  testme2();
  testme3();
}

#[test]
pub fn it_works() {
  all_tests();
}
