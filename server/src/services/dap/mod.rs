// DAP (Debug Adapter Protocol) Module
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
// Sprint 71 - TRACE-002: Breakpoint Management System
// Sprint 71 - TRACE-003: Variable Inspection with AST
// Sprint 72 - TRACE-005: Execution Recording Infrastructure
//
// This module provides Debug Adapter Protocol support for PMAT,
// enabling integration with VSCode and other DAP-compatible debuggers.

pub mod breakpoint_manager;
pub mod execution_recorder;
pub mod server;
pub mod types;
pub mod variable_inspector;

pub use breakpoint_manager::BreakpointManager;
pub use execution_recorder::ExecutionRecorder;
pub use server::DapServer;
pub use types::*;
pub use variable_inspector::VariableInspector;
