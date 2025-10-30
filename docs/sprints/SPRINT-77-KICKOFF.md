# Sprint 77 Kickoff: Timeline UI Playback with Real Recordings
## Interactive Replay & Visualization

**Sprint Goal**: Build Timeline UI components to load and interactively replay .pmat recording files

**Status**: 🚀 KICKOFF
**Start Date**: October 30, 2025
**Methodology**: EXTREME TDD (RED → GREEN → REFACTOR → COMMIT)

---

## Context

### Problem Statement
Sprint 76 delivered complete recording capture infrastructure (.pmat files), but users have no way to **visualize** or **interactively explore** these recordings. The Timeline UI exists (Sprint 72-73) but only works with in-memory snapshots.

**Current Gap**:
```bash
# User can create recordings
$ pmat debug serve --record-dir ./recordings
# Session recorded: ./recordings/session-1730296200.pmat

# User can replay (text output only)
$ pmat debug replay ./recordings/session-1730296200.pmat
📋 Recording Metadata: ...
📊 Snapshot at position 0: ...

# ❌ NO VISUAL TIMELINE UI
# ❌ NO INTERACTIVE NAVIGATION
# ❌ NO DIFF/COMPARISON TOOLS
```

**Desired State**:
```bash
# Visual timeline UI with interactive controls
$ pmat debug timeline ./recordings/session-1730296200.pmat

┌─ Timeline UI ─────────────────────────────────────────┐
│ ⏮ ⏪ ▶ ⏩ ⏭  [=====●==========] Frame 42/100     │
│                                                        │
│ Variables:                  Stack Trace:              │
│  x = 42                     main.rs:10  main()        │
│  y = "hello"                utils.rs:23 helper()      │
│  counter = 0                                          │
│                                                        │
│ 🔍 Jump to frame: [____]    💾 Export snapshot       │
└────────────────────────────────────────────────────────┘

# Compare two recordings (diff mode)
$ pmat debug compare \
    ./recordings/session-A.pmat \
    ./recordings/session-B.pmat
```

### Sprint Dependencies

**Completed Prerequisites**:
- ✅ **Sprint 72**: ExecutionRecorder + in-memory Timeline UI
- ✅ **Sprint 75**: Recording format (.pmat) + deserialization
- ✅ **Sprint 76**: Recording capture + CLI workflow

**Enables Future Work**:
- Sprint 78: Advanced features (compression, seeking, search)
- Sprint 79: Production deployment
- Sprint 80+: Collaborative debugging (shared recordings)

---

## Sprint Objectives

### Primary Goal
Build interactive Timeline UI that loads .pmat files and provides visual, step-through replay with forward/backward navigation and recording comparison.

### Success Criteria
1. Timeline UI can load .pmat files via `pmat debug timeline <file>`
2. Interactive controls: play/pause, step forward/back, jump to frame
3. Display variables, stack traces, timestamps at each frame
4. Comparison mode: side-by-side diff of two recordings
5. Export capabilities: save snapshot as JSON, copy variable values
6. Performance: <100ms to load typical recordings, <16ms frame navigation

---

## Technical Architecture

### Component Integration

```
┌──────────────────────────────────────────────────────┐
│ Timeline UI (Sprint 72-73)                            │
│ ├─ TUI widgets (ratatui)                             │
│ ├─ Navigation controls                               │
│ └─ Variable display                                   │
└──────────────────────────────────────────────────────┘
                    ▲
                    │ NEW: Load from .pmat
                    │
┌──────────────────────────────────────────────────────┐
│ Recording::load_from_file() (Sprint 75)              │
│ ├─ Deserialize .pmat                                 │
│ ├─ Parse metadata                                     │
│ └─ Load all snapshots into memory                    │
└──────────────────────────────────────────────────────┘
                    ▲
                    │
┌──────────────────────────────────────────────────────┐
│ .pmat File (Sprint 76)                               │
│ [PMAT][V1][Metadata][Count][Snapshots...]            │
└──────────────────────────────────────────────────────┘
```

### New Components

#### 1. TimelinePlayer (NEW)
```rust
/// Manages playback state and navigation
pub struct TimelinePlayer {
    recording: Recording,
    current_frame: usize,
    playback_speed: f64,
    is_playing: bool,
}

impl TimelinePlayer {
    pub fn new(recording: Recording) -> Self;
    pub fn next_frame(&mut self) -> Option<&Snapshot>;
    pub fn prev_frame(&mut self) -> Option<&Snapshot>;
    pub fn jump_to(&mut self, frame: usize) -> Result<&Snapshot>;
    pub fn play(&mut self);
    pub fn pause(&mut self);
}
```

#### 2. ComparisonView (NEW)
```rust
/// Side-by-side diff of two recordings
pub struct ComparisonView {
    recording_a: Recording,
    recording_b: Recording,
    sync_mode: SyncMode, // ByFrame, ByTimestamp, ByLocation
}

impl ComparisonView {
    pub fn diff_at_frame(&self, frame: usize) -> VariableDiff;
    pub fn divergence_points(&self) -> Vec<usize>;
}
```

#### 3. Enhanced Timeline CLI Handler (UPDATED)
```rust
// server/src/cli/handlers/debug_handlers.rs

pub async fn handle_debug_timeline(
    recording: PathBuf,
    compare_with: Option<PathBuf>,
    start_frame: Option<usize>,
) -> Result<()> {
    // Load recording(s)
    // Initialize Timeline UI with TimelinePlayer
    // Render TUI loop with ratatui
}
```

---

## Sprint Tickets

### TIMELINE-001: TimelinePlayer State Management

**Goal**: Create TimelinePlayer struct to manage recording playback state and navigation

**Requirements**:
1. Load Recording from .pmat file
2. Track current frame position (0..snapshot_count)
3. Navigate: next(), prev(), jump_to(frame)
4. Playback control: play(), pause(), set_speed()
5. Expose current snapshot for UI rendering

**RED Tests** (10 tests):
1. Load recording from valid .pmat file
2. Initialize at frame 0
3. next_frame() advances to frame 1
4. prev_frame() moves back to frame 0
5. jump_to(N) sets current frame to N
6. jump_to(out_of_bounds) returns error
7. play() starts auto-advance (with timer)
8. pause() stops auto-advance
9. set_speed() changes playback rate
10. current_snapshot() returns correct snapshot

**Files**:
- `server/src/services/dap/timeline_player.rs` (new)
- `server/tests/timeline_player_tests.rs` (new)

---

### TIMELINE-002: Timeline UI Integration

**Goal**: Integrate TimelinePlayer with existing Timeline UI (Sprint 72-73 ratatui widgets)

**Requirements**:
1. Timeline UI accepts TimelinePlayer as input
2. Render progress bar: current frame / total frames
3. Display current snapshot variables in variable panel
4. Display stack trace in stack panel
5. Keyboard controls: ← → (prev/next), Space (play/pause), J (jump)
6. Frame counter display: "Frame 42/100 | 1250ms | main.rs:10"

**RED Tests** (10 tests):
1. Timeline UI renders with TimelinePlayer
2. Progress bar shows correct position
3. Variables panel shows current snapshot variables
4. Stack panel shows current stack frames
5. Pressing → advances frame
6. Pressing ← goes back
7. Space toggles play/pause
8. J prompts for frame number and jumps
9. Frame counter updates on navigation
10. UI updates at playback speed during play mode

**Files**:
- `server/src/services/dap/timeline_ui.rs` (update existing)
- `server/src/cli/handlers/debug_handlers.rs` (update handle_debug_timeline)
- `server/tests/timeline_ui_playback_tests.rs` (new)

---

### TIMELINE-003: Recording Comparison Mode

**Goal**: Side-by-side comparison of two recordings with diff highlighting

**Requirements**:
1. Load two .pmat files simultaneously
2. Display recordings side-by-side in split view
3. Sync navigation: both recordings advance together
4. Highlight variable differences (red/green diff colors)
5. Show divergence points (frames where execution differs)
6. Export diff report as JSON or text

**RED Tests** (10 tests):
1. Load two recordings successfully
2. Render split view with both recordings
3. Navigation syncs both recordings (by frame number)
4. Variable diff highlights differences
5. Divergence detection finds first difference
6. Sync modes: ByFrame, ByTimestamp, ByLocation
7. Export diff report as JSON
8. Handle recordings of different lengths
9. Handle recordings with different variable sets
10. Performance: diff calculation <10ms per frame

**Files**:
- `server/src/services/dap/comparison_view.rs` (new)
- `server/src/cli/handlers/debug_handlers.rs` (update with --compare flag)
- `server/tests/comparison_view_tests.rs` (new)

---

### TIMELINE-004: CLI Integration & UX

**Goal**: Complete CLI workflow with user-friendly commands and help text

**Requirements**:
1. `pmat debug timeline <file>` - Launch Timeline UI
2. `pmat debug compare <file1> <file2>` - Comparison mode
3. `--start-frame N` - Start at specific frame
4. `--speed N` - Set playback speed (0.1x - 10x)
5. Help text with keyboard shortcuts guide
6. Error handling: file not found, invalid format, corrupted recording

**RED Tests** (10 tests):
1. `pmat debug timeline file.pmat` launches UI
2. `--start-frame 50` starts at frame 50
3. `--speed 2.0` sets 2x playback speed
4. Invalid file path shows error
5. Corrupted .pmat shows error message
6. Help text displays keyboard shortcuts
7. `pmat debug compare A.pmat B.pmat` launches comparison
8. Comparison mode validates both files exist
9. `--help` shows complete usage guide
10. Graceful exit on Ctrl+C (no panic, clean terminal)

**Files**:
- `server/src/cli/commands.rs` (update Debug enum)
- `server/src/cli/handlers/debug_handlers.rs` (enhance handlers)
- `server/tests/timeline_cli_integration_tests.rs` (new)

---

## Implementation Plan (EXTREME TDD)

### Phase 1: RED Tests (2 hours)
```bash
# Create all test files with RED tests
server/tests/timeline_player_tests.rs           # 10 tests
server/tests/timeline_ui_playback_tests.rs      # 10 tests
server/tests/comparison_view_tests.rs           # 10 tests
server/tests/timeline_cli_integration_tests.rs  # 10 tests

# Total: 40 RED tests
# All tests use assert!(false) initially to ensure RED phase
```

**Commit**: "test: TIMELINE-001/002/003/004 RED phase tests"

### Phase 2: GREEN Implementation (6-8 hours)

**TIMELINE-001 Implementation** (2 hours):
```rust
// server/src/services/dap/timeline_player.rs

pub struct TimelinePlayer {
    recording: Recording,
    current_frame: usize,
    total_frames: usize,
    playback_speed: f64,
    is_playing: bool,
    play_timer: Option<Instant>,
}

impl TimelinePlayer {
    pub fn new(recording: Recording) -> Self {
        let total_frames = recording.snapshot_count();
        Self {
            recording,
            current_frame: 0,
            total_frames,
            playback_speed: 1.0,
            is_playing: false,
            play_timer: None,
        }
    }

    pub fn next_frame(&mut self) -> Option<&Snapshot> {
        if self.current_frame < self.total_frames - 1 {
            self.current_frame += 1;
            Some(&self.recording.snapshots()[self.current_frame])
        } else {
            None
        }
    }

    pub fn prev_frame(&mut self) -> Option<&Snapshot> {
        if self.current_frame > 0 {
            self.current_frame -= 1;
            Some(&self.recording.snapshots()[self.current_frame])
        } else {
            None
        }
    }

    pub fn jump_to(&mut self, frame: usize) -> Result<&Snapshot> {
        if frame < self.total_frames {
            self.current_frame = frame;
            Ok(&self.recording.snapshots()[frame])
        } else {
            anyhow::bail!("Frame {} out of bounds (total: {})", frame, self.total_frames)
        }
    }

    pub fn current_snapshot(&self) -> &Snapshot {
        &self.recording.snapshots()[self.current_frame]
    }
}
```

**Commit**: "feat: TIMELINE-001 TimelinePlayer state management"

**TIMELINE-002 Implementation** (2-3 hours):
- Update `timeline_ui.rs` to accept TimelinePlayer
- Wire keyboard controls to TimelinePlayer methods
- Update display panels with current snapshot data

**Commit**: "feat: TIMELINE-002 Timeline UI playback integration"

**TIMELINE-003 Implementation** (2 hours):
- Implement ComparisonView with dual-recording state
- Variable diff algorithm
- Split-view rendering

**Commit**: "feat: TIMELINE-003 Recording comparison mode"

**TIMELINE-004 Implementation** (1 hour):
- Update CLI commands with timeline/compare subcommands
- Wire handlers to TimelinePlayer and ComparisonView
- Add help text and error handling

**Commit**: "feat: TIMELINE-004 CLI integration and UX"

### Phase 3: REFACTOR (1 hour)
- Extract shared UI rendering logic
- Optimize diff algorithm (only compare changed variables)
- Add comprehensive error messages
- Performance profiling

**Commit**: "refactor: TIMELINE optimize and clean up"

---

## Error Handling Strategy

### Failure Modes

| Error Scenario | Behavior | User Message |
|----------------|----------|--------------|
| File not found | Exit with error | "Recording not found: {path}" |
| Corrupted .pmat | Exit with error | "Invalid recording format (corrupted file?)" |
| Version mismatch | Exit with error | "Recording format v{X} not supported (expected v1)" |
| Out of memory | Graceful degradation | "Recording too large, use --sparse mode" |
| Terminal too small | Resize prompt | "Terminal too small (need 80x24 minimum)" |

### Performance Constraints

- **Load time**: <100ms for recordings up to 1000 snapshots
- **Frame navigation**: <16ms (60fps for smooth playback)
- **Diff calculation**: <10ms per frame
- **Memory usage**: <50MB for typical recordings

---

## Timeline

### Estimated Duration
- **Phase 1 (RED)**: 2 hours
- **Phase 2 (GREEN)**: 6-8 hours
- **Phase 3 (REFACTOR)**: 1 hour
- **Total**: 9-11 hours (1-2 sessions)

### Prioritization
If time-constrained:
1. **MUST HAVE**: TIMELINE-001 (player), TIMELINE-002 (UI integration)
2. **SHOULD HAVE**: TIMELINE-004 (CLI polish)
3. **NICE TO HAVE**: TIMELINE-003 (comparison mode)

Can defer TIMELINE-003 to Sprint 78 if needed.

---

## Success Metrics

### Functional Requirements
- ✅ Load .pmat file and display in Timeline UI
- ✅ Navigate forward/backward through frames
- ✅ Play/pause with adjustable speed
- ✅ Jump to specific frame
- ✅ Display variables and stack trace
- ✅ Compare two recordings side-by-side

### Quality Metrics
- ✅ 40/40 tests passing (100% coverage)
- ✅ Zero panics or crashes
- ✅ Graceful terminal cleanup on exit
- ✅ Responsive UI (< 16ms frame time)

---

## Next Steps

### After Sprint 77 Completion
**Sprint 78**: Advanced Recording Features
- Recording compression (zstd) - reduce file size by 70%
- Indexed seeking (O(1) jumps) - instant navigation to any frame
- Recording metadata search - find sessions by program, date, etc.

**Sprint 79**: Production Hardening
- Integration tests with real debugging workflows
- Performance benchmarking (1000+ snapshot recordings)
- Documentation and user guides

---

## References

### Related Sprints
- **Sprint 72**: ExecutionRecorder + in-memory Timeline UI
- **Sprint 73**: Timeline UI polish
- **Sprint 75**: .pmat format + Recording deserialization
- **Sprint 76**: Recording capture + CLI workflow

### Specifications
- `docs/specifications/pmat-recording-format.md` (Sprint 75)
- `server/examples/recording_capture_demo.rs` (Sprint 76)

---

**Ready to Begin**: All prerequisites complete. Sprint 77 is ready for RED phase!
