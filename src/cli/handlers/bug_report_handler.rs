// Bug report CLI handler (GH-81)
// Toyota Way: Kaizen - continuous improvement through feedback
//
// Split into submodules via include!():
//   - bug_report_handler_commands.rs: handle_bug_report, create_github_issue, capture helpers
//   - bug_report_handler_tests.rs: unit tests

use anyhow::{Context, Result};
use std::process::Command;

use crate::services::error_capture::{
    clear_error, generate_issue_markdown, load_error, CapturedError,
};

include!("bug_report_handler_commands.rs");
include!("bug_report_handler_tests.rs");
