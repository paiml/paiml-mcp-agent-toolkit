# Sprint 76 Kickoff: Recording Capture Integration
## ExecutionRecorder → .pmat File Pipeline

**Sprint Goal**: Integrate RecordingWriter with ExecutionRecorder to capture live debug sessions as .pmat files

**Status**: ⏳ ACTIVE
**Start Date**: October 30, 2025
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → COMMIT)

---

## Context

### Problem Statement
Sprint 75 delivered the .pmat recording format and serialization infrastructure, but there's no way to actually **create** recordings during live debug sessions. ExecutionRecorder (Sprint 72) captures snapshots in memory but has no persistence layer.

**Current Gap**:
```rust
// Sprint 72 (TRACE-005): ExecutionRecorder exists
let mut recorder = ExecutionRecorder::new();
recorder.record_snapshot(snapshot)?; // In-memory only! 💥

// Sprint 75 (REPLAY-003): RecordingWriter exists
let mut writer = RecordingWriter::new(file, "program", args)?;
writer.write_snapshot(&snapshot)?; // But not connected! 💥
```

**Desired State**:
```rust
// Sprint 76: Integrated pipeline
let mut recorder = ExecutionRecorder::with_writer(file, "program", args)?;
recorder.record_snapshot(snapshot)?; // Writes to .pmat file! ✅
recorder.finalize()?; // Complete recording saved! ✅
```

### Sprint Dependencies

**Completed Prerequisites**:
- ✅ **Sprint 71**: DAP Server (DapServer, types)
- ✅ **Sprint 72**: ExecutionRecorder + snapshot capture
- ✅ **Sprint 75**: Recording format (.pmat) + RecordingWriter

**Enables Future Work**:
- Sprint 77+: Timeline UI playback with real recordings
- Sprint 77+: Recording analysis and comparison tools
- Sprint 77+: Debugger integration (capture sessions from `pmat debug serve`)

---

## Sprint Objectives

### Primary Goal
Enable ExecutionRecorder to write snapshots to .pmat files during live debug sessions, creating persistent recordings that can be replayed with `pmat debug replay`.

### Success Criteria
1. ExecutionRecorder can be initialized with a RecordingWriter
2. Snapshots are automatically written to .pmat files as they're recorded
3. Finalization creates valid .pmat files loadable by Recording::load_from_file()
4. Memory-efficient streaming (no need to buffer all snapshots)
5. Robust error handling (disk full, permission errors, etc.)

---

## Technical Architecture

### Component Integration

```
┌─────────────────────────────────────────────────────────────┐
│ ExecutionRecorder (Sprint 72)                                │
│ ├─ record_snapshot(snapshot)                                 │
│ │  ├─ Store in memory (existing)                             │
│ │  └─ Write to RecordingWriter (NEW!)                        │
│ ├─ with_writer(file, program, args) (NEW!)                   │
│ └─ finalize() (NEW!)                                          │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ RecordingWriter<W> (Sprint 75)                               │
│ ├─ write_snapshot(&snapshot)                                 │
│ ├─ add_environment(key, value)                               │
│ └─ finalize()                                                 │
└─────────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────┐
│ .pmat File (Sprint 75)                                       │
│ [PMAT][V1][Metadata][Count][Snapshots...]                    │
└─────────────────────────────────────────────────────────────┘
```

### Integration Modes

**Mode 1: Memory-Only** (Existing Sprint 72 behavior)
```rust
let mut recorder = ExecutionRecorder::new();
recorder.record_snapshot(snapshot)?;
// No persistence, in-memory snapshots only
```

**Mode 2: Streaming to File** (NEW - Sprint 76)
```rust
let file = File::create("session.pmat")?;
let mut recorder = ExecutionRecorder::with_writer(file, "my_program", vec!["arg1"])?;
recorder.record_snapshot(snapshot)?; // Written to disk immediately
recorder.finalize()?; // Finalize .pmat file
```

**Mode 3: Deferred Persistence** (FUTURE - Sprint 77+)
```rust
let mut recorder = ExecutionRecorder::new();
recorder.record_snapshot(snapshot)?;
// Later...
recorder.save_to_file("session.pmat")?; // Write all snapshots at once
```

---

## Tickets

### CAPTURE-001: ExecutionRecorder with RecordingWriter Integration

**Goal**: Extend ExecutionRecorder to optionally write snapshots to .pmat files via RecordingWriter

**Requirements**:
1. `ExecutionRecorder::with_writer(W, program, args)` constructor
2. Internal `Option<RecordingWriter<W>>` field
3. `record_snapshot()` writes to both memory AND writer (if present)
4. `finalize()` method to complete recording
5. Backward compatible with existing memory-only mode
6. Error propagation from RecordingWriter to caller

**RED Tests** (10 tests):
1. Create recorder with writer
2. Record snapshot writes to file
3. Finalize creates valid .pmat file
4. Multiple snapshots written sequentially
5. Empty recording (no snapshots) is valid
6. Error handling: disk full simulation
7. Error handling: writer finalization failure
8. Memory-only mode still works (no writer)
9. Metadata updates (environment variables)
10. Concurrent snapshot recording (thread safety)

**Files**:
- `server/src/services/dap/execution_recorder.rs` (modify existing)
- `server/tests/execution_recorder_integration_tests.rs` (new)

**Estimated Effort**: 3-4 hours

---

### CAPTURE-002: DAP Server Recording Capture

**Goal**: Integrate recording capture into DAP server sessions (`pmat debug serve`)

**Requirements**:
1. DapServer starts ExecutionRecorder with RecordingWriter
2. Each debug session creates a unique .pmat file (timestamped)
3. Snapshots captured on breakpoint hits, step commands
4. Recording finalized on session end (disconnect)
5. Recording metadata includes DAP client info
6. Optional: `--record-dir` CLI flag to specify output directory
7. Default: `~/.pmat/recordings/session-{timestamp}.pmat`

**RED Tests** (10 tests):
1. DAP server creates recording file on session start
2. Breakpoint hit records snapshot
3. Step command records snapshot
4. Session end finalizes recording
5. Multiple sequential sessions create separate files
6. Concurrent sessions use different files
7. Recording directory creation (if doesn't exist)
8. Recording file naming convention validation
9. Metadata includes client info (VSCode, etc.)
10. Graceful handling if recording fails (continue debugging)

**Files**:
- `server/src/services/dap/server.rs` (modify existing)
- `server/src/cli/handlers/debug_handlers.rs` (modify handle_debug_serve)
- `server/tests/dap_recording_capture_tests.rs` (new)

**Estimated Effort**: 4-5 hours

---

### CAPTURE-003: CLI Recording Workflow End-to-End

**Goal**: Complete end-to-end workflow from `pmat debug serve` → recording capture → `pmat debug replay`

**Requirements**:
1. User runs `pmat debug serve --record-dir ./recordings`
2. Debugger connects, sets breakpoints
3. Program execution captures snapshots
4. Session ends, recording finalized
5. User runs `pmat debug replay ./recordings/session-{timestamp}.pmat`
6. Recording loads and displays correctly
7. All metadata preserved (program, args, environment)
8. Documentation updated with workflow example

**RED Tests** (10 tests):
1. End-to-end: serve → capture → replay
2. Verify recording metadata matches session
3. Verify all snapshots present in recording
4. Verify variable values match execution
5. Verify stack frames match execution
6. Recording file size reasonable (<10MB for 1000 snapshots)
7. Replay displays correct snapshot count
8. Replay position navigation works
9. Multiple sessions create separate recordings
10. Performance: <1ms overhead per snapshot

**Files**:
- `server/tests/recording_workflow_e2e_tests.rs` (new)
- `docs/specifications/components/infrastructure.md` (new)
- `server/examples/recording_capture_demo.rs` (new)

**Estimated Effort**: 3-4 hours

---

## Implementation Phases

### Phase 1: RED Tests (2-3 hours)
1. Create `server/tests/execution_recorder_integration_tests.rs` (10 tests)
2. Create `server/tests/dap_recording_capture_tests.rs` (10 tests)
3. Create `server/tests/recording_workflow_e2e_tests.rs` (10 tests)
4. All tests initially fail (RED phase)
5. Commit: "test: CAPTURE-001/002/003 RED phase tests"

### Phase 2: GREEN Implementation (4-5 hours)

**CAPTURE-001 Implementation**:
```rust
// server/src/services/dap/execution_recorder.rs

pub struct ExecutionRecorder<W: Write> {
    snapshots: Vec<Snapshot>, // Existing field
    writer: Option<RecordingWriter<W>>, // NEW!
}

impl<W: Write> ExecutionRecorder<W> {
    // NEW constructor
    pub fn with_writer(
        writer: W,
        program: String,
        args: Vec<String>
    ) -> Result<Self> {
        let recording_writer = RecordingWriter::new(writer, program, args)?;
        Ok(Self {
            snapshots: vec![],
            writer: Some(recording_writer),
        })
    }

    // MODIFIED: Write to both memory and file
    pub fn record_snapshot(&mut self, snapshot: Snapshot) -> Result<()> {
        // Write to file if writer present
        if let Some(ref mut writer) = self.writer {
            writer.write_snapshot(&snapshot)?;
        }

        // Store in memory (existing behavior)
        self.snapshots.push(snapshot);
        Ok(())
    }

    // NEW: Finalize recording
    pub fn finalize(self) -> Result<()> {
        if let Some(writer) = self.writer {
            writer.finalize()?;
        }
        Ok(())
    }
}
```

**CAPTURE-002 Implementation**:
```rust
// server/src/services/dap/server.rs

impl DapServer {
    pub async fn run(&self, port: u16, host: String) -> Result<()> {
        // ... existing server setup ...

        // NEW: Create recording file
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)?
            .as_secs();
        let recording_path = format!("./recordings/session-{}.pmat", timestamp);
        let recording_file = File::create(&recording_path)?;

        // NEW: Initialize recorder with writer
        let program = env::current_exe()?.display().to_string();
        let args = env::args().collect::<Vec<_>>();
        let mut recorder = ExecutionRecorder::with_writer(
            recording_file,
            program,
            args
        )?;

        // On breakpoint hit:
        let snapshot = self.capture_snapshot()?;
        recorder.record_snapshot(snapshot)?;

        // On session end:
        recorder.finalize()?;
        println!("Recording saved: {}", recording_path);

        Ok(())
    }
}
```

### Phase 3: REFACTOR (1-2 hours)
1. Extract recording file naming to utility function
2. Add error context with anyhow::Context
3. Add logging for recording lifecycle events
4. Improve documentation
5. Add unit tests for helper functions

### Phase 4: COMMIT (30 minutes)
1. Commit CAPTURE-001: "feat: CAPTURE-001 ExecutionRecorder with RecordingWriter integration"
2. Commit CAPTURE-002: "feat: CAPTURE-002 DAP server recording capture"
3. Commit CAPTURE-003: "docs: CAPTURE-003 recording workflow documentation"

---

## Error Handling Strategy

### Failure Modes

| Error Scenario | Behavior | Rationale |
|----------------|----------|-----------|
| Disk full during write | Continue debug session, log error, discard writer | Don't break debugging |
| Permission denied | Fail fast on session start | Better than failing mid-session |
| Writer finalization fails | Log error, return Err to caller | Caller should know recording incomplete |
| Concurrent file access | Use unique filenames (timestamp + PID) | Prevent collisions |
| Recording too large | Optional size limit (e.g., 1GB) | Prevent disk exhaustion |

### Error Messages

```rust
// Good error messages with context
anyhow::Context::context(
    File::create(&path),
    format!("Failed to create recording file: {}", path.display())
)?;

anyhow::Context::context(
    recorder.finalize(),
    "Failed to finalize recording (disk full or permission denied)"
)?;
```

---

## Testing Strategy

### Unit Tests (Ticket-specific)
- ExecutionRecorder with writer (10 tests)
- DAP server integration (10 tests)
- End-to-end workflow (10 tests)
- **Total**: 30 RED tests

### Integration Tests
- Actual file I/O (use tempfile crate)
- Real .pmat file validation with Recording::load_from_file()
- Memory usage testing (ensure streaming, not buffering)

### Property-Based Tests (Optional - Sprint 77)
- Recording fidelity: all snapshots preserved
- Idempotence: record → replay → record → replay (same result)
- Compression: file size reasonable

---

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Snapshot write overhead | <1ms | Benchmark with criterion |
| File write throughput | >1000 snapshots/sec | Integration test |
| Memory overhead | <100MB | Memory profiler |
| Finalization time | <100ms | Integration test |

---

## Documentation Deliverables

### Specifications
1. `docs/specifications/components/infrastructure.md` (300-400 lines)
   - Architecture diagram
   - API reference (ExecutionRecorder extensions)
   - CLI workflow examples
   - Error handling strategy

### Examples
1. `server/examples/recording_capture_demo.rs`
   - Demonstrates ExecutionRecorder::with_writer usage
   - Shows finalization and validation

### User-Facing
1. Update `pmat debug serve --help` with `--record-dir` flag
2. Add recording workflow to pmat-book (Chapter 7: Time-Travel Debugging)

---

## Success Metrics

### Functional
- ✅ All 30 RED tests passing
- ✅ End-to-end workflow succeeds: serve → capture → replay
- ✅ Recording files valid and loadable
- ✅ Metadata preserved correctly

### Non-Functional
- ✅ <1ms overhead per snapshot
- ✅ Memory-efficient streaming (no full buffering)
- ✅ Robust error handling (disk full, permissions)
- ✅ Backward compatible with memory-only mode

### Quality Gates
- ✅ All compilation clean
- ✅ All clippy warnings resolved
- ✅ All 30 tests passing (100%)
- ✅ Documentation complete

---

## Risk Mitigation

### Risk 1: Performance Overhead
**Mitigation**: Use asynchronous I/O (tokio::fs) if synchronous writes too slow

### Risk 2: Disk Space Exhaustion
**Mitigation**: Optional size limit (reject writes after 1GB), log warning

### Risk 3: Finalization Failure
**Mitigation**: Explicit finalize() call in DAP server shutdown handler

### Risk 4: Concurrent Access
**Mitigation**: Unique filenames (timestamp + PID), no shared state

---

## Sprint Capacity

### Estimated Effort
- **CAPTURE-001**: 3-4 hours (recorder integration)
- **CAPTURE-002**: 4-5 hours (DAP server integration)
- **CAPTURE-003**: 3-4 hours (E2E workflow + docs)
- **Total**: 10-13 hours (1-2 sessions)

### Prioritization
If time-constrained:
1. **MUST HAVE**: CAPTURE-001 (core integration)
2. **SHOULD HAVE**: CAPTURE-003 (E2E workflow)
3. **NICE TO HAVE**: CAPTURE-002 (DAP server integration)

Can defer CAPTURE-002 to Sprint 77 if needed.

---

## Next Steps

### After Sprint 76 Completion
**Sprint 77**: Timeline UI Playback with Real Recordings
- Enhance Timeline UI to load .pmat files
- Add interactive replay (step forward/backward)
- Recording comparison (diff two sessions)

**Sprint 78**: Advanced Recording Features
- Recording compression (zstd)
- Indexed seeking (O(1) position jumps)
- Recording metadata search (find by program name, timestamp)

---

## References

### Related Sprints
- **Sprint 71**: DAP Server infrastructure
- **Sprint 72**: ExecutionRecorder snapshot capture
- **Sprint 75**: .pmat format and RecordingWriter

### Specifications
- `docs/specifications/components/infrastructure.md` (Sprint 75)
- `docs/specifications/components/infrastructure.md` (Sprint 71)

### Related Files
- `server/src/services/dap/execution_recorder.rs` (Sprint 72)
- `server/src/services/dap/recording.rs` (Sprint 75)
- `server/src/services/dap/server.rs` (Sprint 71)

---

**Sprint 76: Recording Capture Integration**
**Goal**: Connect ExecutionRecorder → RecordingWriter → .pmat files
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → COMMIT)

🎯 Let's close the loop and create real recordings!
