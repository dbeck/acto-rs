
use super::super::{SenderChannelId};

pub trait OutputCounter {
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize;
}
