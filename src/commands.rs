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
    platform::bind(window, id, endpoint).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn unbind(id: String) -> Result<()> {
    platform::unbind(id).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn connect<R: Runtime>(
    window: tauri::Window<R>,
    id: String,
    endpoint: String,
) -> Result<()> {
    platform::connect(window, id, endpoint).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn disconnect(id: String) -> Result<()> {
    platform::disconnect(id).await.map_err(|e| e.into())
}

#[tauri::command]
pub async fn send(id: String, message: Vec<u8>, addr: Option<String>) -> Result<()> {
    platform::send(id, message, addr).await.map_err(|e| e.into())
}
