#![cfg_attr(coverage_nightly, coverage(off))]
// Work commands - extracted for file health (CB-040)
// Split into submodules via include!() for maintainability

use super::misc_commands::AnnotateOutputFormat;
use clap::Subcommand;
use std::path::PathBuf;

// --- WorkCommands enum (CRUD + workflow subcommands) ---
include!("work_commands_work.rs");

// --- QaWorkCommands enum (Toyota Way quality validation) ---
include!("work_commands_qa.rs");

// --- TestDiscoveryCommands enum (systematic test fixing) ---
include!("work_commands_test_discovery.rs");

/// Sync direction for work sync command
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum SyncDirection {
    /// Sync YAML -> GitHub
    YamlToGithub,
    /// Sync GitHub -> YAML
    GithubToYaml,
    /// Full bidirectional sync
    Full,
}

/// Work priority for CLI (maps to roadmap::Priority)
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq, Default)]
pub enum WorkPriority {
    /// Low priority
    Low,
    /// Medium priority (default)
    #[default]
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

impl WorkPriority {
    /// Convert to roadmap Priority
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn to_roadmap_priority(self) -> crate::models::roadmap::Priority {
        match self {
            WorkPriority::Low => crate::models::roadmap::Priority::Low,
            WorkPriority::Medium => crate::models::roadmap::Priority::Medium,
            WorkPriority::High => crate::models::roadmap::Priority::High,
            WorkPriority::Critical => crate::models::roadmap::Priority::Critical,
        }
    }
}

/// Task type for QA checklist customization
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaTaskType {
    /// New feature implementation
    Feature,
    /// Bug fix
    Bugfix,
    /// Code refactoring
    Refactor,
    /// Documentation update
    Docs,
    /// Performance optimization
    Performance,
    /// Security fix
    Security,
}

/// Output format for QA commands
#[derive(Debug, Clone, Copy, clap::ValueEnum, PartialEq)]
pub enum QaOutputFormat {
    /// Human-readable text
    Text,
    /// JSON for CI/CD
    Json,
    /// YAML config format
    Yaml,
    /// Markdown documentation
    Markdown,
}

#[cfg(test)]
mod work_commands_tests {
    use super::*;
    use crate::models::roadmap::Priority;

    #[test]
    fn test_to_roadmap_priority_low() {
        assert!(matches!(
            WorkPriority::Low.to_roadmap_priority(),
            Priority::Low
        ));
    }

    #[test]
    fn test_to_roadmap_priority_medium() {
        assert!(matches!(
            WorkPriority::Medium.to_roadmap_priority(),
            Priority::Medium
        ));
    }

    #[test]
    fn test_to_roadmap_priority_high() {
        assert!(matches!(
            WorkPriority::High.to_roadmap_priority(),
            Priority::High
        ));
    }

    #[test]
    fn test_to_roadmap_priority_critical() {
        assert!(matches!(
            WorkPriority::Critical.to_roadmap_priority(),
            Priority::Critical
        ));
    }
}
