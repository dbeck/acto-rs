
use super::super::{ChannelPosition, ChannelPositionDiff};

pub struct Private {
}

impl Private {
  pub fn new() -> Private {
    Private {
    }
  }

  pub fn ensure_size(&mut self, _size: usize) {
  }

  #[allow(dead_code)]
  pub fn output_positions(&mut self,
                          _l1: usize,
                          _l2: usize,
                          _output_pos: &Vec<(ChannelPosition, ChannelPositionDiff)>)
  {
  }
}
