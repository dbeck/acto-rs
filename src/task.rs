use super::common::{Schedule};
use super::channel_id::{Id};

pub trait Task {
  fn execute(&mut self)  -> Schedule;
  fn name(&self)         -> &String;
  fn input_names(&self)  -> &Vec<Id>;
  fn output_names(&self) -> &Vec<Id>;
}
