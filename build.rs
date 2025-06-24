const COMMANDS: &[&str] = &["bind", "unbind", "connect", "connect_with_bind", "disconnect", "send"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}