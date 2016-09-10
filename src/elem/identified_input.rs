use super::super::{ChannelId, SenderName};

pub trait IdentifiedInput {
  fn get_input_id(&self, ch_id: usize) -> Option<(ChannelId, SenderName)>;
}
