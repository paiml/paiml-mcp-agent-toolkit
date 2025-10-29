// DAP (Debug Adapter Protocol) Module
// Sprint 71 - TRACE-001: DAP Protocol Server Implementation
//
// This module provides Debug Adapter Protocol support for PMAT,
// enabling integration with VSCode and other DAP-compatible debuggers.

pub mod server;
pub mod types;

pub use server::DapServer;
pub use types::*;
