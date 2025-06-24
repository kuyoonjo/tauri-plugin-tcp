use tauri::{
    plugin::{Builder, TauriPlugin},
    Runtime,
};

mod commands;
pub mod error;
pub mod models;

#[cfg(desktop)]
pub mod desktop;

const PLUGIN_NAME: &str = "tcp";

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            commands::bind,
            commands::unbind,
            commands::connect,
            commands::connect_with_bind,
            commands::disconnect,
            commands::send
        ])
        .build()
}
