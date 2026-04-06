#![cfg_attr(coverage_nightly, coverage(off))]
//! v4.0 provable contracts: Formal Proof Verification (Lean sorry counting).

use crate::cli::handlers::work_contract::{EvidenceType, FalsificationResult};
use anyhow::Result;
use std::path::Path;

/// Count sorry occurrences in Lean source, respecting comments and word boundaries.
/// Handles: line comments (--), nested block comments (/- ... -/), inline block comments,
/// and word-boundary checking to avoid false positives from identifiers like `sorry_helper`.
pub(crate) fn count_lean_sorry_in_source(source: &str) -> usize {
    debug_assert!(!source.is_empty(), "source must not be empty");
    let mut count = 0;
    let mut in_block_comment = 0i32;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.starts_with("--") {
            continue;
        }

        // Strip block comments inline for same-line /- ... -/ handling
        let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);

        if in_block_comment > 0 {
            continue;
        }

        if contains_sorry_word_boundary(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips block comment content from a line, updating nesting depth.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Checks if line contains "sorry" as a standalone word (not part of an identifier).
fn contains_sorry_word_boundary(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let bytes = line.as_bytes();
    let sorry = b"sorry";
    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok = abs_idx == 0
                || !(bytes[abs_idx - 1].is_ascii_alphanumeric() || bytes[abs_idx - 1] == b'_');
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !(bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    || bytes[abs_idx + sorry.len()] == b'_');
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}

/// Test formal proof verification: count sorry occurrences in .lean files
pub(crate) fn test_formal_proof_verification(
    project_path: &Path,
    max_sorry_count: usize,
) -> Result<FalsificationResult> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    print!("Scanning .lean files for sorry... ");

    let mut total_sorry = 0usize;
    let mut sorry_files = Vec::new();

    // Walk project looking for .lean files
    for entry in walkdir::WalkDir::new(project_path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().is_some_and(|ext| ext == "lean") {
            if let Ok(content) = std::fs::read_to_string(path) {
                let count = count_lean_sorry_in_source(&content);
                if count > 0 {
                    total_sorry += count;
                    sorry_files.push(path.to_path_buf());
                }
            }
        }
    }

    if sorry_files.is_empty() && total_sorry == 0 {
        return Ok(FalsificationResult::passed(
            "No .lean files with sorry found".to_string(),
        ));
    }

    if total_sorry <= max_sorry_count {
        Ok(FalsificationResult::passed(format!(
            "{} sorry occurrence(s) within threshold (max: {})",
            total_sorry, max_sorry_count
        )))
    } else {
        Ok(FalsificationResult::failed(
            format!(
                "{} sorry occurrence(s) exceed threshold (max: {}), in: {}",
                total_sorry,
                max_sorry_count,
                sorry_files
                    .iter()
                    .map(|p| p.display().to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            EvidenceType::FileList(sorry_files),
        ))
    }
}
