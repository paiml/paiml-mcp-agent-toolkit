// Type definitions for Claude bridge
// Mirrors Rust types for cross-language compatibility

export enum ErrorCode {
  // Transport errors (1000-1999)
  PipeBrokenPipe = 1001,
  FramingError = 1002,
  MessageTooLarge = 1003,

  // Bridge errors (2000-2999)
  InitializationTimeout = 2001,
  WorkerCrashed = 2002,
  PoolExhausted = 2003,

  // Claude API errors (3000-3999)
  RateLimited = 3001,
  QuotaExceeded = 3002,
  InvalidApiKey = 3003,

  // Application errors (4000-4999)
  ComplexityExceeded = 4001,
  SatdDetected = 4002,
  QualityGateFailed = 4003,
}

export type SourceLang = 'rust' | 'typescript';

export interface BridgeError {
  readonly code: number;
  readonly message: string;
  readonly backtrace?: string;
  readonly sourceLang: SourceLang;
}

export type BridgeResult<T> =
  | { status: 'success'; payload: T }
  | { status: 'error'; payload: BridgeError }
  | { status: 'timeout'; payload: { elapsedMs: number } }
  | { status: 'circuit_open'; payload: { retryAfterMs: number } };

export interface FrameHeader {
  magic: string;
  sequence: bigint;
  length: number;
}

export interface BridgeConfig {
  maxMemoryMb?: number;
  timeoutMs?: number;
  sandboxed?: boolean;
}

export class BridgeException extends Error {
  constructor(
    public readonly code: ErrorCode,
    message: string,
    public readonly sourceLang: SourceLang = 'typescript'
  ) {
    super(message);
    this.name = 'BridgeException';
  }
}

export function unwrapBridgeResult<T>(result: BridgeResult<T>): T {
  switch (result.status) {
    case 'success':
      return result.payload;

    case 'error': {
      const error = new BridgeException(
        result.payload.code,
        result.payload.message,
        result.payload.sourceLang
      );
      if (result.payload.backtrace) {
        error.stack = result.payload.backtrace;
      }
      throw error;
    }

    case 'timeout':
      throw new BridgeException(
        ErrorCode.InitializationTimeout,
        `Operation timed out after ${result.payload.elapsedMs}ms`
      );

    case 'circuit_open':
      throw new BridgeException(
        ErrorCode.PoolExhausted,
        `Circuit open, retry after ${result.payload.retryAfterMs}ms`
      );

    default:
      const _exhaustive: never = result;
      throw new Error('Unhandled result status');
  }
}