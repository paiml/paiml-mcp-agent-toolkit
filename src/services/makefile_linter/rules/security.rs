#![cfg_attr(coverage_nightly, coverage(off))]
//! Security rules for Makefile linting
//!
//! Implements high-priority security rules following TDD:
//! - ShellInjectionRule: Detect potential shell injection vulnerabilities
//! - SensitiveDataRule: Detect hardcoded secrets and credentials
//! - UnsafeCommandRule: Detect unsafe command usage
//! - PrivilegeEscalationRule: Detect potential privilege escalation

use super::{MakefileRule, Severity, Violation};
use crate::services::makefile_linter::ast::{MakefileAst, NodeData};
use regex::Regex;
use std::sync::OnceLock;

/// Detects potential shell injection vulnerabilities
#[derive(Debug, Default)]
pub struct ShellInjectionRule;

/// Detects hardcoded secrets and credentials
#[derive(Debug, Default)]
pub struct SensitiveDataRule;

/// Detects unsafe command usage
#[derive(Debug, Default)]
pub struct UnsafeCommandRule;

/// Detects potential privilege escalation
#[derive(Debug, Default)]
pub struct PrivilegeEscalationRule;

// --- Rule implementations ---
include!("security_rules.rs");

// --- Helper functions ---
include!("security_helpers.rs");

// --- Tests ---
#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::makefile_linter::parser::MakefileParser;

    include!("security_tests.rs");
}
