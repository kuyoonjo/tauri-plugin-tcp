use serde::{Deserialize, Serialize};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;

pub(crate) struct Tcp {
  pub task: JoinHandle<()>,
  pub write_half: Mutex<OwnedWriteHalf>,
  pub endpoint: String,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Payload {
  pub id: String,
  pub addr: String,
  pub data: Vec<u8>,
}