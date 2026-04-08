#![cfg_attr(coverage_nightly, coverage(off))]
// Git Commit Parser (GH-RAG-001)
// Toyota Way: Genchi Genbutsu - Direct git data analysis
// Spec: docs/specifications/git-history-rag-integration.md

use anyhow::{Context, Result};
use git2::{Commit, DiffOptions, Repository, Sort};
use std::path::Path;

/// Type of file change in a commit
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
}

impl ChangeType {
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    /// As str.
    pub fn as_str(&self) -> &'static str {
        match self {
            ChangeType::Added => "A",
            ChangeType::Modified => "M",
            ChangeType::Deleted => "D",
            ChangeType::Renamed => "R",
        }
    }
}

/// Information about a file changed in a commit
#[derive(Debug, Clone)]
pub struct FileChange {
    pub path: String,
    pub change_type: ChangeType,
    pub lines_added: u32,
    pub lines_deleted: u32,
}

/// Parsed commit information
#[derive(Debug, Clone)]
pub struct CommitInfo {
    pub hash: String,
    pub message_subject: String,
    pub message_body: Option<String>,
    pub author_name: String,
    pub author_email: String,
    pub timestamp: i64,
    pub is_merge: bool,
    pub is_fix: bool,
    pub is_feat: bool,
    pub issue_refs: Vec<String>,
    pub files: Vec<FileChange>,
}

impl CommitInfo {
    /// Full commit message (subject + body)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn full_message(&self) -> String {
        match &self.message_body {
            Some(body) if !body.is_empty() => format!("{}\n\n{}", self.message_subject, body),
            _ => self.message_subject.clone(),
        }
    }

    /// Check if this is a meaningful commit for indexing
    /// Skips: merge commits with no custom message, very short messages
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "check_compliance")]
    pub fn is_indexable(&self) -> bool {
        // Skip merge commits with generic messages
        if self.is_merge && self.message_subject.starts_with("Merge ") {
            return false;
        }
        // Skip very short messages (less than 10 chars)
        if self.message_subject.len() < 10 {
            return false;
        }
        true
    }
}

/// Git commit parser using libgit2
pub struct CommitParser {
    repo: Repository,
}

// Parsing methods: open, parse_commits, parse_commit, split_message,
// is_fix_commit, is_feat_commit, extract_issue_refs, get_file_changes,
// head_commit_hash
include!("commit_parser_parsing.rs");

// Unit tests
include!("commit_parser_tests.rs");
