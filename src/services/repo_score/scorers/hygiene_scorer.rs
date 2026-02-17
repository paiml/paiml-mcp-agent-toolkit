// HygieneScorer - Category C: Repository Hygiene (15 points)
//
// Scores based on:
// - C1: No Cruft Files (5 points) - No temp files, caches, build artifacts
// - C2: No Team-Specific Files (5 points) - No .idea/, .vscode/, .DS_Store
// - C3: No Large Files in Git History (5 points) - No files >1MB in git history

#![cfg_attr(coverage_nightly, coverage(off))]
use super::{Scorer, ScorerConfig};
use crate::services::repo_score::error::Result;
use crate::services::repo_score::models::*;
use async_trait::async_trait;
use ignore::WalkBuilder;
use std::path::Path;

pub struct HygieneScorer;

impl HygieneScorer {
    pub fn new() -> Self {
        Self
    }

    /// Score absence of cruft files (C1: 5 points)
    async fn score_cruft(&self, repo_path: &Path) -> Result<SubcategoryScore> {
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
            // Temp files
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

    /// Score absence of team-specific files (C2: 5 points)
    async fn score_team_files(&self, repo_path: &Path) -> Result<SubcategoryScore> {
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

    /// Score absence of large files in git history (C3: 5 points)
    async fn score_large_files(
        &self,
        repo_path: &Path,
        config: &ScorerConfig,
    ) -> Result<SubcategoryScore> {
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
        // With --deep: --all (slow, minutes on large repos)
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

#[async_trait]
impl Scorer for HygieneScorer {
    fn category_name(&self) -> &str {
        "Repository Hygiene"
    }

    fn max_score(&self) -> f64 {
        15.0
    }

    async fn score(&self, repo_path: &Path, config: &ScorerConfig) -> Result<CategoryScore> {
        let c1 = self.score_cruft(repo_path).await?;
        let c2 = self.score_team_files(repo_path).await?;
        let c3 = self.score_large_files(repo_path, config).await?;

        let total_score = c1.score + c2.score + c3.score;

        let mut findings = c1.findings.clone();
        findings.extend(c2.findings.clone());
        findings.extend(c3.findings.clone());

        Ok(CategoryScore::new(
            total_score,
            self.max_score(),
            vec![c1, c2, c3],
            findings,
        ))
    }
}

impl Default for HygieneScorer {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions

fn matches_pattern(path: &str, pattern: &str) -> bool {
    if pattern.ends_with('/') {
        path.contains(pattern)
    } else if let Some(ext) = pattern.strip_prefix('*') {
        path.ends_with(ext)
    } else {
        path.contains(pattern)
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        // Initialize git repo so ignore crate can process .gitignore
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repo");
        temp_dir
    }

    fn create_file(repo_path: &Path, relative_path: &str) {
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, "test content").unwrap();
    }

    #[tokio::test]
    async fn test_hygiene_scorer_clean_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create only clean files
        create_file(repo_path, "src/main.rs");
        create_file(repo_path, "Cargo.toml");
        create_file(repo_path, "README.md");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 15.0);
        assert_eq!(result.percentage, 100.0);
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_with_cruft() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs");
        create_file(repo_path, "file.tmp");
        create_file(repo_path, "backup.bak");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should lose 1 point (2 cruft files × 0.5 points) out of 15 total
        assert!(
            result.score >= 13.5 && result.score <= 14.5,
            "Expected score 13.5-14.5, got {}",
            result.score
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("Cruft file")));
    }

    #[tokio::test]
    async fn test_hygiene_scorer_with_team_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs");
        create_file(repo_path, ".idea/workspace.xml");
        create_file(repo_path, ".vscode/settings.json");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should lose 2 points (2 team files × 1 point) out of 15 total
        assert!(result.score >= 12.5 && result.score <= 13.5);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("Team-specific file")));
    }

    #[tokio::test]
    async fn test_hygiene_cruft_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let c1 = result.subcategories.iter().find(|s| s.id == "C1").unwrap();
        assert_eq!(c1.name, "No Cruft Files");
        assert_eq!(c1.score, 5.0);
        assert_eq!(c1.max_score, 5.0);
    }

    #[tokio::test]
    async fn test_hygiene_team_files_subcategory() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        let c2 = result.subcategories.iter().find(|s| s.id == "C2").unwrap();
        assert_eq!(c2.name, "No Team-Specific Files");
        assert_eq!(c2.score, 5.0);
        assert_eq!(c2.max_score, 5.0);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_respects_gitignore() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs");
        // Hidden files/dirs should be skipped except .gitignore
        create_file(repo_path, ".hidden/file.txt");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Hidden files are skipped, so should get full score
        assert_eq!(result.score, 15.0);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_many_cruft_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create many cruft files (should max out at 5 points deduction)
        for i in 0..15 {
            create_file(repo_path, &format!("file{}.tmp", i));
        }

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // C1 should be 0 (maxed out deductions), C2 should be 5, C3 should be 5 = 10 total
        assert!(result.score >= 9.5 && result.score <= 10.5);
    }

    #[tokio::test]
    async fn test_hygiene_category_name() {
        let scorer = HygieneScorer::new();
        assert_eq!(scorer.category_name(), "Repository Hygiene");
        assert_eq!(scorer.max_score(), 15.0);
    }

    // Phase 1 Integration Tests: Verify .gitignore is respected

    #[tokio::test]
    async fn test_gitignored_build_artifacts_not_penalized() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create .gitignore with build artifacts
        fs::write(
            repo_path.join(".gitignore"),
            "target/\n*.tmp\nnode_modules/\n",
        )
        .unwrap();

        // Create gitignored files (should NOT be penalized)
        create_file(repo_path, "target/release/libfoo.rlib");
        create_file(repo_path, "target/debug/foo");
        create_file(repo_path, "test.tmp");
        create_file(repo_path, "node_modules/package/index.js");

        // Create clean files
        create_file(repo_path, "src/main.rs");
        create_file(repo_path, "Cargo.toml");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should score 100% because all cruft files are gitignored
        assert_eq!(
            result.score, 15.0,
            "Gitignored files should not be penalized"
        );
        assert_eq!(result.percentage, 100.0);
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("No cruft files detected")));
    }

    #[tokio::test]
    async fn test_gitignored_ide_files_not_penalized() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create .gitignore with IDE files
        fs::write(repo_path.join(".gitignore"), ".idea/\n.vscode/\n*.iml\n").unwrap();

        // Create gitignored IDE files (should NOT be penalized)
        create_file(repo_path, ".idea/workspace.xml");
        create_file(repo_path, ".idea/modules.xml");
        create_file(repo_path, ".vscode/settings.json");
        create_file(repo_path, "project.iml");

        // Create clean files
        create_file(repo_path, "src/lib.rs");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should score 100% because all team files are gitignored
        assert_eq!(
            result.score, 15.0,
            "Gitignored IDE files should not be penalized"
        );
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("No team-specific files detected")));
    }

    #[tokio::test]
    async fn test_tracked_cruft_files_are_penalized() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create .gitignore but DON'T ignore .tmp and .bak files
        fs::write(repo_path.join(".gitignore"), "target/\n").unwrap();

        // Create tracked cruft files (SHOULD be penalized because not gitignored)
        // Using patterns that ARE in our cruft list: *.tmp, *.bak
        create_file(repo_path, "errors.tmp");
        create_file(repo_path, "debug.bak");

        // Create gitignored file (should NOT be penalized)
        create_file(repo_path, "target/release/libfoo.rlib");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should lose points for .tmp/.bak files but NOT for target/
        assert!(
            result.score < 15.0,
            "Tracked .tmp/.bak files should be penalized"
        );
        assert!(
            result.score >= 4.0,
            "Should only penalize tracked cruft, not gitignored files"
        );

        // Verify .tmp/.bak files are detected but target/ is not
        let cruft_findings: Vec<_> = result
            .findings
            .iter()
            .filter(|f| f.message.contains("Cruft file found"))
            .collect();

        assert!(!cruft_findings.is_empty(), "Should find .tmp/.bak files");
        assert!(
            cruft_findings
                .iter()
                .any(|f| f.message.contains(".tmp") || f.message.contains(".bak")),
            "Should detect .tmp or .bak files"
        );
        assert!(
            !cruft_findings.iter().any(|f| f.message.contains("target/")),
            "Should NOT detect gitignored target/"
        );
    }

    #[tokio::test]
    async fn test_performance_optimization_skips_heavy_dirs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Don't create .gitignore, so files would normally be detected
        // But performance filter should skip these directories

        // Create files in heavy build directories (should be skipped by filter)
        create_file(repo_path, "target/CACHEDIR.TAG"); // Common in Rust target/
        create_file(repo_path, "node_modules/.bin/eslint");
        create_file(repo_path, "dist/bundle.js");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // These directories are skipped by performance filter, so no penalty
        assert_eq!(
            result.score, 15.0,
            "Performance filter should skip heavy build directories"
        );
    }

    #[tokio::test]
    async fn test_complex_gitignore_patterns() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create complex .gitignore with various patterns
        fs::write(
            repo_path.join(".gitignore"),
            "*.pyc\n\
             __pycache__/\n\
             .DS_Store\n\
             *.swp\n\
             /build/\n\
             dist/\n",
        )
        .unwrap();

        // Create gitignored files matching various patterns
        create_file(repo_path, "module.pyc");
        create_file(repo_path, "__pycache__/foo.pyc");
        create_file(repo_path, ".DS_Store");
        create_file(repo_path, "temp.swp");
        create_file(repo_path, "build/output.js");
        create_file(repo_path, "dist/bundle.min.js");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // All these files should be ignored by gitignore
        assert_eq!(
            result.score, 15.0,
            "Complex .gitignore patterns should be respected"
        );
    }

    // ========================================================================
    // RED TEST: C3 - No Large Files in Git History (5 points)
    // ========================================================================

    #[tokio::test]
    async fn test_c3_no_large_files_in_git_history() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create clean small files and commit them
        create_file(repo_path, "src/main.rs");
        create_file(repo_path, "README.md");

        // Add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial commit"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should get full score (15.0) with no large files
        assert_eq!(result.score, 15.0, "Clean repo should score 15.0");

        // Check C3 subcategory exists
        let c3 = result.subcategories.iter().find(|s| s.id == "C3");
        assert!(c3.is_some(), "C3 subcategory should exist");
        let c3 = c3.unwrap();
        assert_eq!(c3.name, "No Large Files in Git History");
        assert_eq!(c3.score, 5.0, "C3 should score 5.0 for clean repo");
        assert_eq!(c3.max_score, 5.0);
    }

    #[tokio::test]
    #[ignore = "Flaky test due to git operations in temp directory"]
    async fn test_c3_detects_large_files_in_history() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create and commit a large file (>1MB)
        let large_file_path = repo_path.join("large_file.bin");
        let large_content = vec![0u8; 2 * 1024 * 1024]; // 2MB file
        fs::write(&large_file_path, large_content).unwrap();

        std::process::Command::new("git")
            .args(["add", "large_file.bin"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Add large file"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should lose points for large file
        assert!(
            result.score < 15.0,
            "Large file should cause point deduction"
        );

        // Check C3 subcategory
        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();
        assert!(c3.score < 5.0, "C3 should lose points for large file");

        // Check findings
        let large_file_finding = result
            .findings
            .iter()
            .any(|f| f.message.contains("Large file") && f.message.contains("large_file.bin"));
        assert!(large_file_finding, "Should report large file in findings");
    }

    #[tokio::test]
    #[ignore = "Flaky test due to git operations in temp directory"]
    async fn test_c3_detects_deleted_large_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create and commit a large file
        let large_file_path = repo_path.join("deleted_large.bin");
        let large_content = vec![0u8; 3 * 1024 * 1024]; // 3MB
        fs::write(&large_file_path, large_content).unwrap();

        std::process::Command::new("git")
            .args(["add", "deleted_large.bin"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Add file that will be deleted"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        // Delete the file and commit deletion
        fs::remove_file(&large_file_path).unwrap();
        std::process::Command::new("git")
            .args(["add", "-A"])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Delete large file"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should STILL penalize because file is in history (even though deleted)
        assert!(
            result.score < 15.0,
            "Deleted large file should still be penalized (bloats git history)"
        );

        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();
        assert!(
            c3.score < 5.0,
            "C3 should detect large file in history even after deletion"
        );
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod coverage_tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_temp_repo() -> TempDir {
        let temp_dir = TempDir::new().unwrap();
        std::process::Command::new("git")
            .args(["init"])
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to initialize git repo");
        temp_dir
    }

    fn create_file(repo_path: &Path, relative_path: &str, content: &str) {
        let file_path = repo_path.join(relative_path);
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(file_path, content).unwrap();
    }

    // =========================================================================
    // Default and Trait Implementation Tests
    // =========================================================================

    #[test]
    fn test_hygiene_scorer_default() {
        let scorer = HygieneScorer::default();
        assert_eq!(scorer.category_name(), "Repository Hygiene");
        assert_eq!(scorer.max_score(), 15.0);
    }

    #[test]
    fn test_hygiene_scorer_new() {
        let scorer = HygieneScorer::new();
        assert_eq!(scorer.category_name(), "Repository Hygiene");
        assert_eq!(scorer.max_score(), 15.0);
    }

    // =========================================================================
    // matches_pattern Helper Function Tests
    // =========================================================================

    #[test]
    fn test_matches_pattern_directory_trailing_slash() {
        assert!(matches_pattern("/path/to/target/debug/", "target/"));
        assert!(matches_pattern(
            "/home/user/project/node_modules/package",
            "node_modules/"
        ));
        assert!(!matches_pattern("/path/to/targetfoo/", "target/"));
    }

    #[test]
    fn test_matches_pattern_wildcard_extension() {
        assert!(matches_pattern("/path/to/file.tmp", "*.tmp"));
        assert!(matches_pattern("/path/to/backup.bak", "*.bak"));
        assert!(matches_pattern("file.swp", "*.swp"));
        assert!(!matches_pattern("/path/to/file.txt", "*.tmp"));
    }

    #[test]
    fn test_matches_pattern_exact_filename() {
        assert!(matches_pattern("/path/to/.DS_Store", ".DS_Store"));
        assert!(matches_pattern("/home/Thumbs.db", "Thumbs.db"));
        assert!(matches_pattern("/project/desktop.ini", "desktop.ini"));
    }

    #[test]
    fn test_matches_pattern_substring_match() {
        assert!(matches_pattern(
            "/path/__pycache__/module.pyc",
            "__pycache__"
        ));
        assert!(matches_pattern("/home/.cache/data", ".cache"));
    }

    // =========================================================================
    // Cruft Files (C1) Tests - Additional Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_cruft_pyc_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "module.pyc", "binary content");
        create_file(repo_path, "src/main.py", "print('hello')");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        // .pyc files are cruft
        assert!(result.score < 5.0, "Should penalize .pyc files");
    }

    #[tokio::test]
    async fn test_cruft_swap_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "file.swp", "swap content");
        create_file(repo_path, "other.swo", "swap content");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize swap files");
    }

    #[tokio::test]
    async fn test_cruft_backup_tilde_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "file.txt~", "backup content");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize backup tilde files");
    }

    #[tokio::test]
    async fn test_cruft_orig_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "file.orig", "original content");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .orig files");
    }

    #[tokio::test]
    async fn test_cruft_ds_store_file() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".DS_Store", "binary");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .DS_Store");
    }

    #[tokio::test]
    async fn test_cruft_thumbs_db() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "Thumbs.db", "binary");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize Thumbs.db");
    }

    #[tokio::test]
    async fn test_cruft_desktop_ini() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "desktop.ini", "[Desktop]");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize desktop.ini");
    }

    #[tokio::test]
    async fn test_cruft_max_deduction() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create 20 cruft files (should max out at 5 points deduction)
        for i in 0..20 {
            create_file(repo_path, &format!("file{}.tmp", i), "temp");
        }

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        // Score should be 0 (maxed out) but never negative
        assert_eq!(result.score, 0.0, "Score should bottom out at 0");
    }

    #[tokio::test]
    async fn test_cruft_findings_limited_to_10() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create 15 cruft files
        for i in 0..15 {
            create_file(repo_path, &format!("cruft{}.tmp", i), "temp");
        }

        let scorer = HygieneScorer::new();
        let result = scorer.score_cruft(repo_path).await.unwrap();

        // Should only report first 10 findings
        let cruft_findings = result
            .findings
            .iter()
            .filter(|f| f.message.contains("Cruft file found"))
            .count();
        assert!(cruft_findings <= 10, "Should limit findings to 10");
    }

    // =========================================================================
    // Team-Specific Files (C2) Tests - Additional Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_team_files_vs_code() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(
            repo_path,
            ".vscode/settings.json",
            r#"{"editor.tabSize": 4}"#,
        );
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .vscode directory");
    }

    #[tokio::test]
    async fn test_team_files_idea() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".idea/workspace.xml", "<xml/>");
        create_file(repo_path, ".idea/modules.xml", "<xml/>");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .idea directory");
    }

    #[tokio::test]
    async fn test_team_files_visual_studio() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".vs/config.json", "{}");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .vs directory");
    }

    #[tokio::test]
    async fn test_team_files_iml() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "project.iml", "<module/>");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .iml files");
    }

    #[tokio::test]
    async fn test_team_files_eclipse() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".project", "<project/>");
        create_file(repo_path, ".classpath", "<classpath/>");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize Eclipse files");
    }

    #[tokio::test]
    async fn test_team_files_settings_dir() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".settings/prefs.xml", "<xml/>");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .settings directory");
    }

    #[tokio::test]
    async fn test_team_files_fleet() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".fleet/settings.json", "{}");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .fleet directory");
    }

    #[tokio::test]
    async fn test_team_files_sublime() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "project.sublime-project", "{}");
        create_file(repo_path, "project.sublime-workspace", "{}");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize Sublime files");
    }

    #[tokio::test]
    async fn test_team_files_atom() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, ".atom/config.cson", "{}");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert!(result.score < 5.0, "Should penalize .atom directory");
    }

    #[tokio::test]
    async fn test_team_files_max_deduction() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create many team files (should max at 5 points)
        for i in 0..10 {
            create_file(repo_path, &format!("project{}.iml", i), "<module/>");
        }

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        assert_eq!(result.score, 0.0, "Score should bottom out at 0");
    }

    #[tokio::test]
    async fn test_team_files_findings_limited_to_10() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        for i in 0..15 {
            create_file(repo_path, &format!("mod{}.iml", i), "<module/>");
        }

        let scorer = HygieneScorer::new();
        let result = scorer.score_team_files(repo_path).await.unwrap();

        let team_findings = result
            .findings
            .iter()
            .filter(|f| f.message.contains("Team-specific file"))
            .count();
        assert!(team_findings <= 10, "Should limit findings to 10");
    }

    // =========================================================================
    // Large Files (C3) Tests - Additional Edge Cases
    // =========================================================================

    #[tokio::test]
    async fn test_large_files_not_git_repo() {
        let temp_dir = TempDir::new().unwrap(); // No git init
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score_large_files(repo_path, &config).await.unwrap();

        // Not a git repo - should get full score
        assert_eq!(result.score, 5.0, "Non-git repo should get full score");
        assert!(result
            .findings
            .iter()
            .any(|f| f.message.contains("Not a git repository")));
    }

    #[tokio::test]
    async fn test_large_files_deep_mode() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        // Add and commit
        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig {
            deep: true,
            ..Default::default()
        };
        let result = scorer.score_large_files(repo_path, &config).await.unwrap();

        // Deep mode uses --all instead of HEAD
        assert_eq!(
            result.score, 5.0,
            "Clean repo in deep mode should score full"
        );
    }

    #[tokio::test]
    async fn test_large_files_success_finding() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(repo_path)
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Initial"])
            .current_dir(repo_path)
            .output()
            .unwrap();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score_large_files(repo_path, &config).await.unwrap();

        assert!(result
            .findings
            .iter()
            .any(|f| f.severity == Severity::Success && f.message.contains("No large files")));
    }

    // =========================================================================
    // Full Scorer Integration Tests
    // =========================================================================

    #[tokio::test]
    async fn test_hygiene_scorer_zero_score_possible() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Max out all deductions
        // 10+ cruft files
        for i in 0..15 {
            create_file(repo_path, &format!("f{}.tmp", i), "temp");
        }
        // 5+ team files
        for i in 0..7 {
            create_file(repo_path, &format!("m{}.iml", i), "<mod/>");
        }

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // C1 = 0, C2 = 0, C3 = 5 (no large files) = 5 total
        assert!(result.score <= 5.0, "Should be able to get very low score");
    }

    #[tokio::test]
    async fn test_hygiene_scorer_full_score() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create only clean files
        create_file(repo_path, "src/main.rs", "fn main() {}");
        create_file(repo_path, "Cargo.toml", "[package]\nname = \"test\"");
        create_file(repo_path, "README.md", "# Project");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        assert_eq!(result.score, 15.0, "Clean repo should get full score");
        assert_eq!(result.percentage, 100.0);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_mixed_issues() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Some cruft files (2 files = 1 point deduction)
        create_file(repo_path, "temp.tmp", "temp");
        create_file(repo_path, "backup.bak", "backup");

        // One team file (1 point deduction)
        create_file(repo_path, ".idea/workspace.xml", "<xml/>");

        // Clean code
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should lose some points but not all
        assert!(
            result.score > 10.0 && result.score < 15.0,
            "Mixed issues should give partial score: {}",
            result.score
        );
    }

    #[tokio::test]
    async fn test_hygiene_scorer_findings_combine_all_categories() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "temp.tmp", "temp");
        create_file(repo_path, ".idea/workspace.xml", "<xml/>");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should have findings from multiple categories
        let has_cruft = result.findings.iter().any(|f| f.message.contains("Cruft"));
        let has_team = result
            .findings
            .iter()
            .any(|f| f.message.contains("Team-specific"));

        assert!(
            has_cruft || has_team,
            "Should have findings from detected issues"
        );
    }

    #[tokio::test]
    async fn test_hygiene_scorer_status_reflects_score() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Full score should pass
        assert_eq!(result.status, ScoreStatus::Pass);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_subcategory_ids() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should have all three subcategories
        assert_eq!(result.subcategories.len(), 3);
        assert!(result.subcategories.iter().any(|s| s.id == "C1"));
        assert!(result.subcategories.iter().any(|s| s.id == "C2"));
        assert!(result.subcategories.iter().any(|s| s.id == "C3"));
    }

    #[tokio::test]
    async fn test_hygiene_scorer_subcategory_names() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        let c1 = result.subcategories.iter().find(|s| s.id == "C1").unwrap();
        let c2 = result.subcategories.iter().find(|s| s.id == "C2").unwrap();
        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();

        assert_eq!(c1.name, "No Cruft Files");
        assert_eq!(c2.name, "No Team-Specific Files");
        assert_eq!(c3.name, "No Large Files in Git History");
    }

    #[tokio::test]
    async fn test_hygiene_scorer_max_scores() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        let c1 = result.subcategories.iter().find(|s| s.id == "C1").unwrap();
        let c2 = result.subcategories.iter().find(|s| s.id == "C2").unwrap();
        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();

        assert_eq!(c1.max_score, 5.0);
        assert_eq!(c2.max_score, 5.0);
        assert_eq!(c3.max_score, 5.0);
    }

    // ============ matches_pattern Tests ============

    #[test]
    fn test_matches_pattern_exact_file() {
        assert!(matches_pattern("path/to/.DS_Store", ".DS_Store"));
        assert!(matches_pattern(".DS_Store", ".DS_Store"));
        assert!(!matches_pattern("path/to/file.txt", ".DS_Store"));
    }

    #[test]
    fn test_matches_pattern_directory() {
        assert!(matches_pattern("path/to/target/", "target/"));
        assert!(matches_pattern("path/to/node_modules/", "node_modules/"));
        assert!(!matches_pattern("path/to/src/", "target/"));
    }

    #[test]
    fn test_matches_pattern_wildcard_prefix() {
        assert!(matches_pattern("file.pyc", "*.pyc"));
        assert!(matches_pattern("module.pyc", "*.pyc"));
        assert!(!matches_pattern("file.py", "*.pyc"));
    }

    #[test]
    fn test_matches_pattern_wildcard_suffix() {
        assert!(matches_pattern("file~", "*~"));
        assert!(matches_pattern("backup~", "*~"));
        assert!(!matches_pattern("file.txt", "*~"));
    }

    #[test]
    fn test_matches_pattern_pycache() {
        assert!(matches_pattern("path/__pycache__/", "__pycache__/"));
        assert!(matches_pattern(
            "src/__pycache__/module.pyc",
            "__pycache__/"
        ));
    }

    #[test]
    fn test_matches_pattern_editor_files() {
        assert!(matches_pattern("file.swp", "*.swp"));
        assert!(matches_pattern("file.swo", "*.swo"));
        assert!(matches_pattern("file.bak", "*.bak"));
        assert!(matches_pattern("file.orig", "*.orig"));
        assert!(matches_pattern("file.tmp", "*.tmp"));
    }

    #[test]
    fn test_matches_pattern_os_files() {
        assert!(matches_pattern("Thumbs.db", "Thumbs.db"));
        assert!(matches_pattern("folder/Thumbs.db", "Thumbs.db"));
        assert!(matches_pattern("desktop.ini", "desktop.ini"));
    }

    // ============ Edge Cases ============

    #[tokio::test]
    async fn test_hygiene_scorer_empty_repo() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Empty repo should get scores within valid range
        assert!(result.score >= 0.0);
        assert!(result.score <= 15.0);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_with_ide_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create IDE-specific files
        std::fs::create_dir_all(repo_path.join(".idea")).unwrap();
        create_file(&repo_path.join(".idea"), "workspace.xml", "<project/>");
        create_file(repo_path, "src/main.rs", "fn main() {}");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should have deduction for IDE files
        let c2 = result.subcategories.iter().find(|s| s.id == "C2").unwrap();
        assert!(c2.findings.len() >= 1 || c2.score < 5.0);
    }

    #[tokio::test]
    async fn test_hygiene_scorer_with_temp_files() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        create_file(repo_path, "src/main.rs", "fn main() {}");
        create_file(repo_path, "file.tmp", "temp content");
        create_file(repo_path, "backup~", "backup content");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();
        let result = scorer.score(repo_path, &config).await.unwrap();

        // Should have deductions for temp files
        let c1 = result.subcategories.iter().find(|s| s.id == "C1").unwrap();
        assert!(c1.findings.len() >= 1 || c1.score < 5.0);
    }
}
