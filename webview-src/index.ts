import { invoke } from '@tauri-apps/api/core';
import { EventCallback, Options, listen as _listen } from '@tauri-apps/api/event';
import { Buffer } from 'buffer';

/**
 * 
 * @param id A unique ID
 * @param endpoint e.g. 0.0.0.0:8080
 */
export async function bind(id: string, endpoint: string) {
  await invoke('plugin:tcp|bind', {
    id, endpoint,
  });
}

/**
 * 
 * @param id A unique ID
 */
export async function unbind(id: string) {
  await invoke('plugin:tcp|unbind', {
    id
  });
}


/**
 * 
 * @param id A unique ID
 * @param endpoint e.g. 0.0.0.0:8080
 */
export async function connect(id: string, endpoint: string) {
  await invoke('plugin:tcp|connect', {
    id, endpoint,
  });
}

/**
 * 
 * @param id A unique ID
 */
export async function disconnect(id: string) {
  await invoke('plugin:tcp|connect', {
    id
  });
}

/**
 * 
 * @param id A unique ID
 * @param message A string or a uint8 array
 * @param addr Optional destination address. e.g. 0.0.0.0:8080
 */
export async function send(id: string, message: string | number[], addr?: string) {
  await invoke('plugin:tcp|send', {
    id,
    message: typeof message === 'string' ? Array.from(Buffer.from(message)) : message,
    addr,
  });
}

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

export function listen(handler: EventCallback<Payload>, options?: Options) {
  return _listen('plugin://tcp', handler, options);
}
