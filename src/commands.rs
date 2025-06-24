use tauri::Runtime;

use crate::error::Result;

#[cfg(desktop)]
use crate::desktop as platform;

#[tauri::command]
pub async fn bind<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    endpoint: String,
) -> Result<()> {
    platform::bind(window, id, endpoint)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn unbind<R: Runtime>(window: tauri::Window<R>, id: String) -> Result<()> {
    platform::unbind(window, id).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    endpoint: String,
) -> Result<()> {
    platform::connect(window, id, endpoint)
        .await
        .map_err(|e| e.into())
}
#[tauri::command]
pub async fn connect_with_bind<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    local_addr: String,   // 本地绑定地址（如：192.168.1.100:0）
    endpoint: String,      // 远端连接地址（如：example.com:1234）
) -> Result<()> {
    platform::connect_with_bind(window, id, local_addr, endpoint)
        .await
        .map_err(|e| e.into())
}

#[tauri::command]
pub async fn disconnect(id: String) -> Result<()> {
    platform::disconnect(id).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn send(id: String, message: Vec<u8>, addr: Option<String>) -> Result<()> {
    platform::send(id, message, addr)
        .await
        .map_err(|e| e.into())
}
