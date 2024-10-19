const COMMANDS: &[&str] = &["bind", "unbind", "send"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}