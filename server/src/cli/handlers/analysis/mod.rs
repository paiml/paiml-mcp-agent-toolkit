//! Analysis command handlers using uniform contracts
//!
//! This module contains handlers that have been migrated to use the uniform contracts system
//! as part of Sprint 1 of the contract migration initiative.

pub mod code_quality;
pub mod complexity;
pub mod dependencies;
pub mod duplication;
pub mod ml_analysis;
pub mod technical_debt;

// Re-export the main handlers
pub use complexity::handle_complexity;
