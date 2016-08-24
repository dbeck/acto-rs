
pub trait OutputCounter {
  fn get_tx_count(&self, ch_id: usize) -> usize;
}
