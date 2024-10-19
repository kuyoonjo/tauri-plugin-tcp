# tauri-plugin-tcp

This plugin only works with Tauri 2.x only.

## Install

```bash
cargo add tauri-plugin-tcp
```
```bash
npm i @kuyoonjo/tauri-plugin-tcp
```

## Usage

### rust
```rust

tauri::Builder::default()
    .plugin(tauri_plugin_tcp::init())
    ...
```

### javascript
```javascript
import { connect, disconnect, send } from "@kuyoonjo/tauri-plugin-tcp";
import { listen } from "@tauri-apps/api/event";

const id = 'unique-id';
await bind(id, '0.0.0.0:8080');
await send(id, '192.168.1.2:9090', 'hello');
await unbind(id);

await listen("plugin://tcp", (x) => console.log(x.payload));

```

### permissions

add `"tcp:default"` into `"permissions"` list of `src-tauri\capabilities\default.json`

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  ...
  "permissions": [
    "core:default",
    ...
    "tcp:default"
  ]
}
```

## Support

| MacOS | Linux | Windows |
| ----- | ----- | ------- |
| ✅    | ✅    | ✅      |
