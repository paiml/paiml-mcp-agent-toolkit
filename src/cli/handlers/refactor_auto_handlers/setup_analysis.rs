// AI-Powered Automated Refactoring Handler
//
// FULLY IMPLEMENTED state machine for AI-driven automated refactoring:
// - Finds files with EXTREME quality violations (complexity, SATD, coverage)
// - Generates comprehensive rewrite requests for AI agents
// - Waits for AI to provide refactored code that meets ALL quality standards:
//   * Functions with complexity <= 10 (target: 5)
//   * Test coverage >= 80% per file
//   * Zero SATD comments (self-admitted technical debt)
//   * All lint violations fixed (pedantic + nursery)
// - Verifies the refactored code compiles and passes tests
// - Iterates until entire project meets RIGID extreme quality standards
//
// This is an AI-powered tool that outputs requests for AI agents to refactor code.
//
// Split into include!() sub-files for file health compliance (CB-040).
// Sub-files share this module's scope (all imports from parent mod.rs).
// Each sub-file contains a logical grouping of related functionality.

// #![allow(dead_code)] // Functions are being integrated iteratively

use crate::cli::RefactorAutoOutputFormat;

// Types extracted to refactor_auto_types.rs for file health compliance (CB-040)
pub use super::refactor_auto_types::{
    AstMetadata, FileRewritePlan, FixStrategy, FunctionInfo, QualityMetrics, RefactorPhase,
    RefactorProgress, ViolationWithContext,
};
use super::refactor_auto_types::{handle_markdown_analysis, is_markdown_file};

use anyhow::{Context, Result};
use regex;
use serde::{Deserialize, Serialize};
use serde_json;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

// Type definitions: RefactorAutoConfig, RefactorState, QualityProfile,
// RefactorConfig, RefactorMode, PatternConfig, OutputConfig, RefactorContext,
// GitHubIssueRef, GitHubIssueContent, ProjectQualityAnalysis, ComplexityAnalysis,
// SatdAnalysis, CoverageAnalysis, ComplexityViolation, SatdComment, UncoveredLine,
// RefactoringRequest, RefactoringType, RefactoringPriority, RefactoringEffort,
// LintHotspotJsonResponse, LintHotspotJson, ViolationDetailJson
include!("setup_analysis_types.rs");

// Setup and discovery: setup_refactoring_context, load_ignore_patterns,
// discover_source_files
include!("setup_analysis_context.rs");

// GitHub issue integration: handle_special_modes, process_github_issue,
// parse_github_issue_url, fetch_github_issue_content,
// extract_target_files_from_issue
include!("setup_analysis_github.rs");

// Quality analysis: analyze_project_quality, analyze_project_lint_violations,
// analyze_project_complexity, analyze_project_satd, analyze_project_coverage,
// parse_coverage_from_output
include!("setup_analysis_quality.rs");

// Refactoring request generation: generate_refactoring_requests,
// create_complexity_reduction_request, create_lint_fix_requests,
// create_satd_cleanup_requests, create_coverage_improvement_requests
include!("setup_analysis_requests.rs");
