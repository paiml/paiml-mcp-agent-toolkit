// Simple 3-round ping-pong implementation per specification
export interface SimpleMessage {
  type: "ping" | "pong";
  round: number;
}

class Channel<T> {
  private queue: T[] = [];
  private resolvers: ((value: T) => void)[] = [];

  send(value: T): void {
    if (this.resolvers.length > 0) {
      const resolve = this.resolvers.shift();
      if (resolve) {
        resolve(value);
      }
    } else {
      this.queue.push(value);
    }
  }

  receive(timeout = 5): Promise<T | null> {
    if (this.queue.length > 0) {
      const item = this.queue.shift();
      return Promise.resolve(item ?? null);
    }

    return new Promise((resolve) => {
      const timer = setTimeout(() => {
        const index = this.resolvers.indexOf(resolve);
        if (index !== -1) {
          this.resolvers.splice(index, 1);
        }
        resolve(null);
      }, timeout);

      this.resolvers.push((value: T) => {
        clearTimeout(timer);
        resolve(value);
      });
    });
  }
}

export async function simplePingPong(): Promise<SimpleMessage[]> {
  const messages: SimpleMessage[] = [];
  const pingChan = new Channel<SimpleMessage>();
  const pongChan = new Channel<SimpleMessage>();

  // Ping actor
  const pingActor = async () => {
    for (let round = 1; round <= 3; round++) {
      // Send ping
      const pingMsg: SimpleMessage = { type: "ping", round };
      console.log(`Ping: Sending round ${round}`);
      pongChan.send(pingMsg);
      messages.push(pingMsg);

      // Wait for pong
      const pongMsg = await pingChan.receive();
      if (pongMsg) {
        console.log(`Ping: Received pong ${pongMsg.round}`);
        messages.push(pongMsg);
      }
    }
  };

  // Pong actor
  const pongActor = async () => {
    for (let round = 1; round <= 3; round++) {
      // Receive ping
      const pingMsg = await pongChan.receive();
      if (pingMsg) {
        console.log(`Pong: Received ping ${pingMsg.round}`);

        // Send pong back
        const pongMsg: SimpleMessage = { type: "pong", round: pingMsg.round };
        pingChan.send(pongMsg);
        console.log(`Pong: Sent pong ${pongMsg.round}`);
      }
    }
  };

  // Run both actors concurrently
  await Promise.all([pingActor(), pongActor()]);

  return messages;
}
