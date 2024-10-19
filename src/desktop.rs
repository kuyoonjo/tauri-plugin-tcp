use std::{collections::HashMap, sync::Arc};

use debug_print::debug_println;
use lazy_static::lazy_static;
use tauri::{Emitter, Manager, Runtime};
use tokio::{
    io::{self, AsyncReadExt, AsyncWriteExt},
    net::{tcp::OwnedWriteHalf, TcpListener, TcpStream},
    sync::{Mutex, RwLock},
    task::JoinHandle,
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
    let _ = window.app_handle().emit_to(
        window.label(),
        "plugin://tcp",
        Payload {
            id: id.clone(),
            event: PayloadEvent::Connect(endpoint.to_string()),
        },
    );
    let tcp_id = id.clone();
    let addr = endpoint.clone();
    let write_half = Mutex::new(write_half);
    let task = tokio::task::spawn(async move {
        let mut buf = [0; 65535];
        loop {
            if let Ok(len) = read_half.read(&mut buf).await {
                if len == 0 {
                    let _ = window.app_handle().emit_to(
                        window.label(),
                        "plugin://tcp",
                        Payload {
                            id: tcp_id.clone(),
                            event: PayloadEvent::Disconnect(addr.to_string()),
                        },
                    );
                    break;
                }
                debug_println!("{:?} bytes received from {:?}", len, addr);
                let _ = window.app_handle().emit_to(
                    window.label(),
                    "plugin://tcp",
                    Payload {
                        id: tcp_id.clone(),
                        event: PayloadEvent::Message {
                            addr: addr.to_string(),
                            data: buf[..len].to_vec(),
                        },
                    },
                );
            }
        }
        ()
    });

    sockets.insert(
        id,
        Tcp {
            task,
            kind: TcpKind::Client {
                write_half,
                endpoint,
            },
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

pub async fn bind<R: Runtime>(
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

    let listener = TcpListener::bind(&endpoint).await?;
    let _ = window.app_handle().emit_to(
        window.label(),
        "plugin://tcp",
        Payload {
            id: id.clone(),
            event: PayloadEvent::Bind(endpoint.to_string()),
        },
    );
    debug_println!("{} tcp server listening on {}", &id, &endpoint);
    let socks: Arc<RwLock<HashMap<String, Mutex<(OwnedWriteHalf, JoinHandle<()>)>>>> =
        Arc::new(RwLock::new(HashMap::new()));

    let tcp_id = id.clone();
    let socks_clone = socks.clone();
    let task = tokio::spawn(async move {
        loop {
            if let Ok((stream, addr)) = listener.accept().await {
                let (mut read_half, write_half) = stream.into_split();
                debug_println!("{} tcp client connected from {}", tcp_id, &addr);

                let window = window.clone();
                let id = tcp_id.clone();

                let _ = window.app_handle().emit_to(
                    window.label(),
                    "plugin://tcp",
                    Payload {
                        id: id.clone(),
                        event: PayloadEvent::Connect(addr.to_string()),
                    },
                );
                let task = tokio::task::spawn(async move {
                    let mut buf = [0; 65535];
                    loop {
                        if let Ok(len) = read_half.read(&mut buf).await {
                            if len == 0 {
                                let _ = window.app_handle().emit_to(
                                    window.label(),
                                    "plugin://tcp",
                                    Payload {
                                        id: id.clone(),
                                        event: PayloadEvent::Disconnect(addr.to_string()),
                                    },
                                );
                                break;
                            }
                            debug_println!("{:?} bytes received from {:?}", len, addr);
                            let _ = window.app_handle().emit_to(
                                window.label(),
                                "plugin://tcp",
                                Payload {
                                    id: id.clone(),
                                    event: PayloadEvent::Message {
                                        addr: addr.to_string(),
                                        data: buf[..len].to_vec(),
                                    },
                                },
                            );
                        }
                    }
                });
                socks_clone
                    .write()
                    .await
                    .insert(addr.to_string(), Mutex::new((write_half, task)));
            }
        }
    });

    sockets.insert(
        id,
        Tcp {
            task,
            kind: TcpKind::Server { socks },
        },
    );
    Ok(())
}

pub async fn unbind<R: Runtime>(window: tauri::Window<R>, id: String) -> io::Result<()> {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        if let TcpKind::Server { ref socks } = s.kind {
            for (_, wf) in socks.write().await.drain() {
                wf.lock().await.1.abort();
            }
            s.task.abort();
            sockets.remove(&id);
            debug_println!("{} tcp server closed.", &id);
            let _ = window.app_handle().emit_to(
                window.label(),
                "plugin://tcp",
                Payload {
                    id: id.clone(),
                    event: PayloadEvent::Unbind(),
                },
            );
        }
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not bond.", &id),
        ))
    }
}

pub async fn send(id: String, message: Vec<u8>, addr: Option<String>) -> io::Result<()> {
    let sockets = SOCKETS.read().await;

    if let Some(s) = sockets.get(&id) {
        match s.kind {
            TcpKind::Client {
                ref write_half,
                ref endpoint,
            } => {
                write_half.lock().await.write_all(&message).await?;
                debug_println!("{} tcp sent {} bytes to {}", &id, message.len(), endpoint);
            }
            TcpKind::Server { ref socks } => {
                if let Some(addr) = addr {
                    if let Some(wf) = socks.read().await.get(&addr) {
                        wf.lock().await.0.write_all(&message).await?;
                        debug_println!("{} tcp sent {} bytes to {}", &id, message.len(), &addr);
                    } else {
                        return Err(io::Error::new(
                            io::ErrorKind::NotFound,
                            format!("ID {} not socket {} not connected", &id, &addr),
                        ));
                    }
                } else {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        format!("ID {} is a tcp server. The `addr` is required.", &id),
                    ));
                }
            }
        }
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("ID {} not connected or not bond.", &id),
        ))
    }
}
