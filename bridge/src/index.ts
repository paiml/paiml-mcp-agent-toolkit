// Claude Bridge - Minimal GREEN phase implementation
// This is the simplest implementation to make tests pass

import { StdioTransport } from './transport';
import { BridgeConfig, BridgeResult, BridgeException, ErrorCode } from './types';

export class ClaudeBridge {
  private transport: StdioTransport;
  private initTime: number;

  constructor(config: BridgeConfig = {}) {
    const start = Date.now();

    // Simple initialization
    this.transport = new StdioTransport();
    this.initTime = Date.now() - start;

    // Fail fast if initialization exceeds SLA
    if (this.initTime > 500) {
      throw new BridgeException(
        ErrorCode.InitializationTimeout,
        `Initialization exceeded 500ms: ${this.initTime}ms`
      );
    }
  }

  /**
   * Analyze content (minimal implementation)
   */
  async analyzeContent(content: Buffer): Promise<any> {
    // Send request
    const request = {
      method: 'analyze',
      params: { content: content.toString() },
    };

    const requestJson = Buffer.from(JSON.stringify(request), 'utf-8');
    await this.transport.sendAtomic(requestJson);

    // Read response
    const responseBuffer = await this.transport.readFrame();
    const response = JSON.parse(responseBuffer.toString('utf-8'));

    return response;
  }

  /**
   * Get initialization time
   */
  getInitTime(): number {
    return this.initTime;
  }
}

// Export types
export * from './types';
export * from './transport';

// CLI entry point
if (require.main === module) {
  const args = process.argv.slice(2);
  const sandboxed = args.includes('--sandboxed');

  console.error(`Claude Bridge starting (sandboxed: ${sandboxed})`);

  const bridge = new ClaudeBridge({ sandboxed });
  console.error(`Bridge initialized in ${bridge.getInitTime()}ms`);
}