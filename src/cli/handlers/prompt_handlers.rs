#![cfg_attr(coverage_nightly, coverage(off))]
//! Prompt command handlers for workflow prompts
//!
//! This module provides handlers for the `pmat prompt` command, which displays
//! pre-configured workflow prompts that enforce EXTREME TDD and Toyota Way quality principles.
//!
//! ## Module structure
//! - `prompt_handlers_core.rs`: handle_prompt, list_prompts, show_prompt, handle_prompt_command
//! - `prompt_handlers_generators.rs`: handle_generate_prompt, handle_ticket_prompt, handle_implement_prompt, handle_scaffold_repo_prompt
//! - `prompt_handlers_specialized.rs`: handle_comply_prompt, handle_book_prompt, handle_repo_image_prompt, handle_github_issue_prompt
//! - `prompt_handlers_tests.rs`: unit tests and property tests

use crate::cli::commands::PromptCommands;
use crate::cli::PromptOutputFormat;
use crate::models::prompt_model::WorkflowPrompt;
use crate::prompts::DefectAwarePromptGenerator;
use anyhow::{Context, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::path::PathBuf;

/// List of all available prompts (embedded at compile time)
const PROMPTS: &[(&str, &str)] = &[
    (
        "code-coverage",
        include_str!("../../../prompts/code-coverage.yaml"),
    ),
    (
        "clean-repo-cruft",
        include_str!("../../../prompts/clean-repo-cruft.yaml"),
    ),
    ("continue", include_str!("../../../prompts/continue.yaml")),
    (
        "assert-cmd-testing",
        include_str!("../../../prompts/assert-cmd-testing.yaml"),
    ),
    (
        "documentation",
        include_str!("../../../prompts/documentation.yaml"),
    ),
    ("debug", include_str!("../../../prompts/debug.yaml")),
    (
        "mutation-testing",
        include_str!("../../../prompts/mutation-testing.yaml"),
    ),
    (
        "performance-optimization",
        include_str!("../../../prompts/performance-optimization.yaml"),
    ),
    (
        "quality-enforcement",
        include_str!("../../../prompts/quality-enforcement.yaml"),
    ),
    (
        "refactor-hotspots",
        include_str!("../../../prompts/refactor-hotspots.yaml"),
    ),
    (
        "security-audit",
        include_str!("../../../prompts/security-audit.yaml"),
    ),
    (
        "comply-pmat",
        include_str!("../../../prompts/comply-pmat.yaml"),
    ),
    (
        "book-documentation",
        include_str!("../../../prompts/book-documentation.yaml"),
    ),
    (
        "repo-image",
        include_str!("../../../prompts/repo-image.yaml"),
    ),
    (
        "github-ticket",
        include_str!("../../../prompts/github-ticket.yaml"),
    ),
];

// Core prompt handling: handle_prompt, list_prompts, show_prompt, handle_prompt_command
include!("prompt_handlers_core.rs");

// Generator handlers: handle_generate_prompt, handle_ticket_prompt,
// handle_implement_prompt, handle_scaffold_repo_prompt
include!("prompt_handlers_generators.rs");

// Specialized prompt handlers: handle_comply_prompt, handle_book_prompt,
// handle_repo_image_prompt, handle_github_issue_prompt
include!("prompt_handlers_specialized.rs");

// Tests and property tests
include!("prompt_handlers_tests.rs");
