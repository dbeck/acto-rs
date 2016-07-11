use super::common::{Schedule};

pub trait Task {
  fn execute(&mut self) -> Schedule;
  fn name(&self) -> &String;
  fn input_names(&self) -> &Vec<String>;
  fn output_names(&self) -> &Vec<String>;
}
