// C3: No Large Files in Git History (5 points) - No files >1MB in git history

#![cfg_attr(coverage_nightly, coverage(off))]

use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use crate::services::repo_score::scorers::ScorerConfig;
use std::path::Path;

use super::HygieneScorer;

impl HygieneScorer {
    /// Score absence of large files in git history (C3: 5 points)
    pub(crate) async fn score_large_files(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        tracing::debug!("HygieneScorer::score_large_files START");
        let mut large_files_found = vec![];
        let mut deductions: f64 = 0.0;
        const ONE_MB: u64 = 1024 * 1024;

        // Check if this is a git repository
        tracing::debug!("score_large_files: checking if .git exists");
        if !repo_path.join(".git").exists() {
            tracing::debug!("score_large_files: not a git repo, returning full score");
            // Not a git repo - give full score
            return Ok(SubcategoryScore {
                id: "C3".to_string(),
                name: "No Large Files in Git History".to_string(),
                score: 5.0,
                max_score: 5.0,
                findings: vec![Finding {
                    severity: Severity::Info,
                    category: "Hygiene".to_string(),
                    message: "Not a git repository (skipping git history check)".to_string(),
                    location: None,
                    impact_points: 0.0,
                }],
            });
        }

        // PMAT-PERF-001: Default to HEAD only (fast), use --deep for full history
        // Following churn command best practices: default=fast, --deep=thorough
        // PMAT-DEADLOCK-FIX: Use piped command to avoid stdin/stdout deadlock
        // Use git rev-list | git cat-file --batch-check to stream efficiently
        // Default: HEAD only (fast, <1s even on large repos)
        // With --deep: --all (expensive, minutes on large repos)
        let rev_list_target = if config.deep { "--all" } else { "HEAD" };
        let git_command = format!(
            "git rev-list --objects {} | git cat-file --batch-check='%(objecttype) %(objectname) %(objectsize) %(rest)'",
            rev_list_target
        );
        tracing::debug!("score_large_files: running piped command: {}", git_command);

        // CRITICAL: Use shell to pipe commands, avoiding Rust subprocess deadlock
        let output = std::process::Command::new("sh")
            .arg("-c")
            .arg(&git_command)
            .current_dir(repo_path)
            .output();

        tracing::debug!("score_large_files: piped command completed");
        if let Ok(result) = output {
            tracing::debug!("score_large_files: checking command status");
            if result.status.success() {
                tracing::debug!("score_large_files: command succeeded, parsing output");
                let batch_output = String::from_utf8_lossy(&result.stdout);
                tracing::debug!(
                    "score_large_files: parsed {} bytes from command output",
                    batch_output.len()
                );

                for line in batch_output.lines() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 4 && parts[0] == "blob" {
                        if let Ok(size) = parts[2].parse::<u64>() {
                            if size > ONE_MB {
                                let filename = parts[3..].join(" ");
                                // #239: Only penalize files that exist in the current tree.
                                // Files removed from HEAD should not incur penalty.
                                let file_path = repo_path.join(&filename);
                                if !file_path.exists() {
                                    continue;
                                }
                                let size_mb = size as f64 / ONE_MB as f64;
                                large_files_found.push((filename.clone(), size_mb));
                                deductions += 1.0; // 1 point per large file, max 5
                            }
                        }
                    }
                }
            }
        }

        tracing::debug!(
            "score_large_files: found {} large files, calculating score",
            large_files_found.len()
        );
        let score = (5.0 - deductions.min(5.0)).max(0.0);
        let mut findings = vec![];

        if large_files_found.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Hygiene".to_string(),
                message: "No large files (>1MB) detected in git history".to_string(),
                location: None,
                impact_points: 0.0,
            });
        } else {
            for (filename, size_mb) in large_files_found.iter().take(10) {
                findings.push(Finding {
                    severity: Severity::Error,
                    category: "Hygiene".to_string(),
                    message: format!("Large file in git history: {} ({:.2}MB)", filename, size_mb),
                    location: Some(filename.clone()),
                    impact_points: -1.0,
                });
            }

            if large_files_found.len() > 10 {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Hygiene".to_string(),
                    message: format!(
                        "... and {} more large files (total: {} files >1MB)",
                        large_files_found.len() - 10,
                        large_files_found.len()
                    ),
                    location: None,
                    impact_points: 0.0,
                });
            }
        }

        tracing::debug!(
            "HygieneScorer::score_large_files END - returning score {}",
            score
        );
        Ok(SubcategoryScore {
            id: "C3".to_string(),
            name: "No Large Files in Git History".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }
}
