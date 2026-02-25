#![cfg_attr(coverage_nightly, coverage(off))]
use crate::cli::DagType;
use crate::models::churn::CodeChurnAnalysis;
use crate::models::dag::DependencyGraph;
use crate::models::template::TemplateResource;
use crate::services::cache::base::CacheStrategy;
use crate::services::context::FileContext;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, UNIX_EPOCH};

// ── Struct / type definitions ────────────────────────────────────────

/// AST cache strategy – file modification tracking.
#[derive(Clone)]
pub struct AstCacheStrategy;

/// Template cache strategy – embedded, immutable resources.
#[derive(Clone)]
pub struct TemplateCacheStrategy;

/// DAG cache strategy – dependency graph analysis results.
#[derive(Clone)]
pub struct DagCacheStrategy;

/// Code churn cache strategy – Git commit tracking.
#[derive(Clone)]
pub struct ChurnCacheStrategy;

/// Git statistics cache strategy – repository metadata.
#[derive(Clone)]
pub struct GitStatsCacheStrategy;

/// Git repository statistics structure.
///
/// Contains metadata about a Git repository including commit count,
/// authors, current branch, and HEAD commit SHA.
///
/// # Examples
///
/// ```rust,no_run
/// use pmat::services::cache::GitStats;
///
/// let stats = GitStats {
///     total_commits: 150,
///     authors: vec![
///         "alice@example.com".to_string(),
///         "bob@example.com".to_string(),
///     ],
///     branch: "main".to_string(),
///     head_commit: "a1b2c3d4e5f6".to_string(),
/// };
///
/// assert_eq!(stats.total_commits, 150);
/// assert_eq!(stats.authors.len(), 2);
/// assert_eq!(stats.branch, "main");
/// ```
#[derive(Clone)]
pub struct GitStats {
    pub total_commits: usize,
    pub authors: Vec<String>,
    pub branch: String,
    pub head_commit: String,
}

// ── Impl includes ────────────────────────────────────────────────────

include!("strategies_ast.rs");
include!("strategies_template.rs");
include!("strategies_dag.rs");
include!("strategies_git.rs");
include!("strategies_tests.rs");
