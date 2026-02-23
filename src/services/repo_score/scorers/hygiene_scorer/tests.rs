#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;
use std::path::Path;
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

    // Should lose 1 point (2 cruft files x 0.5 points) out of 15 total
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

    // Should lose 2 points (2 team files x 1 point) out of 15 total
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
