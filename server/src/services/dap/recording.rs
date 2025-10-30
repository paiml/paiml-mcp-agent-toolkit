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

impl Recording {
    /// Create a new recording
    pub fn new(program: String, args: Vec<String>) -> Self {
        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;

        Self {
            metadata: RecordingMetadata {
                timestamp,
                program,
                args,
                environment: HashMap::new(),
            },
            snapshots: Vec::new(),
        }
    }

    /// Add a snapshot to the recording
    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    /// Get metadata
    pub fn metadata(&self) -> &RecordingMetadata {
        &self.metadata
    }

    /// Get snapshots
    pub fn snapshots(&self) -> &[Snapshot] {
        &self.snapshots
    }

    /// Get snapshot count
    pub fn snapshot_count(&self) -> usize {
        self.snapshots.len()
    }

    /// Serialize recording to bytes
    ///
    /// Format:
    /// - Magic header (4 bytes: "PMAT")
    /// - Format version (1 byte)
    /// - Metadata (MessagePack)
    /// - Snapshot count (4 bytes, little-endian u32)
    /// - Snapshots array (MessagePack)
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let mut buffer = Vec::new();

        // Write magic header
        buffer.write_all(MAGIC_HEADER)?;

        // Write format version
        buffer.write_all(&[FORMAT_VERSION])?;

        // Serialize metadata with MessagePack
        let metadata_bytes = rmp_serde::to_vec(&self.metadata)
            .context("Failed to serialize metadata")?;
        buffer.write_all(&metadata_bytes)?;

        // Write snapshot count (u32 little-endian)
        let snapshot_count = self.snapshots.len() as u32;
        buffer.write_all(&snapshot_count.to_le_bytes())?;

        // Serialize snapshots with MessagePack
        let snapshots_bytes = rmp_serde::to_vec(&self.snapshots)
            .context("Failed to serialize snapshots")?;
        buffer.write_all(&snapshots_bytes)?;

        Ok(buffer)
    }

    /// Deserialize recording from bytes
    ///
    /// Validates magic header, version, and snapshot count before parsing.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let mut cursor = Cursor::new(bytes);

        // Validate magic header
        let mut magic = [0u8; 4];
        cursor.read_exact(&mut magic)
            .context("Failed to read magic header (file too short or corrupted)")?;

        if &magic != MAGIC_HEADER {
            anyhow::bail!(
                "Invalid magic header: expected {:?}, got {:?}",
                MAGIC_HEADER,
                magic
            );
        }

        // Read and validate version
        let mut version_buf = [0u8; 1];
        cursor.read_exact(&mut version_buf)
            .context("Failed to read format version")?;
        let version = version_buf[0];

        if version > FORMAT_VERSION {
            anyhow::bail!(
                "Unsupported format version: {} (current: {})",
                version,
                FORMAT_VERSION
            );
        }

        // Read remaining bytes for MessagePack parsing
        let mut remaining = Vec::new();
        cursor.read_to_end(&mut remaining)
            .context("Failed to read remaining data")?;

        // Parse metadata
        let mut mp_cursor = Cursor::new(&remaining);
        let metadata: RecordingMetadata = rmp_serde::from_read(&mut mp_cursor)
            .context("Failed to deserialize metadata")?;

        // Calculate position after metadata
        let metadata_end = mp_cursor.position() as usize;
        let after_metadata = &remaining[metadata_end..];

        // Read snapshot count
        if after_metadata.len() < 4 {
            anyhow::bail!("File truncated: missing snapshot count");
        }

        let snapshot_count_bytes: [u8; 4] = after_metadata[0..4]
            .try_into()
            .context("Failed to read snapshot count")?;
        let snapshot_count = u32::from_le_bytes(snapshot_count_bytes);

        // Validate snapshot count (DoS protection)
        if snapshot_count > MAX_SNAPSHOT_COUNT {
            anyhow::bail!(
                "Unreasonable snapshot count: {} (max: {})",
                snapshot_count,
                MAX_SNAPSHOT_COUNT
            );
        }

        // Parse snapshots
        let snapshots_bytes = &after_metadata[4..];
        let snapshots: Vec<Snapshot> = rmp_serde::from_slice(snapshots_bytes)
            .context("Failed to deserialize snapshots")?;

        // Verify snapshot count matches array length
        if snapshots.len() != snapshot_count as usize {
            anyhow::bail!(
                "Snapshot count mismatch: declared {}, actual {}",
                snapshot_count,
                snapshots.len()
            );
        }

        Ok(Self {
            metadata,
            snapshots,
        })
    }

    /// Write recording to file
    pub fn write_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let bytes = self.to_bytes()?;
        std::fs::write(path, bytes)?;
        Ok(())
    }

    /// Load recording from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let bytes = std::fs::read(&path)
            .with_context(|| format!("Failed to read recording file: {}", path.as_ref().display()))?;
        Self::from_bytes(&bytes)
    }
}

/// Validate magic header
pub fn validate_magic_header(bytes: &[u8]) -> bool {
    bytes == MAGIC_HEADER
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_magic_header_validation() {
        assert!(validate_magic_header(b"PMAT"));
        assert!(!validate_magic_header(b"XMAT"));
        assert!(!validate_magic_header(b"PM"));
    }

    #[test]
    fn test_empty_recording_roundtrip() {
        let recording = Recording::new("test_program".to_string(), vec![]);
        let bytes = recording.to_bytes().unwrap();
        let deserialized = Recording::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.metadata().program, "test_program");
        assert_eq!(deserialized.snapshots().len(), 0);
    }

    #[test]
    fn test_recording_with_snapshots_roundtrip() {
        let mut recording = Recording::new("test_program".to_string(), vec!["--test".to_string()]);

        let snapshot1 = Snapshot {
            frame_id: 1,
            timestamp_relative_ms: 0,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1000,
            memory_snapshot: None,
        };

        let snapshot2 = Snapshot {
            frame_id: 2,
            timestamp_relative_ms: 100,
            variables: HashMap::new(),
            stack_frames: vec![],
            instruction_pointer: 0x1008,
            memory_snapshot: None,
        };

        recording.add_snapshot(snapshot1.clone());
        recording.add_snapshot(snapshot2.clone());

        let bytes = recording.to_bytes().unwrap();
        let deserialized = Recording::from_bytes(&bytes).unwrap();

        assert_eq!(deserialized.snapshots().len(), 2);
        assert_eq!(deserialized.snapshots()[0].frame_id, 1);
        assert_eq!(deserialized.snapshots()[1].frame_id, 2);
    }

    #[test]
    fn test_invalid_magic_header() {
        let invalid_bytes = b"XMAT\x01...";
        let result = Recording::from_bytes(invalid_bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("magic header"));
    }

    #[test]
    fn test_unsupported_version() {
        // Create a file with future version (99)
        let mut bytes = Vec::new();
        bytes.extend_from_slice(b"PMAT");
        bytes.push(99); // Future version
        bytes.extend_from_slice(b"metadata...");

        let result = Recording::from_bytes(&bytes);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("version"));
    }
}
