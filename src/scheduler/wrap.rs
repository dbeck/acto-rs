use super::super::{Task, TaskId, ChannelPosition, SenderChannelId};

pub struct TaskWrap {
  task:              Box<Task+Send>,
  id:                TaskId,
  exec_count:        u64,
  output_positions:  Vec<ChannelPosition>,
}

impl TaskWrap {
  pub fn execute(&mut self) -> Result<(), &'static str> {
    self.exec_count += 1;
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

  #[cfg(any(test,feature = "printstats"))]
  fn print_stats(&self) {
    println!(" @drop TaskWrap task_id:{:?} exec_count:{}",
     self.id,
     self.exec_count);
  }

  #[cfg(not(any(test,feature = "printstats")))]
  fn print_stats(&self) {}
}

pub fn new(task: Box<Task+Send>, id: TaskId) -> TaskWrap {
  let n_outputs = task.output_count();
  TaskWrap{
    task:              task,
    id:                id,
    exec_count:        0,
    output_positions:  vec![ChannelPosition(0); n_outputs],
  }
}

impl Drop for TaskWrap {
  fn drop(&mut self) {
    self.print_stats();
  }
}
