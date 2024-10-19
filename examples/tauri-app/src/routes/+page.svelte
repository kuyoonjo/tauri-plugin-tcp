<script>
  import * as tcp from "../../../../webview-dist";
  import { listen } from "@tauri-apps/api/event";
  import { onMount } from "svelte";

  let serverAddr = "0.0.0.0:8080";
  let clientAddr = "";
  let message = "hello";

  async function bind() {
    try {
      await tcp.bind("xxx", serverAddr);
    } catch (e) {
      console.log({ e });
    }
  }

  async function unbind() {
    try {
      await tcp.unbind("xxx");
    } catch (e) {
      console.log({ e });
    }
  }

  async function connect() {
    try {
      await tcp.connect("zzz", serverAddr);
    } catch (e) {
      console.log({ e });
    }
  }

  async function disconnect() {
    try {
      await tcp.disconnect("zzz");
    } catch (e) {
      console.log({ e });
    }
  }

  async function send() {
    try {
      await tcp.send("xxx", message, clientAddr);
    } catch (e) {
      console.log({ e });
    }
  }

  async function sendToServer() {
    try {
      await tcp.send("zzz", message);
    } catch (e) {
      console.log({ e });
    }
  }

  onMount(() => {
    tcp.listen((x) => {
      console.log(x.payload);
      if(x.payload.id === "xxx" && x.payload.event.connect) {
        clientAddr = x.payload.event.connect;
      }
    });
  });
</script>

<main class="container">
  <h3>Server</h3>
  <div class="row">
    <input placeholder="e.g. 0.0.0.0:8080" bind:value={serverAddr} />
    <button on:click={bind}> Bind </button>
    <button on:click={unbind}> Unbind </button>
  </div>
  <div class="row">
    <input disabled placeholder="client addr" bind:value={clientAddr} />
  </div>
  <div class="row">
    <input placeholder="e.g. hello" bind:value={message} />
    <button on:click={send}> Send </button>
  </div>
  <h3>Client</h3>
  <div class="row">
    <input placeholder="e.g. 0.0.0.0:8080" bind:value={serverAddr} />
    <button on:click={connect}> Connect </button>
    <button on:click={disconnect}> Disconnect </button>
    <div class="row">
      <input placeholder="e.g. hello" bind:value={message} />
      <button on:click={sendToServer}> Send </button>
    </div>
  </div>
</main>
