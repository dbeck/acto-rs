use super::super::{Task, ChannelPosition, TaskId, SenderChannelId, ChannelId};
use scheduler::thread_private::ThreadPrivate;

pub struct TaskAndOutputs {
  task:              Box<Task+Send>,
  output_positions:  Vec<(ChannelPosition, TaskId)>,
}

impl TaskAndOutputs {
  #[inline(always)]
  pub fn execute(&mut self,
                 has_dependents: bool,
                 stop: &mut bool,
                 private_data: &mut ThreadPrivate)
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
                             dependents: Vec<(ChannelId, TaskId)>)
  {
    let n_pos = self.output_positions.len();
    let slice = self.output_positions.as_mut_slice();
    for dependent in dependents {
      let ch_id = dependent.0;
      let idx = ch_id.sender_id.0;
      if idx < n_pos {
        slice[idx].1 = dependent.1;
      }
    }
  }
}

pub fn new(task: Box<Task+Send>) -> TaskAndOutputs {
  let n_outputs = task.output_count();
  TaskAndOutputs{
    task:              task,
    output_positions:  vec![(ChannelPosition(0), TaskId(0)); n_outputs],
  }
}
