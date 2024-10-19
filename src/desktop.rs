use std::collections::HashMap;

use debug_print::debug_println;
use lazy_static::lazy_static;
use tauri::{Emitter, Manager, Runtime};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{Mutex, RwLock},
    time::{self, sleep},
};

use crate::models::*;

lazy_static! {
    static ref SOCKETS: RwLock<HashMap<String, Tcp>> = RwLock::new(HashMap::new());
}

pub async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    endpoint: String,
) -> io::Result<()> {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        s.task.abort();
        sockets.remove(&id);
        sleep(time::Duration::from_millis(100)).await;
    }

    let stream = TcpStream::connect(&endpoint).await?;
    let (mut read_half, write_half) = stream.into_split();
    debug_println!("{} tcp connected to {}", &id, &endpoint);
    let tcp_id = id.clone();
    let addr = endpoint.clone();
    let write_half = Mutex::new(write_half);
    let task = tokio::task::spawn(async move {
        let mut buf = [0; 65535];
        loop {
            if let Ok(len) = read_half.read(&mut buf).await {
                if len == 0 {
                    break;
                }
                debug_println!("{:?} bytes received from {:?}", len, addr);
                let _ = window.app_handle().emit_to(
                    window.label(),
                    "plugin://tcp",
                    Payload {
                        id: id.clone(),
                        addr: addr.to_string(),
                        data: buf[..len].to_vec(),
                    },
                );
            }
        }
        ()
    });

    sockets.insert(
        tcp_id,
        Tcp {
            task,
            write_half,
            endpoint,
        },
    );
    Ok(())
}

pub async fn disconnect(id: String) -> io::Result<()> {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        s.task.abort();
        sockets.remove(&id);
        debug_println!("{} tcp disconnected", &id);
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not disconnected.", &id),
        ))
    }
}

pub async fn send(id: String, message: Vec<u8>) -> io::Result<()> {
    let sockets = SOCKETS.read().await;

    if let Some(s) = sockets.get(&id) {
        s.write_half.lock().await.write_all(&message).await?;
        debug_println!(
            "{} tcp sent {} bytes to {}",
            &id,
            message.len(),
            &s.endpoint
        );
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not disconnected.", &id),
        ))
    }
}
