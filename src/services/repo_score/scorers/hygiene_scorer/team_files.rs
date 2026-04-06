// C2: No Team-Specific Files (5 points) - No .idea/, .vscode/, .DS_Store

#![cfg_attr(coverage_nightly, coverage(off))]

use super::patterns::matches_pattern;
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use ignore::WalkBuilder;
use std::path::Path;

use super::HygieneScorer;

impl HygieneScorer {
    /// Score absence of team-specific files (C2: 5 points)
    #[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
    pub(crate) async fn score_team_files(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        debug_assert!(
            repo_path.exists(),
            "repo_path must exist: {}",
            repo_path.display()
        );
        tracing::debug!("HygieneScorer::score_team_files START");
        let team_patterns = vec![
            ".idea/",
            ".vscode/",
            ".vs/",
            "*.iml",
            ".project",
            ".classpath",
            ".settings/",
            ".fleet/",
            ".atom/",
            ".sublime-project",
            ".sublime-workspace",
        ];

        let mut team_files_found = vec![];
        let mut deductions: f64 = 0.0;

        // Performance optimization: Skip heavy directories (PMAT-BUG-001)
        let skip_dirs_team = [".git", "target", "node_modules", "dist", "build"];

        // Use ignore::WalkBuilder to respect .gitignore
        let walker = WalkBuilder::new(repo_path)
            .hidden(false) // Check hidden dirs like .idea/, .vscode/
            .git_ignore(true) // Respect .gitignore
            .git_exclude(true) // Respect .git/info/exclude
            .max_depth(Some(3)) // Shallower depth for team files
            .filter_entry(move |entry| {
                // CRITICAL: Skip .git/ directory to prevent traversing thousands of files
                let path = entry.path();
                !skip_dirs_team.iter().any(|d| {
                    path.file_name()
                        .and_then(|n| n.to_str())
                        .map(|n| n == *d)
                        .unwrap_or(false)
                })
            })
            .build();

        tracing::debug!("HygieneScorer::score_team_files starting walker iteration");
        for entry in walker.flatten() {
            // Check both files and directories (directories like .idea/, .vscode/ are problematic)
            let path = entry.path();
            let path_str = path.to_string_lossy();

            for pattern in &team_patterns {
                if matches_pattern(&path_str, pattern) {
                    team_files_found.push(path_str.to_string());
                    deductions += 1.0; // 1 point per team file, max 5 points
                    break;
                }
            }
        }

        tracing::debug!(
            "HygieneScorer::score_team_files END - found {} team files",
            team_files_found.len()
        );
        let score = (5.0 - deductions.min(5.0)).max(0.0);
        let mut findings = vec![];

        if team_files_found.is_empty() {
            findings.push(Finding {
                severity: Severity::Success,
                category: "Hygiene".to_string(),
                message: "No team-specific files detected".to_string(),
                location: None,
                impact_points: 0.0,
            });
        } else {
            for team_file in team_files_found.iter().take(10) {
                findings.push(Finding {
                    severity: Severity::Error,
                    category: "Hygiene".to_string(),
                    message: format!("Team-specific file found: {}", team_file),
                    location: Some(team_file.clone()),
                    impact_points: -1.0,
                });
            }
        }

        Ok(SubcategoryScore {
            id: "C2".to_string(),
            name: "No Team-Specific Files".to_string(),
            score,
            max_score: 5.0,
            findings,
        })
    }
}
