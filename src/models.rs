use std::collections::HashMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tokio::net::tcp::OwnedWriteHalf;
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;

pub(crate) struct Tcp {
  pub task: JoinHandle<()>,
  pub kind: TcpKind,
}
pub(crate) enum TcpKind {
  Client {
    write_half: Mutex<OwnedWriteHalf>,
    endpoint: String,
  },
  Server {
    socks: Arc<RwLock<HashMap<String, Mutex<(OwnedWriteHalf, JoinHandle<()>)>>>>,
  },
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) struct Payload {
  pub id: String,
  pub event: PayloadEvent,
}

#[derive(Serialize, Deserialize, Clone)]
pub(crate) enum PayloadEvent {
  #[serde(rename = "connect")]
  Connect(String),
  #[serde(rename = "disconnect")]
  Disconnect(String),
  #[serde(rename = "message")]
  Message {
    addr: String,
    data: Vec<u8>,
  }
}