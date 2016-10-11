use super::super::{Task, ChannelPosition, ChannelPositionDiff, SenderChannelId};

pub struct TaskWrap {
  task:              Box<Task+Send>,
  output_positions:  Vec<(ChannelPosition, ChannelPositionDiff)>,
}

impl TaskWrap {
  pub fn execute(&mut self) -> Result<(), &'static str> {
    let result = self.task.execute();
    let n_outputs = self.output_positions.len();
    let mut slice = self.output_positions.as_mut_slice();
    for i in 0..n_outputs {
      let old_position = slice[i].0;
      let new_position = self.task.output_channel_pos(SenderChannelId(i));
      let diff = new_position.0 - old_position.0;
      slice[i] = (new_position, ChannelPositionDiff(diff));
    }
    result
  }

  #[allow(dead_code)]
  pub fn output_positions(&self) -> &Vec<(ChannelPosition, ChannelPositionDiff)> {
    &self.output_positions
  }
}

pub fn new(task: Box<Task+Send>) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task:              task,
    output_positions:  vec![(ChannelPosition(0), ChannelPositionDiff(0)); n_outputs],
  }
}
