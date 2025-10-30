# Sprint 75 Kickoff: Recording Format & Serialization
## Time-Travel Debugging Persistence Layer

**Sprint Goal**: Define and implement .pmat recording file format for time-travel debugging
**Status**: 🚀 STARTED
**Start Date**: October 30, 2025
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → COMMIT)
**Prerequisites**: Sprint 74 complete (CLI commands implemented)

---

## Sprint Context

Sprint 74 delivered the CLI interface for time-travel debugging:
- ✅ `pmat debug serve` - DAP server running
- ✅ `pmat debug replay` - Replay handler implemented

**Current Gap**: The replay handler can read .pmat files but doesn't know how to parse them. This sprint bridges that gap.

---

## Sprint Tickets

### REPLAY-001: Define .pmat File Format Specification ⏳

**Goal**: Establish binary format for recording execution snapshots

**RED Phase**:
- Write tests expecting specific file structure
- Test magic header validation
- Test version compatibility checks
- Test snapshot count parsing
- Test metadata extraction

**GREEN Phase**:
- Define format: Magic bytes + Version + Metadata + Snapshots
- Choose encoding: MessagePack for efficiency
- Define Snapshot structure (timestamp, variables, stack frames)

**REFACTOR Phase**:
- Document specification in `docs/specifications/pmat-recording-format.md`
- Add format validation helpers

**COMMIT Phase**:
- Commit with specification and tests

**Expected Files**:
- `server/tests/recording_format_tests.rs` (NEW)
- `server/src/services/dap/recording.rs` (NEW - types only)
- `docs/specifications/pmat-recording-format.md` (NEW)

---

### REPLAY-002: Implement Snapshot Serialization 📝

**Goal**: Write snapshots to .pmat files

**RED Phase**:
- Tests for `Recording::new()`
- Tests for `Recording::add_snapshot()`
- Tests for `Recording::write_to_file()`
- Tests for file size validation
- Tests for corrupted write detection

**GREEN Phase**:
- Implement `Recording` struct
- Implement MessagePack encoding
- Implement atomic file writes (temp + rename)
- Error handling for I/O failures

**REFACTOR Phase**:
- Optimize buffer allocation
- Add compression (optional zstd)
- Wire into DAP server for capture

**COMMIT Phase**:
- Commit with serialization working

**Expected Files**:
- `server/src/services/dap/recording.rs` (MODIFIED)
- `server/tests/recording_serialization_tests.rs` (NEW)

---

### REPLAY-003: Implement Snapshot Deserialization 📖

**Goal**: Read snapshots from .pmat files for replay

**RED Phase**:
- Tests for `Recording::load_from_file()`
- Tests for snapshot iteration
- Tests for malformed file handling
- Tests for version mismatch errors
- Tests for partial read handling

**GREEN Phase**:
- Implement `Recording::load_from_file()`
- Implement MessagePack decoding
- Implement streaming read for large files
- Error handling with context messages

**REFACTOR Phase**:
- Wire into `handle_debug_replay()` in `server/src/cli/handlers/debug_handlers.rs`
- Replace placeholder output with actual snapshot loading
- Integration testing with real .pmat files

**COMMIT Phase**:
- Commit with end-to-end replay working

**Expected Files**:
- `server/src/services/dap/recording.rs` (MODIFIED)
- `server/tests/recording_deserialization_tests.rs` (NEW)
- `server/src/cli/handlers/debug_handlers.rs` (MODIFIED - wire deserialization)

---

## File Format Design (Preliminary)

```
.pmat File Structure (MessagePack binary):

[Magic Header: 4 bytes]  "PMAT"
[Format Version: u8]      1
[Metadata Block]
  - timestamp: u64 (Unix epoch milliseconds)
  - program: String
  - args: Vec<String>
  - environment: HashMap<String, String>
[Snapshot Count: u32]
[Snapshot Array]
  - Snapshot {
      frame_id: u64,
      timestamp_relative_ms: u32,
      variables: HashMap<String, Value>,
      stack_frames: Vec<StackFrame>,
      instruction_pointer: u64,
      memory_snapshot: Option<Vec<u8>>
    }
```

**Rationale for MessagePack**:
- ✅ Binary format (compact)
- ✅ Schema-less (flexible evolution)
- ✅ Fast encoding/decoding
- ✅ Wide language support (future polyglot replay)
- ✅ Better than JSON (50-70% size reduction)
- ✅ Better than Protobuf for our use case (no schema compilation)

**Alternative Considered**: Cap'n Proto (rejected - too complex for MVP)

---

## Integration Points

### Sprint 71 Integration (DAP Server)
- **File**: `server/src/services/dap/server.rs`
- **Integration**: When DAP server processes debug events, call `Recording::add_snapshot()`
- **Status**: Deferred to future sprint (capture phase)

### Sprint 72 Integration (Timeline UI)
- **File**: `server/src/services/dap/timeline_ui.rs`
- **Integration**: Pass loaded snapshots to `TimelineUI::new(snapshots)`
- **Status**: Prepared in REPLAY-003

### Sprint 74 Integration (Replay Handler)
- **File**: `server/src/cli/handlers/debug_handlers.rs:71`
- **Current**: `let _recording_data = std::fs::read(&recording)?;`
- **After REPLAY-003**: `let recording = Recording::load_from_file(&recording)?;`

---

## Dependencies

**Crates to Add** (`server/Cargo.toml`):
```toml
rmp-serde = "1.1"  # MessagePack for Rust
zstd = "0.13"      # Optional compression
```

**Existing Dependencies**:
- `serde` ✅ (already in Cargo.toml)
- `anyhow` ✅ (error handling)
- `tokio` ✅ (async I/O)

---

## Quality Gates

### EXTREME TDD Discipline
- ✅ RED: Write failing test first
- ✅ GREEN: Minimal implementation to pass
- ✅ REFACTOR: Improve design, tests still pass
- ✅ COMMIT: Atomic commit with message

### Test Coverage
- Target: 100% coverage for recording.rs
- Minimum: 90% coverage (tolerate error paths)

### Performance Targets
- Write: <10ms for 100 snapshots
- Read: <50ms for 100 snapshots
- File size: <1MB for 1000 snapshots (uncompressed)

### Failure Modes Tested
- ❌ Corrupted magic header
- ❌ Version mismatch
- ❌ Truncated file
- ❌ Invalid MessagePack
- ❌ Out of memory (huge snapshot count)

---

## Risks & Mitigations

### Risk: Format Evolution
**Likelihood**: High (format will need updates)
**Impact**: High (breaking old recordings)
**Mitigation**:
- Version byte in header
- Forward compatibility: newer readers can read older formats
- Backward compatibility: graceful degradation with warnings

### Risk: Large File Sizes
**Likelihood**: Medium (memory-heavy programs)
**Impact**: Medium (slow I/O, storage issues)
**Mitigation**:
- Streaming read/write (don't load entire file)
- Optional zstd compression (REPLAY-002 enhancement)
- Snapshot count limits in UI

### Risk: MessagePack Schema Changes
**Likelihood**: Low (format is stable)
**Impact**: Medium (deserialization failures)
**Mitigation**:
- Use `#[serde(default)]` for new fields
- Validate schema in tests
- Comprehensive error messages

---

## Success Criteria

✅ Sprint complete when:
1. All 3 tickets (REPLAY-001, REPLAY-002, REPLAY-003) delivered
2. `pmat debug replay recording.pmat` loads and displays snapshots
3. File format documented in specification
4. All tests passing (RED → GREEN → REFACTOR verified)
5. No clippy warnings
6. No compilation warnings

---

## Estimated Effort

- **REPLAY-001**: 2-3 hours (specification + RED tests)
- **REPLAY-002**: 3-4 hours (serialization + GREEN phase)
- **REPLAY-003**: 3-4 hours (deserialization + integration)
- **Total**: 8-11 hours (1-2 sessions)

---

**Sprint 75: Recording Format & Serialization**
**Goal**: Enable persistent time-travel debugging recordings
**Methodology**: EXTREME TDD

🎯 Let's build the persistence layer for time-travel debugging!
