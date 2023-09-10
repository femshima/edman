import {
  EDMAN_UNIQUE_NAME,
  NativeMessageKinds,
  NativeResultKinds,
} from "./generated/ce-adapter";

interface WithId {
  id: string;
}

type NativeMessage = NativeMessageKinds & WithId;
type NativeResult = NativeResultKinds & WithId;

type With<T, U> = U extends { type: T } ? U : never;

type NativeTypes = NativeMessage["type"];
type INativeMessage<T extends NativeTypes> = With<T, NativeMessageKinds>;
type INativeResult<T extends NativeTypes> = With<T, NativeResultKinds>;

export class NativeMessaging {
  private port?: chrome.runtime.Port;
  private listeners = {
    onDisconnect: this.onDisconnect.bind(this),
    onMessage: this.onMessage.bind(this),
  };
  private callbacks: Map<string, (result: NativeResultKinds) => void> =
    new Map();
  private counter = 0;

  public constructor() {
    this.ensurePort();
  }

  private ensurePort() {
    if (!this.port) {
      this.port = chrome.runtime.connectNative(EDMAN_UNIQUE_NAME);
      this.port.onDisconnect.addListener(this.listeners.onDisconnect);
      this.port.onMessage.addListener(this.listeners.onMessage);
    }
    return this.port;
  }
  private onDisconnect() {
    this.port?.onDisconnect.removeListener(this.listeners.onDisconnect);
    this.port?.onMessage.removeListener(this.listeners.onMessage);
    this.port = undefined;
  }
  private onMessage(message: NativeResult) {
    const id = message.id;
    if (!id) return;

    const reply = {
      ...message,
      id: undefined,
    };
    delete reply.id;
    this.callbacks.get(id)?.(reply);

    this.callbacks.delete(id);
  }
  private send(message: NativeMessageKinds): Promise<NativeResultKinds> {
    return new Promise((resolve) => {
      const port = this.ensurePort();

      const id = `${Date.now()}-${this.counter++}`;
      this.callbacks.set(id, resolve);

      port.postMessage({
        ...message,
        id,
      });
    });
  }

  public async sendNativeMessage<T extends NativeTypes>(
    type: T,
    data: INativeMessage<T>["data"],
  ): Promise<INativeResult<T>> {
    const result = await this.send({ type, data } as INativeMessage<T>);

    if (result.type === type) {
      return result as INativeResult<T>;
    } else if (result.type === "err") {
      throw new Error(`Native process returned error: ${result.data}`);
    } else {
      throw new Error(
        `Return type mismatch: expected ${type} but got ${result.type}`,
      );
    }
  }
}

export const native = new NativeMessaging();
