<script setup lang="ts">
import {ref} from "vue";
import * as tcp from "@kuyoonjo/tauri-plugin-tcp";
import {listen} from "@tauri-apps/api/event";

interface TcpMsg extends tcp.Payload {
}

const mode = ref("server")

const server_addr = ref("0.0.0.0:12345")
const local_addr = ref("")
const remote_addr = ref("")

const bind_local = ref(false)

const recv_msg = ref("")
const send_msg = ref("")

const started = ref(false)

async function start_tcp_server() {
  tcp.bind("tcp_server", server_addr.value);
  let unlisten = await listen("plugin://tcp", async (event) => {
        const payload = event.payload as TcpMsg;
        if (payload.id != "tcp_server") {
          unlisten()
          return
        }
        if (payload.event.connect) {
          console.log("connect: " + payload.event.connect)
          remote_addr.value = payload.event.connect
          started.value = true
        }
        if (payload.event.disconnect) {
          console.log("disconnect: " + payload.event.disconnect)
          remote_addr.value = ""
          started.value = false
        }
        if (payload.event.message) {
          console.log("recv: " + payload.event.message.data)
          const decoder = new TextDecoder();
          const msg = decoder.decode(new Uint8Array(payload.event.message.data));
          recv_msg.value += msg
        }
      }
  )
}

async function start_tcp_client() {
  if (bind_local.value) {
    tcp.connect_with_bind("tcp_client", local_addr.value, server_addr.value)
  } else {
    tcp.connect("tcp_client", server_addr.value)
  }
  let unlisten = await listen("plugin://tcp", async (event) => {
        const payload = event.payload as TcpMsg;
        if (payload.id != "tcp_client") {
          unlisten()
          return
        }
        if (payload.event.connect) {
          console.log("connect: " + payload.event.connect)
          remote_addr.value = payload.event.connect
          started.value = true
        }
        if (payload.event.disconnect) {
          console.log("disconnect: " + payload.event.disconnect)
          remote_addr.value = ""
          started.value = false
        }
        if (payload.event.message) {
          console.log("recv: " + payload.event.message.data)
          const decoder = new TextDecoder();
          const msg = decoder.decode(new Uint8Array(payload.event.message.data));
          recv_msg.value += msg
        }
      }
  )
}

async function send() {
  if (!started.value) {
    return
  }
  if (mode.value == "server") {
    console.log("send to server")
    tcp.send("tcp_server", send_msg.value, remote_addr.value);
  } else {
    console.log("send to client")
    tcp.send("tcp_client", send_msg.value)
  }
}

</script>

<template>
  <main class="container">
    <h1>Tauri plugin TCP demo</h1>

    <!-- choose TCP server mode or client mode -->
    <div class="row">
      <label>
        <input type="radio" name="mode" value="server" v-model="mode" checked/>
        TCP服务器
      </label>
      <label>
        <input type="radio" name="mode" value="client" v-model="mode"/>
        TCP客户端
      </label>
    </div>

    <div class="section" v-if="mode === 'server'">
      <h2>TCP服务器</h2>
      <div class="row">
        <input v-model="server_addr" placeholder="监听地址"/>
        <button @click="start_tcp_server">监听</button>
      </div>
      <label>
        连接的客户端 {{ remote_addr }}
      </label>
    </div>
    <div class="section" v-else>
      <h2>TCP客户端</h2>
      <div class="row">
        <input v-model="server_addr" type="text" placeholder="目标地址"/>
        <input v-model="bind_local" type="checkbox"/> 绑定本地地址
        <input v-if="bind_local" v-model="local_addr" type="text" placeholder="绑定地址"/>
        <button @click="start_tcp_client">连接</button>
      </div>
      <label>
        连接的客户端 {{ remote_addr }}
      </label>
    </div>

    <textarea class="recv-box" readonly v-model="recv_msg" placeholder="接收到的数据"></textarea>
    <input v-model="send_msg" type="text" placeholder="发送消息"/>
    <button @click="send">发送</button>
  </main>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  color: #0f0f0f;
  background-color: #f6f6f6;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

.container {
  margin: 0;
  padding-top: 5vh;
  display: flex;
  flex-direction: column;
  justify-content: center;
  text-align: center;
}

.section {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.row {
  display: flex;
  justify-content: center;
}

.recv-box {
  width: 95%;
  height: 200px;
  border: 1px solid #ccc;
  border-radius: 8px;
  padding: 10px;
  font-family: monospace;
  font-size: 14px;
  resize: none;
  background-color: #f9f9f9;
}

a {
  font-weight: 500;
  color: #646cff;
  text-decoration: inherit;
}

a:hover {
  color: #535bf2;
}

h1 {
  text-align: center;
}

input,
button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  color: #0f0f0f;
  background-color: #ffffff;
  transition: border-color 0.25s;
  box-shadow: 0 2px 2px rgba(0, 0, 0, 0.2);
}

button {
  cursor: pointer;
}

button:hover {
  border-color: #396cd8;
}

button:active {
  border-color: #396cd8;
  background-color: #e8e8e8;
}

input,
button {
  outline: none;
}

#greet-input {
  margin-right: 5px;
}

@media (prefers-color-scheme: dark) {
  :root {
    color: #f6f6f6;
    background-color: #2f2f2f;
  }

  a:hover {
    color: #24c8db;
  }

  input,
  button {
    color: #ffffff;
    background-color: #0f0f0f98;
  }

  button:active {
    background-color: #0f0f0f69;
  }
}
</style>