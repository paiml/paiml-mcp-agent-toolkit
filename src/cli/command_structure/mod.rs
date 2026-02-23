#![cfg_attr(coverage_nightly, coverage(off))]
//! Command Structure - Comprehensive CLI decomposition architecture
//!
//! This module provides the complete command structure decomposition to reduce
//! the main CLI module from 145 functions and 10,304 lines to a manageable size.
//!
//! Architecture:
//! - `CommandExecutor`: Main command execution orchestrator
//! - `CommandRegistry`: Registry of all available commands
//! - `CommandGroup`: Logical grouping of related commands
//! - `ModularHandlers`: Individual command implementation modules

mod analyze;
mod demo;
mod executor;
mod generate;
mod utility;

// Re-export all public items that were previously public from the file
pub use analyze::AnalyzeCommandGroup;
pub use demo::{CommandExecutorFactory, DemoCommandGroup};
pub use generate::GenerateCommandGroup;
pub use utility::UtilityCommandGroup;

use crate::stateless_server::StatelessTemplateServer;
use std::sync::Arc;

/// Main command executor that orchestrates all CLI operations
pub struct CommandExecutor {
    server: Arc<StatelessTemplateServer>,
    registry: CommandRegistry,
}

/// Registry that manages all available commands and their handlers
#[derive(Default)]
pub struct CommandRegistry {
    generate_handlers: GenerateCommandGroup,
    analyze_handlers: AnalyzeCommandGroup,
    utility_handlers: UtilityCommandGroup,
    demo_handlers: DemoCommandGroup,
}

impl CommandExecutor {
    /// Create new command executor with server instance
    #[must_use]
    pub fn new(server: Arc<StatelessTemplateServer>) -> Self {
        Self {
            server,
            registry: CommandRegistry::default(),
        }
    }
}
