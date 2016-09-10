use super::super::{ChannelId, SenderName, ReceiverChannelId};

pub trait IdentifiedInput {
  fn get_input_id(&self, ch_id: ReceiverChannelId) -> Option<(ChannelId, SenderName)>;
}
