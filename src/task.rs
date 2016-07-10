use super::common::{Schedule};

pub trait Task {
  fn execute(&mut self) -> Schedule;
  fn input_names(&self) -> &Vec<String>;
  fn output_names(&self) -> &Vec<String>;
}
