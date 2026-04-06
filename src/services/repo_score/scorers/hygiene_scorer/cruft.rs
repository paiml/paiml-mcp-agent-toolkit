// C1: No Cruft Files (5 points) - No transient files, caches, build artifacts

#![cfg_attr(coverage_nightly, coverage(off))]

use super::patterns::matches_pattern;
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use ignore::WalkBuilder;
use std::path::Path;

use super::HygieneScorer;

impl HygieneScorer {
    /// Score absence of cruft files (C1: 5 points)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn score_cruft(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        tracing::debug!("HygieneScorer::score_cruft START");
        let cruft_patterns = vec![
            // Build artifacts
            "target/",
            "dist/",
            "build/",
            "out/",
            "*.pyc",
            "__pycache__/",
            "node_modules/",
            ".next/",
            ".cache/",
            // Transient files
            "*.tmp",
            "*.swp",
            "*.swo",
            "*~",
            // OS files (not in .gitignore)
            ".DS_Store",
            "Thumbs.db",
            "desktop.ini",
            // Editor backups
            "*.bak",
            "*.orig",
        ];

        let mut cruft_found = vec![];
        let mut deductions: f64 = 0.0;

        // Build directory list for performance optimization (skip heavy directories early)
        // CRITICAL: Include .git/ to prevent traversing thousands of git object files (PMAT-BUG-001)
        let skip_dirs = [
            ".git",
            "target",
            "node_modules",
            "dist",
            "build",
            ".next",
            "__pycache__",
            ".cache",
        ];

        // Use ignore::WalkBuilder to respect .gitignore (Phase 1: Root Cause Fix)
        let walker = WalkBuilder::new(repo_path)
            .hidden(false) // Don't skip hidden files by default
            .git_ignore(true) // CRITICAL: Respect .gitignore (eliminates 71% false positives)
            .git_exclude(true) // Also respect .git/info/exclude
            .max_depth(Some(5)) // Maintain max depth for performance
            .filter_entry(move |entry| {
                // Performance optimization: Skip known heavy build directories
                let path = entry.path();
                !skip_dirs.iter().any(|d| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == *d)
                        .unwrap_or(false)
                })
            })
            .build();

        tracing::debug!("HygieneScorer::score_cruft starting walker iteration");
        for entry in walker.flatten() {
            // Skip directories, only process files
            if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                continue;
            }

            let path = entry.path();
            let path_str = path.to_string_lossy();

            for pattern in &cruft_patterns {
                if matches_pattern(&path_str, pattern) {
                    cruft_found.push(path_str.to_string());
                    deductions += 0.5; // 0.5 points per cruft file, max 5 points
                    break;
                }
            }
        }

        tracing::debug!(
            "HygieneScorer::score_cruft END - found {} cruft files",
            cruft_found.len()
        );
        let score = (5.0 - deductions.min(5.0)).max(0.0);
        let mut findings = vec![];

        if cruft_found.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Hygiene".to_string(),
                message: "No cruft files detected".to_string(),
                location: None,
                impact_points: 0.0,
            });
        } else {
            for cruft_file in cruft_found.iter().take(10) {
                findings.push(Finding {
                    severity: Severity::Warning,
                    category: "Hygiene".to_string(),
                    message: format!("Cruft file found: {}", cruft_file),
                    location: Some(cruft_file.clone()),
                    impact_points: -0.5,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "C1".to_string(),
            name: "No Cruft Files".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }
}
