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
// Default and Trait Implementation Tests
// =========================================================================

#[test]
fn test_hygiene_scorer_default() {
    let scorer = HygieneScorer;
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
