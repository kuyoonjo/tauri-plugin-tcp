use socket2::{Domain, Socket, Type};
use std::net::ToSocketAddrs;
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
                    SOCKETS.write().await.remove(&tcp_id);
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

pub async fn connect_with_bind<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    local_addr: String, // 本地绑定地址（如：192.168.1.100:0）
    endpoint: String,   // 远端连接地址（如：example.com:1234）
) -> io::Result<()> {
    let mut sockets = SOCKETS.write().await;

    if let Some(s) = sockets.get(&id) {
        s.task.abort();
        sockets.remove(&id);
        sleep(time::Duration::from_millis(100)).await;
    }

    let local_addr = local_addr
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid local_addr"))?;
    let remote_addr = endpoint
        .to_socket_addrs()?
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidInput, "invalid endpoint"))?;

    // 使用 socket2 手动创建 socket 并绑定本地地址
    let socket = Socket::new(Domain::for_address(remote_addr), Type::STREAM, None)?;
    socket.bind(&local_addr.into())?;

    // Windows 兼容性处理：connect 时用阻塞模式，之后再转非阻塞交给 tokio
    socket.set_nonblocking(false)?;
    socket.connect(&remote_addr.into())?;
    socket.set_nonblocking(true)?;

    let stream = TcpStream::from_std(socket.into())?;
    let (mut read_half, write_half) = stream.into_split();

    debug_println!(
        "{} tcp connected to {} from {}",
        &id,
        &endpoint,
        &local_addr
    );

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
                    SOCKETS.write().await.remove(&tcp_id);
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
