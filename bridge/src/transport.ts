// TypeScript stdio transport implementation
// Mirrors Rust implementation with length-prefixed framing

import { BridgeException, ErrorCode, FrameHeader } from './types';

export class StdioTransport {
  private static readonly PIPE_BUF = 4096;
  private static readonly FRAME_HEADER_SIZE = 16;
  private static readonly MAGIC = Buffer.from('PMAT', 'ascii');

  private sequenceNum = 0n;
  private writeBuffer: Buffer;
  private readBuffer: Buffer;

  constructor(
    private stdin = process.stdin,
    private stdout = process.stdout
  ) {
    this.writeBuffer = Buffer.alloc(StdioTransport.PIPE_BUF);
    this.readBuffer = Buffer.alloc(StdioTransport.PIPE_BUF);
  }

  /**
   * Send message with atomic write guarantee
   */
  async sendAtomic(payload: Buffer): Promise<void> {
    const maxPayload = StdioTransport.PIPE_BUF - StdioTransport.FRAME_HEADER_SIZE;

    if (payload.length > maxPayload) {
      throw new BridgeException(
        ErrorCode.MessageTooLarge,
        `Payload ${payload.length} bytes exceeds atomic limit ${maxPayload}`
      );
    }

    const seq = this.sequenceNum++;

    // Build frame: [magic(4)][seq(8)][len(4)][payload]
    let offset = 0;
    StdioTransport.MAGIC.copy(this.writeBuffer, offset);
    offset += 4;

    this.writeBuffer.writeBigUInt64LE(seq, offset);
    offset += 8;

    this.writeBuffer.writeUInt32LE(payload.length, offset);
    offset += 4;

    payload.copy(this.writeBuffer, offset);
    offset += payload.length;

    // Single atomic write
    return new Promise((resolve, reject) => {
      this.stdout.write(this.writeBuffer.subarray(0, offset), (err) => {
        if (err) {
          reject(
            new BridgeException(ErrorCode.PipeBrokenPipe, `Write failed: ${err.message}`)
          );
        } else {
          resolve();
        }
      });
    });
  }

  /**
   * Read frame with header validation
   */
  async readFrame(): Promise<Buffer> {
    // Read header
    const header = await this.readExact(StdioTransport.FRAME_HEADER_SIZE);

    // Verify magic
    const magic = header.subarray(0, 4);
    if (!magic.equals(StdioTransport.MAGIC)) {
      throw new BridgeException(
        ErrorCode.FramingError,
        'Invalid magic bytes in frame'
      );
    }

    // Extract sequence and length
    const seq = header.readBigUInt64LE(4);
    const len = header.readUInt32LE(12);

    if (len > StdioTransport.PIPE_BUF) {
      throw new BridgeException(
        ErrorCode.MessageTooLarge,
        `Frame length ${len} exceeds maximum ${StdioTransport.PIPE_BUF}`
      );
    }

    // Read payload
    return await this.readExact(len);
  }

  /**
   * Read exactly n bytes from stdin
   */
  private async readExact(n: number): Promise<Buffer> {
    const chunks: Buffer[] = [];
    let remaining = n;

    return new Promise((resolve, reject) => {
      const onData = (chunk: Buffer) => {
        chunks.push(chunk);
        remaining -= chunk.length;

        if (remaining === 0) {
          this.stdin.removeListener('data', onData);
          this.stdin.removeListener('end', onEnd);
          this.stdin.removeListener('error', onError);
          resolve(Buffer.concat(chunks, n));
        } else if (remaining < 0) {
          this.stdin.removeListener('data', onData);
          this.stdin.removeListener('end', onEnd);
          this.stdin.removeListener('error', onError);
          reject(
            new BridgeException(ErrorCode.FramingError, 'Read more bytes than expected')
          );
        }
      };

      const onEnd = () => {
        this.stdin.removeListener('data', onData);
        this.stdin.removeListener('error', onError);
        reject(new BridgeException(ErrorCode.PipeBrokenPipe, 'Unexpected EOF'));
      };

      const onError = (err: Error) => {
        this.stdin.removeListener('data', onData);
        this.stdin.removeListener('end', onEnd);
        reject(new BridgeException(ErrorCode.PipeBrokenPipe, `Read error: ${err.message}`));
      };

      this.stdin.on('data', onData);
      this.stdin.once('end', onEnd);
      this.stdin.once('error', onError);
    });
  }
}