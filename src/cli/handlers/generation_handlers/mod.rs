#![cfg_attr(coverage_nightly, coverage(off))]
//! Template generation and scaffolding handlers
//!
//! This module contains the extracted implementations for template generation,
//! project scaffolding, and template validation operations.
//!
//! ## Submodules
//! - `template_handlers`: Template generation, scaffolding, and validation
//! - `agent_scaffold`: Agent scaffolding with context builders
//! - `wasm_scaffold`: WASM project scaffolding

mod agent_scaffold;
mod template_handlers;
mod wasm_scaffold;

pub use template_handlers::{handle_generate, handle_scaffold, handle_validate};

pub use agent_scaffold::{
    handle_list_agent_templates, handle_scaffold_agent, handle_validate_agent_template,
    ScaffoldAgentParams,
};

pub use wasm_scaffold::{handle_scaffold_wasm, ScaffoldWasmParams};

#[cfg(test)]
mod tests;

#[cfg(test)]
mod property_tests;
