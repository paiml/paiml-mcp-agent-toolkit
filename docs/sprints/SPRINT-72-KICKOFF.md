# Sprint 72: Time-Travel Debugging - KICKOFF

**Sprint**: 72
**Status**: Starting
**Date**: October 30, 2025
**Duration**: 1-2 weeks (estimated)
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR)
**Depends On**: Sprint 71 (100% Complete)

---

## Overview

Sprint 72 implements **Time-Travel Debugging**, the second phase of PMAT's Interactive Tracing & Debugging capabilities. This builds on Sprint 71's DAP foundation to enable recording execution, taking snapshots, and replaying program state forward/backward.

**Parent Specification**: `docs/specifications/components/infrastructure.md`
**Related**: Part 1, Feature 1.2 (Time-Travel Debugging)

---

## Sprint Goals

Implement time-travel debugging infrastructure that enables:
1. ✨ Execution recording with state capture
2. ✨ Snapshot management with delta-based storage
3. ✨ Replay engine with forward/backward navigation
4. ✨ Deterministic state reconstruction
5. ✨ Multi-language support (Rust, Python, TypeScript/JavaScript)

**Success Criteria**:
- Can record program execution to `.pmat` file
- Snapshots use delta-based storage (<10% overhead)
- Can replay execution with step forward/backward
- State reconstruction is deterministic
- All tests passing with EXTREME TDD methodology
- Replay latency <50ms per step

---

## Architecture Overview

### Core Data Structures

```rust
/// Represents a single point-in-time snapshot of execution state
pub struct ExecutionSnapshot {
    /// Unique timestamp (nanoseconds since epoch)
    pub timestamp: u64,
    /// Sequence number (0-indexed)
    pub sequence: usize,
    /// Variable values at this point
    pub variables: HashMap<String, serde_json::Value>,
    /// Call stack frames
    pub call_stack: Vec<StackFrame>,
    /// Source code location
    pub location: SourceLocation,
    /// Delta from previous snapshot (for compression)
    pub delta: Option<SnapshotDelta>,
}

/// Represents the difference between two snapshots
pub struct SnapshotDelta {
    /// Variables that changed
    pub changed_vars: HashMap<String, serde_json::Value>,
    /// Variables that were removed
    pub removed_vars: HashSet<String>,
    /// Stack depth change
    pub stack_delta: i32,
}

/// Manages execution recording
pub struct ExecutionRecorder {
    /// All snapshots in chronological order
    snapshots: Vec<ExecutionSnapshot>,
    /// Current recording state
    is_recording: bool,
    /// Integration with DAP server
    dap_server: Arc<Mutex<DapServer>>,
}

/// Manages time-travel replay
pub struct ReplayEngine {
    /// All snapshots available for replay
    snapshots: Vec<ExecutionSnapshot>,
    /// Current position in replay
    current_index: usize,
    /// Replay mode (forward, backward, paused)
    mode: ReplayMode,
}
```

---

## Tickets (EXTREME TDD)

**Note**: Sprint 71 used TRACE-004 for DAP-PMAT Integration (not in original spec). Sprint 72 tickets renumbered to avoid conflicts:
- ~~TRACE-004~~ → TRACE-005 (Execution Recording)
- ~~TRACE-005~~ → TRACE-006 (Snapshot Management)
- ~~TRACE-006~~ → TRACE-007 (Replay Engine)

### TRACE-005: Execution Recording Infrastructure

**Goal**: Implement execution recording that captures program state at each step

**Estimated Time**: 6-8 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/execution_recorder_tests.rs):

```rust
use pmat::services::dap::ExecutionRecorder;
use std::sync::{Arc, Mutex};

// RED Test 1: Create recorder
#[test]
fn test_create_execution_recorder() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let recorder = ExecutionRecorder::new(dap_server);

    assert!(!recorder.is_recording());
    assert_eq!(recorder.snapshot_count(), 0);
}

// RED Test 2: Start recording
#[test]
fn test_start_recording() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap_server);

    recorder.start_recording();
    assert!(recorder.is_recording());
}

// RED Test 3: Stop recording
#[test]
fn test_stop_recording() {
    let dap_server = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap_server);

    recorder.start_recording();
    recorder.stop_recording();

    assert!(!recorder.is_recording());
}

// RED Test 4: Capture snapshot
#[test]
fn test_capture_snapshot() {
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/sample.rs");
    dap_server.simulate_stop_at_line("tests/fixtures/sample.rs", 3);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().unwrap();

    assert_eq!(snapshot.sequence, 0);
    assert!(snapshot.timestamp > 0);
    assert_eq!(snapshot.location.line, 3);
}

// RED Test 5: Multiple snapshots
#[test]
fn test_multiple_snapshots() {
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/sample.rs");

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();

    // Capture 3 snapshots
    for i in 2..=4 {
        dap.lock().unwrap().simulate_stop_at_line("tests/fixtures/sample.rs", i);
        recorder.capture_snapshot().unwrap();
    }

    assert_eq!(recorder.snapshot_count(), 3);
}

// RED Test 6: Snapshot contains variables
#[test]
fn test_snapshot_captures_variables() {
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/sample.rs");
    dap_server.simulate_stop_at_line("tests/fixtures/sample.rs", 3);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().unwrap();

    assert!(snapshot.variables.len() > 0, "Snapshot should capture variables");
}

// RED Test 7: Snapshot contains call stack
#[test]
fn test_snapshot_captures_call_stack() {
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/complex.rs");
    dap_server.simulate_stop_at_line("tests/fixtures/complex.rs", 11);

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();
    let snapshot = recorder.capture_snapshot().unwrap();

    assert!(snapshot.call_stack.len() > 0, "Snapshot should capture call stack");
}

// RED Test 8: Save recording to file
#[test]
fn test_save_recording_to_file() {
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/sample.rs");

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap);

    recorder.start_recording();

    for i in 2..=4 {
        dap.lock().unwrap().simulate_stop_at_line("tests/fixtures/sample.rs", i);
        recorder.capture_snapshot().unwrap();
    }

    recorder.save_to_file("/tmp/test_recording.pmat").unwrap();

    assert!(std::path::Path::new("/tmp/test_recording.pmat").exists());
}

// RED Test 9: Load recording from file
#[test]
fn test_load_recording_from_file() {
    // First, create a recording
    let mut dap_server = DapServer::new();
    dap_server.launch("tests/fixtures/sample.rs");

    let dap = Arc::new(Mutex::new(dap_server));
    let mut recorder = ExecutionRecorder::new(dap.clone());

    recorder.start_recording();

    for i in 2..=4 {
        dap.lock().unwrap().simulate_stop_at_line("tests/fixtures/sample.rs", i);
        recorder.capture_snapshot().unwrap();
    }

    recorder.save_to_file("/tmp/test_load.pmat").unwrap();

    // Now load it
    let loaded_recorder = ExecutionRecorder::load_from_file("/tmp/test_load.pmat").unwrap();

    assert_eq!(loaded_recorder.snapshot_count(), 3);
}

// RED Test 10: Cannot capture when not recording
#[test]
fn test_cannot_capture_when_not_recording() {
    let dap = Arc::new(Mutex::new(DapServer::new()));
    let mut recorder = ExecutionRecorder::new(dap);

    let result = recorder.capture_snapshot();
    assert!(result.is_err(), "Should fail when not recording");
}
```

**Implementation Files**:
- `server/src/services/dap/execution_recorder.rs` (new)
- `server/src/services/dap/types.rs` (extend with ExecutionSnapshot, SnapshotDelta)
- `server/src/services/dap/mod.rs` (export ExecutionRecorder)

**Estimated Time**: 6-8 hours
- RED phase (tests): 2 hours
- GREEN phase (implementation): 3-4 hours
- REFACTOR phase: 1-2 hours

---

### TRACE-006: Snapshot Management and Delta Storage

**Goal**: Implement efficient delta-based snapshot storage to minimize memory overhead

**Estimated Time**: 4-6 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/snapshot_manager_tests.rs):

```rust
// RED Test 1: Compute delta between snapshots
#[test]
fn test_compute_snapshot_delta() {
    let snapshot1 = create_snapshot_with_vars(hashmap!{
        "x" => json!(10),
        "y" => json!(20),
    });

    let snapshot2 = create_snapshot_with_vars(hashmap!{
        "x" => json!(15),  // Changed
        "y" => json!(20),  // Unchanged
        "z" => json!(30),  // New
    });

    let delta = SnapshotDelta::compute(&snapshot1, &snapshot2);

    assert_eq!(delta.changed_vars.len(), 1);  // Only x changed
    assert!(delta.changed_vars.contains_key("x"));
    assert_eq!(delta.removed_vars.len(), 0);
}

// RED Test 2: Apply delta to reconstruct snapshot
#[test]
fn test_apply_delta() {
    let snapshot1 = create_snapshot_with_vars(hashmap!{
        "x" => json!(10),
        "y" => json!(20),
    });

    let delta = SnapshotDelta {
        changed_vars: hashmap!{ "x" => json!(15) },
        removed_vars: HashSet::new(),
        stack_delta: 0,
    };

    let snapshot2 = snapshot1.apply_delta(&delta);

    assert_eq!(snapshot2.variables.get("x").unwrap(), &json!(15));
    assert_eq!(snapshot2.variables.get("y").unwrap(), &json!(20));
}

// RED Test 3: Delta compression ratio
#[test]
fn test_delta_compression_efficiency() {
    let snapshots = create_100_similar_snapshots();

    // Full storage size
    let full_size = snapshots.iter()
        .map(|s| serde_json::to_vec(s).unwrap().len())
        .sum::<usize>();

    // Delta-compressed storage size
    let compressed_size = calculate_delta_compressed_size(&snapshots);

    let compression_ratio = (full_size - compressed_size) as f64 / full_size as f64;

    assert!(compression_ratio > 0.8, "Should achieve >80% compression for similar snapshots");
}

// RED Test 4-10: Additional delta tests
```

**Implementation Files**:
- `server/src/services/dap/snapshot_delta.rs` (new)
- `server/src/services/dap/types.rs` (extend SnapshotDelta)

**Estimated Time**: 4-6 hours

---

### TRACE-007: Replay Engine with Forward/Backward Navigation

**Goal**: Implement replay engine that can navigate through recorded execution

**Estimated Time**: 6-8 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/replay_engine_tests.rs):

```rust
// RED Test 1: Create replay engine from recording
#[test]
fn test_create_replay_engine() {
    let recording = create_test_recording();
    let engine = ReplayEngine::from_recording(recording);

    assert_eq!(engine.current_position(), 0);
    assert_eq!(engine.total_snapshots(), 10);
}

// RED Test 2: Step forward
#[test]
fn test_step_forward() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    engine.step_forward();
    assert_eq!(engine.current_position(), 1);

    engine.step_forward();
    assert_eq!(engine.current_position(), 2);
}

// RED Test 3: Step backward
#[test]
fn test_step_backward() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    engine.goto(5);
    engine.step_backward();

    assert_eq!(engine.current_position(), 4);
}

// RED Test 4: Jump to specific position
#[test]
fn test_goto_position() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    engine.goto(7);
    assert_eq!(engine.current_position(), 7);
}

// RED Test 5: Get current snapshot
#[test]
fn test_get_current_snapshot() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    engine.goto(3);
    let snapshot = engine.current_snapshot();

    assert_eq!(snapshot.sequence, 3);
}

// RED Test 6: Cannot step backward from beginning
#[test]
fn test_cannot_step_backward_from_start() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    let result = engine.step_backward();
    assert!(result.is_err());
}

// RED Test 7: Cannot step forward from end
#[test]
fn test_cannot_step_forward_from_end() {
    let recording = create_test_recording();
    let mut engine = ReplayEngine::from_recording(recording);

    engine.goto(9);  // Last position
    let result = engine.step_forward();
    assert!(result.is_err());
}

// RED Test 8: Replay latency <50ms
#[test]
fn test_replay_performance() {
    let recording = create_large_recording(1000); // 1000 snapshots
    let mut engine = ReplayEngine::from_recording(recording);

    let start = std::time::Instant::now();
    engine.step_forward();
    let elapsed = start.elapsed();

    assert!(elapsed.as_millis() < 50, "Step forward should be <50ms");
}

// RED Test 9-10: Additional replay tests
```

**Implementation Files**:
- `server/src/services/dap/replay_engine.rs` (new)
- `server/src/services/dap/mod.rs` (export ReplayEngine)

**Estimated Time**: 6-8 hours

---

## Sprint 72 Progress Tracking

**Overall Goal**: 100% completion (3/3 tickets)

- ⏸️ **TRACE-005:** Execution Recording Infrastructure - Not started
- ⏸️ **TRACE-006:** Snapshot Management and Delta Storage - Not started
- ⏸️ **TRACE-007:** Replay Engine with Forward/Backward Navigation - Not started

**Target**: 30 tests passing (10 per ticket)

---

## Files Structure

```
server/
├── src/services/dap/
│   ├── mod.rs                    # Module exports
│   ├── types.rs                  # DAP protocol types (extend with new types)
│   ├── server.rs                 # DapServer (from Sprint 71)
│   ├── breakpoint_manager.rs     # Breakpoints (from Sprint 71)
│   ├── variable_inspector.rs     # Variables (from Sprint 71)
│   ├── execution_recorder.rs     # NEW - TRACE-005
│   ├── snapshot_delta.rs         # NEW - TRACE-006
│   └── replay_engine.rs          # NEW - TRACE-007
│
├── tests/
│   ├── dap_server_tests.rs       # Sprint 71 ✅
│   ├── breakpoint_manager_tests.rs # Sprint 71 ✅
│   ├── variable_inspector_tests.rs # Sprint 71 ✅
│   ├── dap_integration_tests.rs  # Sprint 71 ✅
│   ├── execution_recorder_tests.rs  # NEW - TRACE-005
│   ├── snapshot_manager_tests.rs    # NEW - TRACE-006
│   └── replay_engine_tests.rs       # NEW - TRACE-007
│
└── tests/fixtures/                # From Sprint 71
    ├── sample.rs
    ├── sample.py
    └── complex.rs
```

---

## Dependencies

**Required from Sprint 71**:
- ✅ DapServer with state management
- ✅ VariableInspector for capturing variables
- ✅ BreakpointManager for execution control
- ✅ Test fixtures (sample.rs, sample.py, complex.rs)

**New Dependencies**:
- `serde_json` - For JSON serialization of snapshots
- `bincode` or `rmp-serde` - For efficient `.pmat` file format (optional)
- `lz4` or `zstd` - For snapshot compression (optional)

---

## Testing Commands

```bash
# Sprint 71 tests (should still pass)
cargo test --test dap_server_tests
cargo test --test breakpoint_manager_tests
cargo test --test variable_inspector_tests
cargo test --test dap_integration_tests

# Sprint 72 tests (new)
cargo test --test execution_recorder_tests
cargo test --test snapshot_manager_tests
cargo test --test replay_engine_tests

# Run all DAP tests
cargo test dap
```

---

## Success Metrics

**Functional Requirements**:
- [ ] Can record execution to `.pmat` file
- [ ] Snapshots include variables, call stack, location
- [ ] Delta compression achieves >80% size reduction
- [ ] Can replay forward/backward through execution
- [ ] State reconstruction is deterministic
- [ ] Supports Rust, Python, TypeScript/JavaScript

**Performance Requirements**:
- [ ] Snapshot capture overhead <10%
- [ ] Delta computation <5ms per snapshot
- [ ] Replay step latency <50ms
- [ ] File load time <1s for 1000 snapshots

**Quality Requirements**:
- [ ] 30+ tests passing (10 per ticket)
- [ ] EXTREME TDD methodology (RED → GREEN → REFACTOR)
- [ ] Zero clippy warnings
- [ ] All quality gates passing

---

## Next Steps After Sprint 72

**Sprint 73**: Interactive REPL (from spec)
- TRACE-008: REPL framework with mode system
- TRACE-009: Integration with TDG analyzer
- TRACE-010: Fix suggestion engine

**Sprint 74**: ML Bug Prediction (Phase 2)
- BUG-001: Feature extraction from codebase
- BUG-002: ML model training pipeline
- BUG-003: Prediction engine with confidence scores

---

## References

- **Parent Spec**: `docs/specifications/components/infrastructure.md`
- **Sprint 71 Docs**: `docs/sprints/SPRINT-71-*.md`
- **Related Projects**:
  - rr-debugger (C/C++ time-travel debugging)
  - Omniscient Debugger (execution recording)
  - ruchyruchy (DAP debugging, time-travel)

---

**Kickoff Complete**
Ready to begin TRACE-005 implementation using EXTREME TDD methodology.

**Estimated Total Time**: 16-22 hours
**Estimated Calendar Time**: 1-2 weeks (at 2-4 hours/day)
**Recommended Approach**: Implement tickets sequentially (TRACE-005 → TRACE-006 → TRACE-007)
