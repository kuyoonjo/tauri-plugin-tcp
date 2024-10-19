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
import { connect, disconnect, send, listen } from "@kuyoonjo/tauri-plugin-tcp";

// Server side
const sid = 'unique-server-id';
await bind(sid, '0.0.0.0:8080');
await send(sid, '192.168.1.2:9090', 'hello');
let clientAddr = '';
await listen((x) => {
  console.log(x.payload);
  if (x.payload.id === sid && x.payload.event.connect) {
    clientAddr = x.payload.event.connect;
    await send(sid, 'hello', clientAddr);
  }
});
await unbind(sid);

// Client side
const cid = 'unique-client-id';
await connect(cid, '0.0.0.0:8080');
await listen((x) => {
  console.log(x.payload);
  if (x.payload.id === cid && x.payload.event.message) {
    // npm i buffer
    // import { Buffer } from 'buffer';
    let str = Buffer.from(x.payload.event.message.data).toString();
    if (str === 'hello')
      await send(cid, 'world');
  }
});
await disconnect(cid);
```

#### Event Payload Interface
```typescript
export interface Payload {
  id: string;
  event: {
    bind?: string;
    unbind?: [];
    connect?: string;
    disconnect?: string;
    message?: {
      addr: string;
      data: number[];
    };
  };
}
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
