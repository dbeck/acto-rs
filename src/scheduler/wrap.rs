use super::super::{Task, ChannelPosition, TaskId, SenderChannelId, ChannelId};
use super::prv::{Private};

pub struct TaskWrap {
  task:              Box<Task+Send>,
  output_positions:  Vec<(ChannelPosition, TaskId)>,
}

impl TaskWrap {
  #[inline(always)]
  pub fn execute(&mut self,
                 has_dependents: bool,
                 stop: &mut bool,
                 private_data: &mut Private)
  {
    self.task.execute(stop);
    if has_dependents {
      let n_outputs = self.output_positions.len();
      let slice = self.output_positions.as_mut_slice();
      for i in 0..n_outputs {
        let old_position = slice[i].0;
        let new_position = self.task.output_channel_pos(SenderChannelId(i));
        if old_position.0 < new_position.0 {
          private_data.save_trigger(slice[i].1);
        }
        slice[i].0 = self.task.output_channel_pos(SenderChannelId(i));
      }
    }
  }

  pub fn register_dependents(&mut self,
                             deps: Vec<(ChannelId, TaskId)>)
  {
    let n_pos = self.output_positions.len();
    let slice = self.output_positions.as_mut_slice();
    for dep in deps {
      let ch_id = dep.0;
      let idx = ch_id.sender_id.0;
      if idx < n_pos {
        slice[idx].1 = dep.1;
      }
    }
  }
}

pub fn new(task: Box<Task+Send>) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task:              task,
    output_positions:  vec![(ChannelPosition(0), TaskId(0)); n_outputs],
  }
}
