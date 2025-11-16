// DAP (Debug Adapter Protocol) Module
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
// Sprint 71 - TRACE-002: Breakpoint Management System
// Sprint 71 - TRACE-003: Variable Inspection with AST
// Sprint 72 - TRACE-005: Execution Recording Infrastructure
//
// This module provides Debug Adapter Protocol support for PMAT,
// enabling integration with VSCode and other DAP-compatible debuggers.

pub mod breakpoint_manager;
pub mod comparison_view; // Sprint 77: TIMELINE-003 - Recording comparison
pub mod execution_recorder;
pub mod recording; // Sprint 75: .pmat file format
pub mod replay_engine;
pub mod server;
pub mod timeline_player; // Sprint 77: TIMELINE-001 - Recording playback control
#[cfg(feature = "tui")]
pub mod timeline_tui; // Sprint 78: TUI-001 - Terminal event loop
pub mod timeline_ui;
pub mod types;
pub mod variable_diff;
pub mod variable_inspector;

pub use breakpoint_manager::BreakpointManager;
pub use comparison_view::{ComparisonView, DiffStatus, SyncMode};
pub use execution_recorder::ExecutionRecorder;
pub use recording::{
    CompressionLevel, Recording, RecordingMetadata, RecordingWriter, Snapshot, SnapshotSerializer,
    StackFrame,
};
pub use replay_engine::ReplayEngine;
pub use server::DapServer;
pub use timeline_player::TimelinePlayer;
pub use timeline_ui::TimelineUI;
pub use types::*;
pub use variable_diff::VariableDiff;
pub use variable_inspector::VariableInspector;
