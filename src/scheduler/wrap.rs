use super::super::{Task, ChannelPosition, SenderChannelId};

pub struct TaskWrap {
  task:              Box<Task+Send>,
  output_positions:  Vec<ChannelPosition>,
}

impl TaskWrap {
  pub fn execute(&mut self) -> Result<(), &'static str> {
    let result = self.task.execute();
    let n_outputs = self.output_positions.len();
    let mut slice = self.output_positions.as_mut_slice();
    for i in 0..n_outputs {
      slice[i] = self.task.output_channel_pos(SenderChannelId(i));
    }
    result
  }

  #[allow(dead_code)]
  pub fn output_positions(&self) -> &Vec<ChannelPosition> {
    &self.output_positions
  }
}

pub fn new(task: Box<Task+Send>) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task:              task,
    output_positions:  vec![ChannelPosition(0); n_outputs],
  }
}
