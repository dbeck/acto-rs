
use super::super::{SenderChannelId, ReceiverChannelId};

pub trait InputCounter {
  fn get_rx_count(&self, ch_id: ReceiverChannelId) -> usize;
}

pub trait OutputCounter {
  fn get_tx_count(&self, ch_id: SenderChannelId) -> usize;
}
