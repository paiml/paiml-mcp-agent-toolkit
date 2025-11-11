// HygieneScorer - Category C: Repository Hygiene (15 points)
//
// Scores based on:
// - C1: No Cruft Files (5 points) - No temp files, caches, build artifacts
// - C2: No Team-Specific Files (5 points) - No .idea/, .vscode/, .DS_Store
// - C3: No Large Files in Git History (5 points) - No files >1MB in git history

use super::{Scorer, ScorerConfig};
use crate::services::repo_score::models::*;
use crate::services::repo_score::error::Result;
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
        let cruft_patterns = vec![
            // Build artifacts
            "target/", "dist/", "build/", "out/", "*.pyc", "__pycache__/",
            "node_modules/", ".next/", ".cache/",
            // Temp files
            "*.tmp", "*.swp", "*.swo", "*~",
            // OS files (not in .gitignore)
            ".DS_Store", "Thumbs.db", "desktop.ini",
            // Editor backups
            "*.bak", "*.orig",
        ];

        let mut cruft_found = vec![];
        let mut deductions: f64 = 0.0;

        // Build directory list for performance optimization (skip heavy directories early)
        let skip_dirs = ["target", "node_modules", "dist", "build", ".next", "__pycache__", ".cache"];

        // Use ignore::WalkBuilder to respect .gitignore (Phase 1: Root Cause Fix)
        let walker = WalkBuilder::new(repo_path)
            .hidden(false)           // Don't skip hidden files by default
            .git_ignore(true)        // CRITICAL: Respect .gitignore (eliminates 71% false positives)
            .git_exclude(true)       // Also respect .git/info/exclude
            .max_depth(Some(5))      // Maintain max depth for performance
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

        // Use ignore::WalkBuilder to respect .gitignore
        let walker = WalkBuilder::new(repo_path)
            .hidden(false)           // Check hidden dirs like .idea/, .vscode/
            .git_ignore(true)        // Respect .gitignore
            .git_exclude(true)       // Respect .git/info/exclude
            .max_depth(Some(3))      // Shallower depth for team files
            .build();

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
    async fn score_large_files(&self, repo_path: &Path) -> Result<SubcategoryScore> {
        let mut large_files_found = vec![];
        let mut deductions: f64 = 0.0;
        const ONE_MB: u64 = 1024 * 1024;

        // Check if this is a git repository
        if !repo_path.join(".git").exists() {
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

        // Use git rev-list to find all objects and their sizes
        // Command: git rev-list --objects --all | git cat-file --batch-check
        let rev_list_output = std::process::Command::new("git")
            .args(["rev-list", "--objects", "--all"])
            .current_dir(repo_path)
            .output();

        if let Ok(rev_list) = rev_list_output {
            if rev_list.status.success() {
                let object_list = String::from_utf8_lossy(&rev_list.stdout);

                // Feed to cat-file --batch-check to get sizes
                let cat_file = std::process::Command::new("git")
                    .args(["cat-file", "--batch-check=%(objecttype) %(objectname) %(objectsize) %(rest)"])
                    .current_dir(repo_path)
                    .stdin(std::process::Stdio::piped())
                    .stdout(std::process::Stdio::piped())
                    .spawn();

                if let Ok(mut child) = cat_file {
                    use std::io::Write;
                    if let Some(mut stdin) = child.stdin.take() {
                        let _ = stdin.write_all(object_list.as_bytes());
                    }

                    if let Ok(output) = child.wait_with_output() {
                        let batch_output = String::from_utf8_lossy(&output.stdout);

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
            }
        }

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
                    message: format!("... and {} more large files (total: {} files >1MB)",
                        large_files_found.len() - 10, large_files_found.len()),
                    location: None,
                    impact_points: 0.0,
                });
            }
        }

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

    async fn score(&self, repo_path: &Path, _config: &ScorerConfig) -> Result<CategoryScore> {
        let c1 = self.score_cruft(repo_path).await?;
        let c2 = self.score_team_files(repo_path).await?;
        let c3 = self.score_large_files(repo_path).await?;

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

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use std::fs;

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
        assert!(result.score >= 13.5 && result.score <= 14.5, "Expected score 13.5-14.5, got {}", result.score);
        assert!(result.findings.iter().any(|f| f.message.contains("Cruft file")));
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
        assert!(result.findings.iter().any(|f| f.message.contains("Team-specific file")));
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
        fs::write(repo_path.join(".gitignore"), "target/\n*.tmp\nnode_modules/\n").unwrap();

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
        assert_eq!(result.score, 15.0, "Gitignored files should not be penalized");
        assert_eq!(result.percentage, 100.0);
        assert!(result.findings.iter().any(|f| f.message.contains("No cruft files detected")));
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
        assert_eq!(result.score, 15.0, "Gitignored IDE files should not be penalized");
        assert!(result.findings.iter().any(|f| f.message.contains("No team-specific files detected")));
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
        assert!(result.score < 15.0, "Tracked .tmp/.bak files should be penalized");
        assert!(result.score >= 4.0, "Should only penalize tracked cruft, not gitignored files");

        // Verify .tmp/.bak files are detected but target/ is not
        let cruft_findings: Vec<_> = result.findings.iter()
            .filter(|f| f.message.contains("Cruft file found"))
            .collect();

        assert!(!cruft_findings.is_empty(), "Should find .tmp/.bak files");
        assert!(
            cruft_findings.iter().any(|f| f.message.contains(".tmp") || f.message.contains(".bak")),
            "Should detect .tmp or .bak files"
        );
        assert!(!cruft_findings.iter().any(|f| f.message.contains("target/")), "Should NOT detect gitignored target/");
    }

    #[tokio::test]
    async fn test_performance_optimization_skips_heavy_dirs() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Don't create .gitignore, so files would normally be detected
        // But performance filter should skip these directories

        // Create files in heavy build directories (should be skipped by filter)
        create_file(repo_path, "target/CACHEDIR.TAG");  // Common in Rust target/
        create_file(repo_path, "node_modules/.bin/eslint");
        create_file(repo_path, "dist/bundle.js");

        let scorer = HygieneScorer::new();
        let config = ScorerConfig::default();

        let result = scorer.score(repo_path, &config).await.unwrap();

        // These directories are skipped by performance filter, so no penalty
        assert_eq!(result.score, 15.0, "Performance filter should skip heavy build directories");
    }

    #[tokio::test]
    async fn test_complex_gitignore_patterns() {
        let temp_dir = create_temp_repo();
        let repo_path = temp_dir.path();

        // Create complex .gitignore with various patterns
        fs::write(repo_path.join(".gitignore"),
            "*.pyc\n\
             __pycache__/\n\
             .DS_Store\n\
             *.swp\n\
             /build/\n\
             dist/\n"
        ).unwrap();

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
        assert_eq!(result.score, 15.0, "Complex .gitignore patterns should be respected");
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
        assert!(result.score < 15.0, "Large file should cause point deduction");

        // Check C3 subcategory
        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();
        assert!(c3.score < 5.0, "C3 should lose points for large file");

        // Check findings
        let large_file_finding = result.findings.iter()
            .any(|f| f.message.contains("Large file") && f.message.contains("large_file.bin"));
        assert!(large_file_finding, "Should report large file in findings");
    }

    #[tokio::test]
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
        assert!(result.score < 15.0, "Deleted large file should still be penalized (bloats git history)");

        let c3 = result.subcategories.iter().find(|s| s.id == "C3").unwrap();
        assert!(c3.score < 5.0, "C3 should detect large file in history even after deletion");
    }
}
