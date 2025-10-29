// DAP (Debug Adapter Protocol) Module
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
// Sprint 71 - TRACE-002: Breakpoint Management System
//
// This module provides Debug Adapter Protocol support for PMAT,
// enabling integration with VSCode and other DAP-compatible debuggers.

pub mod breakpoint_manager;
pub mod server;
pub mod types;

pub use breakpoint_manager::BreakpointManager;
pub use server::DapServer;
pub use types::*;
