//! Test discovery and fixing handlers for GH-98
//!
//! Systematic test fixing agent with 5-phase automation:
//! 1. Discovery: Run tests, capture ALL failures
//! 2. Categorization: Group by root cause
//! 3. Bulk Marking: Add #[ignore] with reasons
//! 4. Verification: Ensure all tests pass
//! 5. Tracking: Create GitHub issues

use crate::cli::commands::TestDiscoveryCommands;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// Test failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestFailure {
    /// Full test name (module::function)
    pub name: String,
    /// File path where test is defined
    pub file: PathBuf,
    /// Line number in file
    pub line: Option<u32>,
    /// Failure reason/message
    pub reason: String,
    /// Failure category
    pub category: FailureCategory,
    /// Test duration (if available)
    pub duration_ms: Option<u64>,
}

/// Failure categories for triage
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FailureCategory {
    /// Test timed out
    Timeout,
    /// Compilation error
    CompileError,
    /// Runtime panic/error
    RuntimeError,
    /// Assertion failure
    AssertionFailure,
    /// Unknown/uncategorized
    Unknown,
}

/// Test discovery report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryReport {
    /// Total tests discovered
    pub total_tests: usize,
    /// Number of failures
    pub failures: usize,
    /// List of all failures
    pub test_failures: Vec<TestFailure>,
    /// Discovery timestamp
    pub timestamp: String,
    /// Command used
    pub command: String,
}

/// Categorized failure group
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureGroup {
    /// Root cause description
    pub root_cause: String,
    /// Suggested ignore reason
    pub ignore_reason: String,
    /// Priority: 1 (fix now) to 5 (ignore indefinitely)
    pub priority: u8,
    /// Tests in this category
    pub tests: Vec<TestFailure>,
}

/// Categorization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategorizationReport {
    /// Total failures
    pub total_failures: usize,
    /// Grouped failures
    pub groups: Vec<FailureGroup>,
    /// Timestamp
    pub timestamp: String,
}

/// GitHub issue template for test failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestIssueTemplate {
    /// Issue title
    pub title: String,
    /// Issue body (markdown)
    pub body: String,
    /// Labels to apply
    pub labels: Vec<String>,
    /// Root cause category
    pub category: String,
    /// Number of tests in this issue
    pub test_count: usize,
}

/// Tickets report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TicketsReport {
    /// Generated tickets
    pub tickets: Vec<TestIssueTemplate>,
    /// Total tests covered
    pub total_tests: usize,
    /// Timestamp
    pub timestamp: String,
}

/// Handle test-discovery command
pub async fn handle_test_discovery_command(command: TestDiscoveryCommands) -> Result<()> {
    match command {
        TestDiscoveryCommands::Run {
            path,
            output,
            use_nextest,
            timeout,
        } => handle_discovery_run(&path, &output, use_nextest, timeout).await,

        TestDiscoveryCommands::Categorize { input, output } => {
            handle_categorization(&input, &output).await
        }

        TestDiscoveryCommands::Mark { input, apply } => handle_mark(&input, apply).await,

        TestDiscoveryCommands::Verify { path } => handle_verify(&path).await,

        TestDiscoveryCommands::CreateTickets {
            input,
            create,
            output,
            repo,
            labels,
        } => {
            handle_create_tickets(&input, create, output.as_deref(), repo.as_deref(), labels).await
        }

        TestDiscoveryCommands::ResolvePaths {
            input,
            output,
            path,
        } => handle_resolve_paths(&input, &output, &path).await,
    }
}

// Phase 1: Discovery - test execution and output parsing
include!("test_discovery_handlers_discovery.rs");

// Phase 2: Categorization - failure grouping and pattern extraction
include!("test_discovery_handlers_categorization.rs");

// Phase 3-4: Marking and verification
include!("test_discovery_handlers_marking.rs");

// Phase 5: Ticket generation and GitHub integration
include!("test_discovery_handlers_tickets.rs");

// Phase 6: Path resolution and utility functions
include!("test_discovery_handlers_resolution.rs");

// Tests
include!("test_discovery_handlers_tests.rs");
