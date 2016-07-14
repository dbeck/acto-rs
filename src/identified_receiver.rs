extern crate lossyq;
use self::lossyq::spsc::Receiver;
use super::channel_id;
use super::common::Message;

pub struct IdentifiedReceiver<Input: Copy+Send> {
  pub id    : channel_id::Id,
  pub input : Receiver<Message<Input>>,
}
