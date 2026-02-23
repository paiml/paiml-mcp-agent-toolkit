#![cfg_attr(coverage_nightly, coverage(off))]

use super::*;
use std::fs;
use std::path::Path;
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
