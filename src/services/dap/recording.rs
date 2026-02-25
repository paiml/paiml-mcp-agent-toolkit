#![cfg_attr(coverage_nightly, coverage(off))]
//! REPLAY-001: .pmat Recording Format Implementation
//! Sprint 75 - GREEN Phase
//!
//! Defines the binary format for time-travel debugging recordings.
//!
//! File Format:
//! ```text
//! [Magic Header: 4 bytes]  "PMAT"
//! [Format Version: u8]      1
//! [Metadata Block: MessagePack]  RecordingMetadata
//! [Snapshot Count: u32]     Number of snapshots (little-endian)
//! [Snapshot Array: MessagePack]  Vec<Snapshot>
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Cursor, Read, Write};
use std::path::Path;

/// Magic header for .pmat files (4 bytes: "PMAT")
pub const MAGIC_HEADER: &[u8; 4] = b"PMAT";

/// Current format version
pub const FORMAT_VERSION: u8 = 1;

/// Maximum reasonable snapshot count (DoS protection)
const MAX_SNAPSHOT_COUNT: u32 = 10_000_000; // 10 million snapshots

/// Recording metadata (serialized via MessagePack)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecordingMetadata {
    /// Unix timestamp (milliseconds since epoch)
    pub timestamp: u64,

    /// Program name or path
    pub program: String,

    /// Command-line arguments
    #[serde(default)]
    pub args: Vec<String>,

    /// Environment variables (subset)
    #[serde(default)]
    pub environment: HashMap<String, String>,
}

/// Single execution snapshot
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Snapshot {
    /// Frame ID (unique identifier for this snapshot)
    pub frame_id: u64,

    /// Relative timestamp in milliseconds (from recording start)
    pub timestamp_relative_ms: u32,

    /// Variable values at this point (name -> value as JSON)
    #[serde(default)]
    pub variables: HashMap<String, serde_json::Value>,

    /// Stack frames (innermost first)
    #[serde(default)]
    pub stack_frames: Vec<StackFrame>,

    /// Instruction pointer address
    pub instruction_pointer: u64,

    /// Optional memory snapshot (heap state)
    #[serde(default)]
    pub memory_snapshot: Option<Vec<u8>>,
}

/// Stack frame information
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StackFrame {
    /// Function or method name
    pub name: String,

    /// Source file path
    #[serde(default)]
    pub file: Option<String>,

    /// Line number
    #[serde(default)]
    pub line: Option<u32>,

    /// Local variables in this frame
    #[serde(default)]
    pub locals: HashMap<String, serde_json::Value>,
}

/// Complete recording (metadata + snapshots)
#[derive(Debug, Clone)]
pub struct Recording {
    /// Recording metadata
    metadata: RecordingMetadata,

    /// Execution snapshots
    snapshots: Vec<Snapshot>,
}

/// Compression level for recording serialization
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CompressionLevel {
    /// No compression (fastest)
    #[default]
    None,
    /// Fast compression (balanced)
    Fast,
    /// Best compression (slowest)
    Best,
}

/// Streaming recording writer for incremental .pmat file creation
///
/// Enables writing snapshots one at a time without loading all in memory.
/// Automatically tracks snapshot count and writes valid .pmat format on finalize.
pub struct RecordingWriter<W: Write> {
    /// Output writer
    writer: W,

    /// Recording metadata
    metadata: RecordingMetadata,

    /// Snapshot count (tracked incrementally)
    snapshot_count: u32,

    /// Snapshots buffer (MessagePack encoded, accumulated until finalize)
    snapshots_buffer: Vec<u8>,

    /// Whether finalize() has been called
    finalized: bool,

    /// Compression level
    compression: CompressionLevel,
}

/// Snapshot serializer with buffer reuse
///
/// Optimizes serialization by reusing internal buffers across multiple snapshots.
pub struct SnapshotSerializer {
    /// Internal buffer (reused for each serialization)
    buffer: Vec<u8>,

    /// Compression level
    compression: CompressionLevel,
}

// Core Recording implementation (constructor, accessors, serialization)
include!("recording_core.rs");

// Streaming writer and snapshot serializer implementations
include!("recording_streaming.rs");

// Tests
include!("recording_tests.rs");
