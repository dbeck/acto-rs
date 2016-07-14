use super::common::{Schedule};

pub trait Task {
  fn execute(&mut self)  -> Schedule;
  fn name(&self)         -> &String;
}
