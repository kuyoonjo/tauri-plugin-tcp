use tauri::Runtime;

use crate::error::Result;

#[cfg(desktop)]
use crate::desktop as platform;


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
pub async fn send(id: String, message: Vec<u8>) -> Result<()> {
    platform::send(id, message).await.map_err(|e| e.into())
}
