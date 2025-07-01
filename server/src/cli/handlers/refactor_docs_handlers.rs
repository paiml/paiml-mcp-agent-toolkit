//! Documentation refactoring handlers
//!
//! AI-assisted documentation cleanup that identifies and removes:
//! - Temporary files (fix-*.sh, test-*.md, etc.)
//! - Outdated status files (*_STATUS.md, *_PROGRESS.md)
//! - Build artifacts (*.mmd, optimization_state.json)
//! - Custom patterns defined by the user
//!
//! Follows Zero Tolerance Quality Standards from CLAUDE.md:
//! - No Temporary Code: All code is production-ready or it doesn't exist

use crate::cli::RefactorDocsOutputFormat;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::io::{self, AsyncBufReadExt, AsyncWriteExt, BufReader};

/// File category for classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileCategory {
    TemporaryScript,
    StatusReport,
    BuildArtifact,
    TestFixture,
    CustomPattern,
    Unknown,
}

impl std::fmt::Display for FileCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileCategory::TemporaryScript => write!(f, "Temporary Script"),
            FileCategory::StatusReport => write!(f, "Status Report"),
            FileCategory::BuildArtifact => write!(f, "Build Artifact"),
            FileCategory::TestFixture => write!(f, "Test Fixture"),
            FileCategory::CustomPattern => write!(f, "Custom Pattern"),
            FileCategory::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Information about a file identified for cleanup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CruftFile {
    pub path: PathBuf,
    pub category: FileCategory,
    pub size_bytes: u64,
    pub modified: SystemTime,
    pub age_days: u32,
    pub reason: String,
    pub pattern_matched: String,
}

/// Summary statistics for the cleanup operation
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CleanupSummary {
    pub total_files_scanned: usize,
    pub cruft_files_found: usize,
    pub total_size_bytes: u64,
    pub files_by_category: HashMap<String, usize>,
    pub size_by_category: HashMap<String, u64>,
    pub oldest_file_days: u32,
    pub newest_file_days: u32,
}

/// Result of the documentation refactoring analysis
#[derive(Debug, Serialize, Deserialize)]
pub struct RefactorDocsResult {
    pub cruft_files: Vec<CruftFile>,
    pub summary: CleanupSummary,
    pub preserved_files: Vec<PathBuf>,
    pub errors: Vec<String>,
}

/// Handle refactor docs command
#[allow(clippy::too_many_arguments)]
pub async fn handle_refactor_docs(
    project_path: PathBuf,
    include_docs: bool,
    include_root: bool,
    additional_dirs: Vec<PathBuf>,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    temp_patterns: Vec<String>,
    status_patterns: Vec<String>,
    artifact_patterns: Vec<String>,
    custom_patterns: Vec<String>,
    min_age_days: u32,
    max_size_mb: u64,
    recursive: bool,
    preserve_patterns: Vec<String>,
    output: Option<PathBuf>,
    auto_remove: bool,
    backup: bool,
    backup_dir: PathBuf,
    perf: bool,
) -> Result<()> {
    let start_time = std::time::Instant::now();

    // Collect all directories to scan
    let mut scan_dirs = Vec::new();
    
    if include_root {
        scan_dirs.push(project_path.clone());
    }
    
    if include_docs {
        let docs_dir = project_path.join("docs");
        if docs_dir.exists() {
            scan_dirs.push(docs_dir);
        }
    }
    
    scan_dirs.extend(additional_dirs);

    // Combine all patterns
    let mut all_patterns = Vec::new();
    all_patterns.extend(temp_patterns.iter().map(|p| (p.clone(), FileCategory::TemporaryScript)));
    all_patterns.extend(status_patterns.iter().map(|p| (p.clone(), FileCategory::StatusReport)));
    all_patterns.extend(artifact_patterns.iter().map(|p| (p.clone(), FileCategory::BuildArtifact)));
    all_patterns.extend(custom_patterns.iter().map(|p| (p.clone(), FileCategory::CustomPattern)));

    // Scan for cruft files
    let mut result = scan_for_cruft(
        &scan_dirs,
        &all_patterns,
        &preserve_patterns,
        min_age_days,
        max_size_mb * 1024 * 1024, // Convert MB to bytes
        recursive,
    ).await?;

    // Sort cruft files by size (largest first)
    result.cruft_files.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));

    // Handle interactive mode
    if format == RefactorDocsOutputFormat::Interactive && !dry_run && !auto_remove {
        result = handle_interactive_mode(result).await?;
    }

    // Create backup if requested
    if backup && !dry_run && (!result.cruft_files.is_empty() || auto_remove) {
        create_backup(&result.cruft_files, &backup_dir).await?;
    }

    // Remove files if not in dry run mode
    if !dry_run && (auto_remove || format == RefactorDocsOutputFormat::Interactive) {
        remove_files(&result.cruft_files).await?;
    }

    // Format and output results
    let output_content = format_output(&result, format, dry_run, perf, start_time.elapsed())?;

    if let Some(output_path) = output {
        tokio::fs::write(output_path, &output_content).await?;
    } else {
        println!("{}", output_content);
    }

    // Exit with appropriate code
    if !result.cruft_files.is_empty() && !auto_remove && !dry_run {
        std::process::exit(1); // Files found but not removed
    }

    Ok(())
}

/// Scan directories for cruft files
async fn scan_for_cruft(
    scan_dirs: &[PathBuf],
    patterns: &[(String, FileCategory)],
    preserve_patterns: &[String],
    min_age_days: u32,
    max_size_bytes: u64,
    recursive: bool,
) -> Result<RefactorDocsResult> {
    let mut cruft_files = Vec::new();
    let mut preserved_files = Vec::new();
    let mut errors = Vec::new();
    let mut total_files_scanned = 0;
    let mut summary = CleanupSummary::default();
    
    let now = SystemTime::now();

    for dir in scan_dirs {
        if !dir.exists() {
            errors.push(format!("Directory does not exist: {}", dir.display()));
            continue;
        }

        let files = if recursive {
            collect_files_recursive(dir).await?
        } else {
            collect_files_flat(dir).await?
        };

        total_files_scanned += files.len();

        for file_path in files {
            // Check if file should be preserved
            if should_preserve(&file_path, preserve_patterns) {
                preserved_files.push(file_path.clone());
                continue;
            }

            // Get file metadata
            let metadata = match fs::metadata(&file_path) {
                Ok(m) => m,
                Err(e) => {
                    errors.push(format!("Failed to read metadata for {}: {}", file_path.display(), e));
                    continue;
                }
            };

            // Skip if file is too large
            if metadata.len() > max_size_bytes {
                continue;
            }

            // Calculate age in days
            let age_days = match metadata.modified() {
                Ok(modified) => {
                    let duration = now.duration_since(modified).unwrap_or_default();
                    (duration.as_secs() / 86400) as u32
                }
                Err(_) => 0,
            };

            // Skip if file is too new
            if age_days < min_age_days {
                continue;
            }

            // Check against patterns
            if let Some((pattern, category)) = matches_pattern(&file_path, patterns) {
                let cruft = CruftFile {
                    path: file_path,
                    category,
                    size_bytes: metadata.len(),
                    modified: metadata.modified().unwrap_or(SystemTime::now()),
                    age_days,
                    reason: format!("Matches pattern: {}", pattern),
                    pattern_matched: pattern,
                };

                // Update summary
                let category_str = category.to_string();
                *summary.files_by_category.entry(category_str.clone()).or_default() += 1;
                *summary.size_by_category.entry(category_str).or_default() += metadata.len();
                summary.oldest_file_days = summary.oldest_file_days.max(age_days);
                summary.newest_file_days = if summary.newest_file_days == 0 {
                    age_days
                } else {
                    summary.newest_file_days.min(age_days)
                };

                cruft_files.push(cruft);
            }
        }
    }

    summary.total_files_scanned = total_files_scanned;
    summary.cruft_files_found = cruft_files.len();
    summary.total_size_bytes = cruft_files.iter().map(|f| f.size_bytes).sum();

    Ok(RefactorDocsResult {
        cruft_files,
        summary,
        preserved_files,
        errors,
    })
}

/// Collect files recursively
async fn collect_files_recursive(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut dirs_to_process = vec![dir.to_path_buf()];

    while let Some(current_dir) = dirs_to_process.pop() {
        let mut entries = tokio::fs::read_dir(&current_dir).await?;

        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            
            if path.is_dir() {
                dirs_to_process.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }

    Ok(files)
}

/// Collect files in a single directory (non-recursive)
async fn collect_files_flat(dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut entries = tokio::fs::read_dir(dir).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();
        if path.is_file() {
            files.push(path);
        }
    }

    Ok(files)
}

/// Check if a file should be preserved
fn should_preserve(path: &Path, preserve_patterns: &[String]) -> bool {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for pattern in preserve_patterns {
        if let Ok(pattern_glob) = glob::Pattern::new(pattern) {
            if pattern_glob.matches(file_name) {
                return true;
            }
        }
    }

    false
}

/// Check if a file matches any pattern
fn matches_pattern(path: &Path, patterns: &[(String, FileCategory)]) -> Option<(String, FileCategory)> {
    let file_name = path.file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");

    for (pattern, category) in patterns {
        if let Ok(pattern_glob) = glob::Pattern::new(pattern) {
            if pattern_glob.matches(file_name) {
                return Some((pattern.clone(), *category));
            }
        }
    }

    None
}

/// Handle interactive mode
async fn handle_interactive_mode(mut result: RefactorDocsResult) -> Result<RefactorDocsResult> {
    let mut stdin = BufReader::new(io::stdin());
    let mut stdout = io::stdout();
    let mut to_remove = Vec::new();

    println!("\n🔍 Found {} files for potential cleanup:\n", result.cruft_files.len());

    for (idx, file) in result.cruft_files.iter().enumerate() {
        println!("[{}] {} ({} bytes, {} days old)", 
            idx + 1,
            file.path.display(),
            file.size_bytes,
            file.age_days
        );
        println!("    Category: {}", file.category);
        println!("    Reason: {}", file.reason);
        
        stdout.write_all(b"\n    Remove this file? [y/N/a/q] ").await?;
        stdout.flush().await?;

        let mut response = String::new();
        stdin.read_line(&mut response).await?;
        
        match response.trim().to_lowercase().as_str() {
            "y" | "yes" => {
                to_remove.push(file.clone());
                println!("    ✓ Marked for removal");
            }
            "a" | "all" => {
                // Add all remaining files
                to_remove.extend(result.cruft_files[idx..].iter().cloned());
                println!("    ✓ Marked all remaining files for removal");
                break;
            }
            "q" | "quit" => {
                println!("    ✗ Cancelled");
                break;
            }
            _ => {
                println!("    ✗ Skipped");
            }
        }
    }

    result.cruft_files = to_remove;
    Ok(result)
}

/// Create backup of files
async fn create_backup(files: &[CruftFile], backup_dir: &Path) -> Result<()> {
    // Create backup directory with timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d_%H%M%S");
    let backup_path = backup_dir.join(format!("refactor_docs_{}", timestamp));
    
    tokio::fs::create_dir_all(&backup_path).await?;

    println!("📦 Creating backup in: {}", backup_path.display());

    for file in files {
        let relative_path = file.path.strip_prefix("/").unwrap_or(&file.path);
        let backup_file_path = backup_path.join(relative_path);
        
        if let Some(parent) = backup_file_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        tokio::fs::copy(&file.path, &backup_file_path).await
            .with_context(|| format!("Failed to backup {}", file.path.display()))?;
    }

    println!("✅ Backup created successfully");
    Ok(())
}

/// Remove files
async fn remove_files(files: &[CruftFile]) -> Result<()> {
    let mut removed = 0;
    let mut errors = Vec::new();

    for file in files {
        match tokio::fs::remove_file(&file.path).await {
            Ok(_) => {
                removed += 1;
            }
            Err(e) => {
                errors.push(format!("Failed to remove {}: {}", file.path.display(), e));
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("⚠️  Errors during removal:");
        for error in errors {
            eprintln!("  - {}", error);
        }
    }

    println!("🗑️  Removed {} files", removed);
    Ok(())
}

/// Format output based on format type
fn format_output(
    result: &RefactorDocsResult,
    format: RefactorDocsOutputFormat,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    match format {
        RefactorDocsOutputFormat::Summary => format_summary(result, dry_run, perf, elapsed),
        RefactorDocsOutputFormat::Detailed => format_detailed(result, dry_run, perf, elapsed),
        RefactorDocsOutputFormat::Json => format_json(result),
        RefactorDocsOutputFormat::Interactive => format_summary(result, dry_run, perf, elapsed),
    }
}

/// Format summary output
fn format_summary(
    result: &RefactorDocsResult,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = String::new();

    output.push_str("# Documentation Refactoring Report\n\n");
    
    if dry_run {
        output.push_str("**Mode**: Dry Run (no files will be removed)\n\n");
    }

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Files Scanned**: {}\n", result.summary.total_files_scanned));
    output.push_str(&format!("- **Cruft Files Found**: {}\n", result.summary.cruft_files_found));
    output.push_str(&format!("- **Total Size**: {:.2} MB\n", result.summary.total_size_bytes as f64 / 1_048_576.0));
    output.push_str(&format!("- **Oldest File**: {} days\n", result.summary.oldest_file_days));
    output.push_str(&format!("- **Newest File**: {} days\n\n", result.summary.newest_file_days));

    if !result.summary.files_by_category.is_empty() {
        output.push_str("## Files by Category\n\n");
        for (category, count) in &result.summary.files_by_category {
            let size = result.summary.size_by_category.get(category).unwrap_or(&0);
            output.push_str(&format!("- **{}**: {} files ({:.2} MB)\n", 
                category, count, *size as f64 / 1_048_576.0));
        }
        output.push('\n');
    }

    if !result.errors.is_empty() {
        output.push_str("## ⚠️ Errors\n\n");
        for error in &result.errors {
            output.push_str(&format!("- {}\n", error));
        }
        output.push('\n');
    }

    if perf {
        output.push_str(&format!("⏱️  Analysis completed in {:.2}s\n", elapsed.as_secs_f64()));
    }

    Ok(output)
}

/// Format detailed output
fn format_detailed(
    result: &RefactorDocsResult,
    dry_run: bool,
    perf: bool,
    elapsed: std::time::Duration,
) -> Result<String> {
    let mut output = format_summary(result, dry_run, perf, elapsed)?;

    if !result.cruft_files.is_empty() {
        output.push_str("## Cruft Files Details\n\n");
        
        for file in &result.cruft_files {
            let modified_date = DateTime::<Utc>::from(file.modified);
            output.push_str(&format!("### {}\n", file.path.display()));
            output.push_str(&format!("- **Category**: {}\n", file.category));
            output.push_str(&format!("- **Size**: {} bytes\n", file.size_bytes));
            output.push_str(&format!("- **Age**: {} days\n", file.age_days));
            output.push_str(&format!("- **Modified**: {}\n", modified_date.format("%Y-%m-%d %H:%M:%S")));
            output.push_str(&format!("- **Pattern**: {}\n", file.pattern_matched));
            output.push_str(&format!("- **Reason**: {}\n\n", file.reason));
        }
    }

    if !result.preserved_files.is_empty() && result.preserved_files.len() <= 20 {
        output.push_str("## Preserved Files\n\n");
        for file in &result.preserved_files {
            output.push_str(&format!("- {}\n", file.display()));
        }
        output.push('\n');
    }

    Ok(output)
}

/// Format JSON output
fn format_json(result: &RefactorDocsResult) -> Result<String> {
    serde_json::to_string_pretty(result).context("Failed to serialize to JSON")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_category_display() {
        assert_eq!(FileCategory::TemporaryScript.to_string(), "Temporary Script");
        assert_eq!(FileCategory::StatusReport.to_string(), "Status Report");
        assert_eq!(FileCategory::BuildArtifact.to_string(), "Build Artifact");
    }

    #[test]
    fn test_should_preserve() {
        let patterns = vec!["README.md".to_string(), "LICENSE*".to_string()];
        
        assert!(should_preserve(Path::new("README.md"), &patterns));
        assert!(should_preserve(Path::new("LICENSE"), &patterns));
        assert!(should_preserve(Path::new("LICENSE.txt"), &patterns));
        assert!(!should_preserve(Path::new("test.md"), &patterns));
    }

    #[test]
    fn test_matches_pattern() {
        let patterns = vec![
            ("fix-*.sh".to_string(), FileCategory::TemporaryScript),
            ("*_STATUS.md".to_string(), FileCategory::StatusReport),
        ];

        assert_eq!(
            matches_pattern(Path::new("fix-test.sh"), &patterns),
            Some(("fix-*.sh".to_string(), FileCategory::TemporaryScript))
        );
        
        assert_eq!(
            matches_pattern(Path::new("BUILD_STATUS.md"), &patterns),
            Some(("*_STATUS.md".to_string(), FileCategory::StatusReport))
        );
        
        assert_eq!(
            matches_pattern(Path::new("normal.txt"), &patterns),
            None
        );
    }
}