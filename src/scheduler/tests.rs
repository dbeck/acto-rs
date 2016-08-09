
#[test]
fn event_test() {
  use std::thread;

  let mut e1 = super::event::new();
  let mut e2 = e1.clone();

  let t = thread::spawn(move|| {
    assert_eq!(e1.wait(0, 200_000), 1);
  });

  e2.notify();
  t.join().unwrap();
}
