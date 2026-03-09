# Sprint 73 Kickoff: Time-Travel UI & Visualization
## Building on Sprint 72's Time-Travel Debugging Infrastructure

**Sprint**: 73
**Status**: Ready to Begin
**Date**: October 30, 2025
**Prerequisites**: Sprint 72 complete (30/30 tests passing)
**Related Spec**: `docs/specifications/components/infrastructure.md`

---

## Executive Summary

Sprint 73 builds on Sprint 72's time-travel debugging infrastructure by adding **user-facing visualization and interaction** capabilities. While Sprint 72 gave us the engine (recording, compression, replay), Sprint 73 gives us the **driver's cockpit** (timeline, diff visualization, UI integration).

**Goal**: Transform time-travel debugging from a programmatic API into a visual, interactive developer experience.

---

## Sprint Goals

1. **TRACE-008**: Execution Timeline Visualization
   - Terminal-based timeline UI
   - Snapshot navigation with visual indicators
   - Performance metrics display

2. **TRACE-009**: Variable Diff Highlighting
   - Visual diff between snapshots
   - Changed/added/removed variable highlighting
   - Stack frame comparison

3. **TRACE-010**: VSCode Extension Integration
   - Debug adapter registration
   - Time-travel controls in IDE
   - Inline variable highlighting

---

## Context from Sprint 72

### What We Built (Sprint 72)
- ✅ ExecutionRecorder - Captures program state at each step
- ✅ SnapshotDelta - Compresses snapshots (79.3% efficiency)
- ✅ ReplayEngine - Navigate forward/backward (<1ms latency)

### What We're Building Now (Sprint 73)
- 🎯 Timeline UI - Visual representation of execution history
- 🎯 Diff Visualization - Show what changed between snapshots
- 🎯 VSCode Integration - Time-travel debugging in your editor

---

## Ticket Breakdown

### TRACE-008: Execution Timeline Visualization

**Goal**: Terminal-based UI for visualizing execution timeline and navigating snapshots

**Estimated Time**: 6-8 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/timeline_ui_tests.rs):

```rust
// RED Test 1: Render timeline from recording
#[test]
fn test_render_timeline() {
    let recording = create_test_recording(10); // 10 snapshots
    let ui = TimelineUI::new(recording);

    let output = ui.render();

    assert!(output.contains("0─────1─────2─────3─────4─────5─────6─────7─────8─────9"));
    assert!(output.contains("^")); // Position indicator
}

// RED Test 2: Navigate timeline with keyboard
#[test]
fn test_navigate_timeline() {
    let recording = create_test_recording(10);
    let mut ui = TimelineUI::new(recording);

    ui.handle_key('→'); // Step forward
    assert_eq!(ui.current_position(), 1);

    ui.handle_key('←'); // Step backward
    assert_eq!(ui.current_position(), 0);
}

// RED Test 3: Jump to specific snapshot
#[test]
fn test_jump_to_snapshot() {
    let recording = create_test_recording(10);
    let mut ui = TimelineUI::new(recording);

    ui.jump_to(5);
    assert_eq!(ui.current_position(), 5);

    let output = ui.render();
    assert!(output.contains("     ^")); // Position indicator at 5
}

// RED Test 4: Display snapshot details
#[test]
fn test_display_snapshot_details() {
    let recording = create_test_recording_with_vars();
    let mut ui = TimelineUI::new(recording);

    ui.jump_to(3);
    let details = ui.render_details();

    assert!(details.contains("Snapshot #3"));
    assert!(details.contains("Variables:"));
    assert!(details.contains("Call Stack:"));
    assert!(details.contains("Location:"));
}

// RED Test 5: Show performance metrics
#[test]
fn test_show_performance_metrics() {
    let recording = create_test_recording(100);
    let ui = TimelineUI::new(recording);

    let metrics = ui.render_metrics();

    assert!(metrics.contains("Total snapshots: 100"));
    assert!(metrics.contains("Recording size:"));
    assert!(metrics.contains("Compression ratio:"));
}

// RED Test 6-10: Additional timeline tests
// - Color-coded timeline
// - Responsive width handling
// - Error handling for empty recordings
// - Bookmark management
// - Search snapshots by variable value
```

**Implementation Files**:
- `server/src/services/dap/timeline_ui.rs` (new)
- `server/src/services/dap/mod.rs` (export TimelineUI)

---

### TRACE-009: Variable Diff Highlighting

**Goal**: Visual comparison of variables between snapshots

**Estimated Time**: 6-8 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/variable_diff_tests.rs):

```rust
// RED Test 1: Detect changed variables
#[test]
fn test_detect_changed_variables() {
    let snapshot1 = create_snapshot_with_vars(0, hashmap!{"x" => 10, "y" => 20});
    let snapshot2 = create_snapshot_with_vars(1, hashmap!{"x" => 15, "y" => 20});

    let diff = VariableDiff::compute(&snapshot1, &snapshot2);

    assert_eq!(diff.changed.len(), 1);
    assert!(diff.changed.contains_key("x"));
    assert_eq!(diff.changed["x"].old_value, json!(10));
    assert_eq!(diff.changed["x"].new_value, json!(15));
}

// RED Test 2: Detect new variables
#[test]
fn test_detect_new_variables() {
    let snapshot1 = create_snapshot_with_vars(0, hashmap!{"x" => 10});
    let snapshot2 = create_snapshot_with_vars(1, hashmap!{"x" => 10, "z" => 30});

    let diff = VariableDiff::compute(&snapshot1, &snapshot2);

    assert_eq!(diff.added.len(), 1);
    assert!(diff.added.contains("z"));
    assert_eq!(diff.added["z"], json!(30));
}

// RED Test 3: Detect removed variables
#[test]
fn test_detect_removed_variables() {
    let snapshot1 = create_snapshot_with_vars(0, hashmap!{"x" => 10, "y" => 20});
    let snapshot2 = create_snapshot_with_vars(1, hashmap!{"x" => 10});

    let diff = VariableDiff::compute(&snapshot1, &snapshot2);

    assert_eq!(diff.removed.len(), 1);
    assert!(diff.removed.contains("y"));
}

// RED Test 4: Render diff as colored text
#[test]
fn test_render_diff_colored() {
    let diff = create_sample_diff();

    let output = diff.render_colored();

    assert!(output.contains("\x1b[32m")); // Green for added
    assert!(output.contains("\x1b[31m")); // Red for removed
    assert!(output.contains("\x1b[33m")); // Yellow for changed
}

// RED Test 5: Side-by-side diff display
#[test]
fn test_side_by_side_diff() {
    let snapshot1 = create_snapshot_with_vars(0, hashmap!{"x" => 10, "y" => 20});
    let snapshot2 = create_snapshot_with_vars(1, hashmap!{"x" => 15, "z" => 30});

    let diff = VariableDiff::compute(&snapshot1, &snapshot2);
    let output = diff.render_side_by_side();

    assert!(output.contains("Snapshot #0"));
    assert!(output.contains("Snapshot #1"));
    assert!(output.contains("x: 10"));
    assert!(output.contains("x: 15"));
}

// RED Test 6-10: Additional diff tests
// - Deep object diff (nested structures)
// - Array diff visualization
// - Type change detection
// - Diff statistics summary
// - Export diff to JSON
```

**Implementation Files**:
- `server/src/services/dap/variable_diff.rs` (new)
- `server/src/services/dap/mod.rs` (export VariableDiff)

---

### TRACE-010: VSCode Extension Integration

**Goal**: Integrate time-travel debugging into VSCode

**Estimated Time**: 8-10 hours

**Phase**: RED (Write failing tests first)

**Test Requirements** (server/tests/vscode_extension_tests.rs):

```rust
// RED Test 1: Generate DAP configuration
#[test]
fn test_generate_dap_config() {
    let config = VSCodeExtension::generate_launch_json("src/main.rs");

    let json: serde_json::Value = serde_json::from_str(&config).unwrap();

    assert_eq!(json["type"], "pmat-debug");
    assert_eq!(json["request"], "launch");
    assert_eq!(json["program"], "src/main.rs");
}

// RED Test 2: Handle time-travel commands
#[test]
fn test_handle_time_travel_commands() {
    let mut extension = VSCodeExtension::new();

    let response = extension.handle_command("stepBackward");
    assert!(response.success);

    let response = extension.handle_command("jumpToSnapshot", json!({"position": 5}));
    assert!(response.success);
    assert_eq!(response.current_position, 5);
}

// RED Test 3: Send variable updates to IDE
#[test]
fn test_send_variable_updates() {
    let mut extension = VSCodeExtension::new();

    let snapshot1 = create_snapshot_with_vars(0, hashmap!{"x" => 10});
    let snapshot2 = create_snapshot_with_vars(1, hashmap!{"x" => 15});

    let updates = extension.compute_variable_updates(&snapshot1, &snapshot2);

    assert_eq!(updates.len(), 1);
    assert_eq!(updates[0].name, "x");
    assert_eq!(updates[0].old_value, "10");
    assert_eq!(updates[0].new_value, "15");
}

// RED Test 4: Timeline widget state management
#[test]
fn test_timeline_widget_state() {
    let mut widget = TimelineWidget::new(10); // 10 snapshots

    widget.set_position(5);
    assert_eq!(widget.position(), 5);

    let json = widget.to_json();
    assert_eq!(json["position"], 5);
    assert_eq!(json["total"], 10);
}

// RED Test 5: Integration with DAP server
#[test]
fn test_dap_server_integration() {
    let server = DapServer::new();
    let extension = VSCodeExtension::with_server(server);

    // Extension can query DAP server
    let stopped_file = extension.get_current_file();
    assert!(stopped_file.is_some());
}

// RED Test 6-10: Additional extension tests
// - Extension activation/deactivation
// - Webview panel creation
// - Configuration persistence
// - Error handling for missing recordings
// - Multi-language debugging support
```

**Implementation Files**:
- `vscode-extension/` (new directory)
  - `package.json` - Extension manifest
  - `src/extension.ts` - Main extension code
  - `src/debugAdapter.ts` - DAP adapter implementation
  - `src/timelinePanel.ts` - Timeline webview
- `server/src/services/dap/vscode_bridge.rs` (new) - Rust bridge for extension

---

## Sprint 73 Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      VSCode Extension                        │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │   Timeline   │  │  Variable    │  │  Debug       │      │
│  │   Webview    │  │  Diff View   │  │  Adapter     │      │
│  └───────┬──────┘  └──────┬───────┘  └──────┬───────┘      │
└──────────┼─────────────────┼──────────────────┼─────────────┘
           │                 │                  │
           │         JSON-RPC / DAP             │
           │                 │                  │
┌──────────┼─────────────────┼──────────────────┼─────────────┐
│          │                 │                  │              │
│    ┌─────▼─────┐     ┌────▼────┐       ┌────▼────┐        │
│    │ Timeline  │     │Variable │       │  DAP    │        │
│    │    UI     │     │  Diff   │       │ Server  │        │
│    └───────────┘     └─────────┘       └─────────┘        │
│          │                 │                  │              │
│          └─────────┬───────┴──────────────────┘              │
│                    │                                         │
│             ┌──────▼─────────┐                              │
│             │ ReplayEngine   │                              │
│             └────────────────┘                              │
│                                                              │
│         Sprint 72 Infrastructure (Complete)                  │
└─────────────────────────────────────────────────────────────┘
```

---

## Files Structure

```
server/
├── src/services/dap/
│   ├── timeline_ui.rs          # TRACE-008 (new)
│   ├── variable_diff.rs         # TRACE-009 (new)
│   ├── vscode_bridge.rs         # TRACE-010 (new)
│   └── mod.rs                   # Export new modules
└── tests/
    ├── timeline_ui_tests.rs     # 10 tests for TRACE-008
    ├── variable_diff_tests.rs   # 10 tests for TRACE-009
    └── vscode_extension_tests.rs # 10 tests for TRACE-010

vscode-extension/               # TRACE-010 (new directory)
├── package.json
├── src/
│   ├── extension.ts
│   ├── debugAdapter.ts
│   └── timelinePanel.ts
└── resources/
    └── icons/
```

---

## Success Criteria

### Functional Requirements
- [ ] Timeline UI renders execution history in terminal
- [ ] Variable diff shows what changed between snapshots
- [ ] VSCode extension integrates with DAP server
- [ ] Time-travel controls work in IDE
- [ ] All 30 tests passing (10 per ticket)

### Quality Requirements
- [ ] EXTREME TDD methodology followed (RED → GREEN → REFACTOR → COMMIT)
- [ ] UI responsive for 1000+ snapshot recordings
- [ ] Variable diff handles nested objects/arrays
- [ ] Extension compatible with VSCode 1.80+
- [ ] Zero clippy warnings

### Performance Requirements
- [ ] Timeline rendering: <100ms for 1000 snapshots
- [ ] Diff computation: <50ms per comparison
- [ ] Extension startup: <2 seconds

---

## Dependencies

### External Dependencies
- **crossterm** (Rust crate) - Terminal UI library
- **colored** (Rust crate) - ANSI color support
- **VSCode Extension API** - 1.80+ required

### Internal Dependencies
- Sprint 72 infrastructure (ExecutionRecorder, SnapshotDelta, ReplayEngine)
- Sprint 71 infrastructure (DapServer, VariableInspector)
- Tree-sitter parsers (for syntax highlighting)

---

## EXTREME TDD Methodology

### RED Phase (Write Failing Tests)
1. Write 10 tests per ticket
2. Verify tests fail with meaningful errors
3. Tests drive API design

### GREEN Phase (Implement Minimal Code)
1. Implement just enough to pass tests
2. No premature optimization
3. Focus on correctness first

### REFACTOR Phase (Optimize & Clean)
1. Extract common patterns
2. Improve readability
3. Optimize performance

### COMMIT Phase (Document & Commit)
1. Comprehensive commit messages
2. Reference ticket numbers
3. Document test results

---

## Timeline

**Total Estimated Time**: 20-26 hours (assuming full focus)

| Ticket | Estimated Time | Tests |
|--------|---------------|-------|
| TRACE-008 | 6-8 hours | 10 |
| TRACE-009 | 6-8 hours | 10 |
| TRACE-010 | 8-10 hours | 10 |
| **Total** | **20-26 hours** | **30** |

---

## Risk Assessment

### Low Risk
- Timeline UI (terminal-based, no external dependencies)
- Variable diff (builds directly on Sprint 72)

### Medium Risk
- VSCode extension (external API, TypeScript)
- Webview rendering (HTML/CSS/JS)

### Mitigation Strategies
- Start with TRACE-008 and TRACE-009 (lower risk)
- Test VSCode extension early
- Have fallback: CLI-only timeline if extension blocked

---

## Next Steps

1. **Create RED tests** for TRACE-008 (Timeline UI)
2. **Verify tests fail** with meaningful errors
3. **GREEN phase**: Implement TimelineUI
4. **Repeat** for TRACE-009 and TRACE-010

---

**Ready to begin Sprint 73!** 🚀

Let's transform time-travel debugging from an API into a visual experience that developers will love.
