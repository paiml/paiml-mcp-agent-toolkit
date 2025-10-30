# .pmat Recording Format Specification

**Version**: 1.0
**Status**: Stable
**Sprint**: 75 (REPLAY-001)
**Date**: 2025-10-30

## Overview

The `.pmat` file format is a binary format for storing time-travel debugging recordings. It enables capture and replay of program execution, preserving snapshots of variables, stack frames, and instruction pointers at each step.

## Design Goals

1. **Compact**: Use binary serialization to minimize file size
2. **Fast**: Efficient encoding/decoding for large recordings
3. **Schema-less**: Allow format evolution without recompilation
4. **Validatable**: Magic header and version byte for file integrity
5. **Safe**: DoS protection against malicious files

## Format Structure

### Binary Layout

```
Offset  | Size      | Type          | Description
--------|-----------|---------------|----------------------------------
0-3     | 4 bytes   | [u8; 4]       | Magic header: b"PMAT"
4       | 1 byte    | u8            | Format version (current: 1)
5-?     | Variable  | MessagePack   | RecordingMetadata struct
?       | 4 bytes   | u32           | Snapshot count (little-endian)
?-EOF   | Variable  | MessagePack   | Array of Snapshot structs
```

### Magic Header

- **Value**: `0x50 0x4D 0x41 0x54` (ASCII: "PMAT")
- **Purpose**: File type identification and corruption detection
- **Validation**: First 4 bytes of every `.pmat` file must match exactly

### Format Version

- **Current Version**: `1`
- **Type**: Unsigned 8-bit integer
- **Purpose**: Forward compatibility detection
- **Behavior**:
  - Parsers MUST reject files with version > their supported version
  - Parsers MAY support reading older versions

### MessagePack Serialization

All structured data (metadata, snapshots) is serialized using [MessagePack](https://msgpack.org/):
- **Library**: `rmp-serde` (Rust), compatible with any MessagePack implementation
- **Benefits**:
  - 50-70% smaller than JSON
  - Schema-less (field-based serialization)
  - Wide language support (Python, JavaScript, Go, etc.)

## Data Structures

### RecordingMetadata

Captures information about the recorded program execution.

```rust
struct RecordingMetadata {
    timestamp: u64,               // Unix milliseconds since epoch
    program: String,              // Program name or path
    args: Vec<String>,            // Command-line arguments
    environment: HashMap<String, String>,  // Environment variables (subset)
}
```

**MessagePack Encoding**: Serialized as a map with 4 keys.

**Example**:
```json
{
  "timestamp": 1698765432000,
  "program": "/usr/bin/python3",
  "args": ["script.py", "--verbose"],
  "environment": {
    "PATH": "/usr/bin:/bin",
    "USER": "developer"
  }
}
```

### Snapshot

A single point-in-time capture of execution state.

```rust
struct Snapshot {
    frame_id: u64,                        // Unique identifier
    timestamp_relative_ms: u32,           // Milliseconds since recording start
    variables: HashMap<String, serde_json::Value>,  // Variable name -> JSON value
    stack_frames: Vec<StackFrame>,        // Stack trace (innermost first)
    instruction_pointer: u64,             // Current instruction address
    memory_snapshot: Option<Vec<u8>>,     // Optional heap state capture
}
```

**MessagePack Encoding**: Serialized as a map with 6 keys.

**Example**:
```json
{
  "frame_id": 42,
  "timestamp_relative_ms": 150,
  "variables": {
    "x": 10,
    "name": "Alice",
    "items": [1, 2, 3]
  },
  "stack_frames": [
    {
      "name": "process_data",
      "file": "main.py",
      "line": 45,
      "locals": {"count": 5}
    }
  ],
  "instruction_pointer": 4198400,
  "memory_snapshot": null
}
```

### StackFrame

Represents a single frame in the call stack.

```rust
struct StackFrame {
    name: String,                         // Function or method name
    file: Option<String>,                 // Source file path
    line: Option<u32>,                    // Line number
    locals: HashMap<String, serde_json::Value>,  // Local variables
}
```

**MessagePack Encoding**: Serialized as a map with 4 keys.

## Validation Rules

### File Integrity

1. **Magic Header Validation**:
   - First 4 bytes MUST be `0x50 0x4D 0x41 0x54` ("PMAT")
   - Reject files with incorrect header immediately

2. **Version Validation**:
   - Byte 5 MUST be `<= FORMAT_VERSION` (current: 1)
   - Future versions may introduce breaking changes

3. **Snapshot Count Validation**:
   - Declared count MUST match actual array length
   - Reject files with count > `MAX_SNAPSHOT_COUNT` (10,000,000)
   - **Rationale**: DoS protection against malicious files

4. **Truncation Detection**:
   - MessagePack deserialization errors indicate truncated files
   - Return error with diagnostic message

### Empty Recordings

Empty recordings (0 snapshots) are VALID:
- Useful for testing format support
- Represents programs that completed before first snapshot

**Example**:
```
[PMAT][01][metadata_msgpack][00 00 00 00]
```

## Serialization Algorithm

### Encoding (.pmat file creation)

```rust
fn to_bytes(recording: &Recording) -> Result<Vec<u8>> {
    let mut buffer = Vec::new();

    // 1. Write magic header
    buffer.write_all(b"PMAT")?;

    // 2. Write format version
    buffer.write_all(&[FORMAT_VERSION])?;

    // 3. Serialize metadata with MessagePack
    let metadata_bytes = rmp_serde::to_vec(&recording.metadata)?;
    buffer.write_all(&metadata_bytes)?;

    // 4. Write snapshot count (u32 little-endian)
    let snapshot_count = recording.snapshots.len() as u32;
    buffer.write_all(&snapshot_count.to_le_bytes())?;

    // 5. Serialize snapshots array with MessagePack
    let snapshots_bytes = rmp_serde::to_vec(&recording.snapshots)?;
    buffer.write_all(&snapshots_bytes)?;

    Ok(buffer)
}
```

### Decoding (.pmat file loading)

```rust
fn from_bytes(bytes: &[u8]) -> Result<Recording> {
    let mut cursor = Cursor::new(bytes);

    // 1. Validate magic header
    let mut magic = [0u8; 4];
    cursor.read_exact(&mut magic)?;
    if &magic != b"PMAT" {
        return Err(anyhow!("Invalid magic header"));
    }

    // 2. Validate version
    let mut version_buf = [0u8; 1];
    cursor.read_exact(&mut version_buf)?;
    let version = version_buf[0];
    if version > FORMAT_VERSION {
        return Err(anyhow!("Unsupported version: {}", version));
    }

    // 3. Read remaining bytes
    let mut remaining = Vec::new();
    cursor.read_to_end(&mut remaining)?;

    // 4. Parse metadata
    let mut mp_cursor = Cursor::new(&remaining);
    let metadata: RecordingMetadata = rmp_serde::from_read(&mut mp_cursor)?;

    // 5. Read snapshot count
    let metadata_end = mp_cursor.position() as usize;
    let after_metadata = &remaining[metadata_end..];

    if after_metadata.len() < 4 {
        return Err(anyhow!("File truncated: missing snapshot count"));
    }

    let snapshot_count = u32::from_le_bytes(after_metadata[0..4].try_into()?);

    // 6. Validate snapshot count (DoS protection)
    if snapshot_count > MAX_SNAPSHOT_COUNT {
        return Err(anyhow!("Unreasonable snapshot count: {}", snapshot_count));
    }

    // 7. Parse snapshots
    let snapshots: Vec<Snapshot> = rmp_serde::from_slice(&after_metadata[4..])?;

    // 8. Verify count matches array length
    if snapshots.len() != snapshot_count as usize {
        return Err(anyhow!("Snapshot count mismatch"));
    }

    Ok(Recording { metadata, snapshots })
}
```

## Error Handling

### Error Categories

| Error Type | Example | Handling |
|------------|---------|----------|
| **Invalid Magic Header** | First 4 bytes != "PMAT" | Return error immediately, do not parse further |
| **Unsupported Version** | Version byte > FORMAT_VERSION | Return error with version numbers |
| **Truncated File** | EOF before snapshot array | Return error indicating corruption |
| **Count Mismatch** | Declared count != actual array length | Return error, do not trust file |
| **DoS Attack** | Snapshot count > 10,000,000 | Return error, refuse to parse |
| **MessagePack Error** | Invalid MessagePack bytes | Propagate deserialization error |

### Error Messages

```rust
// Good error messages (user-facing):
"Invalid magic header: expected [80, 77, 65, 84], got [88, 77, 65, 84]"
"Unsupported format version: 99 (current: 1)"
"File truncated: missing snapshot count"
"Unreasonable snapshot count: 4294967295 (max: 10000000)"
"Snapshot count mismatch: declared 100, actual 50"
```

## File Extensions

- **Primary**: `.pmat`
- **Compressed** (future): `.pmat.zst` (zstd compression)

## Version History

### Version 1 (2025-10-30)

**Initial format specification**:
- Magic header: "PMAT"
- MessagePack serialization
- Metadata + snapshot array
- DoS protection (MAX_SNAPSHOT_COUNT)

**Limitations**:
- No compression (handled externally)
- No incremental writing (full file write)
- No index for random access

**Future Enhancements** (v2+):
- Built-in zstd compression
- Streaming format for incremental writes
- Index block for O(1) snapshot lookup
- Delta encoding for memory snapshots

## Implementation References

- **Rust Implementation**: `server/src/services/dap/recording.rs`
- **Test Suite**: `server/tests/recording_format_tests.rs`
- **Sprint Documentation**: `docs/sprints/SPRINT-75-KICKOFF.md`

## Example Usage

### Creating a Recording

```rust
use pmat::services::dap::recording::{Recording, Snapshot, StackFrame};

let mut recording = Recording::new(
    "my_program".to_string(),
    vec!["--verbose".to_string()]
);

let snapshot = Snapshot {
    frame_id: 1,
    timestamp_relative_ms: 0,
    variables: HashMap::new(),
    stack_frames: vec![
        StackFrame {
            name: "main".to_string(),
            file: Some("main.rs".to_string()),
            line: Some(10),
            locals: HashMap::new(),
        }
    ],
    instruction_pointer: 0x1000,
    memory_snapshot: None,
};

recording.add_snapshot(snapshot);
recording.write_to_file("execution.pmat")?;
```

### Loading a Recording

```rust
use pmat::services::dap::recording::Recording;

let recording = Recording::load_from_file("execution.pmat")?;

println!("Program: {}", recording.metadata().program);
println!("Snapshots: {}", recording.snapshot_count());

for snapshot in recording.snapshots() {
    println!("Frame {}: {} variables",
        snapshot.frame_id,
        snapshot.variables.len()
    );
}
```

## Polyglot Support

The `.pmat` format is designed for polyglot time-travel debugging:

### Supported Languages (Current)
- **Rust**: Native support via DAP server
- **Python**: Future Sprint 76 (DAP client integration)
- **JavaScript/TypeScript**: Future Sprint 77
- **Go**: Future Sprint 78

### MessagePack Libraries

| Language | Library | Status |
|----------|---------|--------|
| Rust | `rmp-serde` | ✅ Used in PMAT |
| Python | `msgpack` | ✅ Compatible |
| JavaScript | `@msgpack/msgpack` | ✅ Compatible |
| Go | `github.com/vmihailenco/msgpack` | ✅ Compatible |
| Java | `org.msgpack:msgpack-core` | ✅ Compatible |

## Security Considerations

1. **DoS Protection**: MAX_SNAPSHOT_COUNT prevents memory exhaustion
2. **File Size Limits**: Recommend max file size checks (e.g., 1GB)
3. **Path Traversal**: File paths in stack frames are informational only
4. **Code Execution**: No executable code in .pmat files (data only)

## Compliance

- **MessagePack Specification**: https://github.com/msgpack/msgpack/blob/master/spec.md
- **Serde Integration**: https://serde.rs/
- **Rust Implementation**: Follows Rust API guidelines

## References

- MessagePack Specification: https://msgpack.org/
- Debug Adapter Protocol: https://microsoft.github.io/debug-adapter-protocol/
- PMAT Roadmap: `ROADMAP.md` (Sprint 75)
