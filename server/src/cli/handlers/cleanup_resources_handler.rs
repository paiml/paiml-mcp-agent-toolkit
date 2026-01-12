// Cleanup resources CLI handler (GH-86)
// Toyota Way: Muda elimination - remove waste from development environments

use anyhow::Result;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use crate::cli::OutputFormat;

/// Cleanup target types
#[derive(Debug, Clone, PartialEq)]
pub enum CleanupTarget {
    Rust,
    Docker,
    Node,
    Git,
    Logs,
    Caches,
    All,
}

impl CleanupTarget {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "rust" => Some(Self::Rust),
            "docker" => Some(Self::Docker),
            "node" => Some(Self::Node),
            "git" => Some(Self::Git),
            "logs" => Some(Self::Logs),
            "caches" => Some(Self::Caches),
            "all" => Some(Self::All),
            _ => None,
        }
    }
}

/// Cleanup candidate found during scan
#[derive(Debug, Clone)]
pub struct CleanupCandidate {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub category: String,
    pub description: String,
    pub age_days: u32,
}

/// Cleanup result summary
#[derive(Debug, Default)]
pub struct CleanupResult {
    pub candidates: Vec<CleanupCandidate>,
    pub total_size_bytes: u64,
    pub items_found: usize,
    pub items_cleaned: usize,
    pub space_freed_bytes: u64,
    pub errors: Vec<String>,
}

/// Handle the `pmat maintain cleanup-resources` command
pub async fn handle_cleanup_resources(
    project_dir: &Path,
    targets: &[String],
    execute: bool,
    exclude: &[String],
    min_age_days: u32,
    format: OutputFormat,
) -> Result<()> {
    // Parse targets
    let parsed_targets: Vec<CleanupTarget> = targets
        .iter()
        .filter_map(|t| CleanupTarget::parse(t))
        .collect();

    if parsed_targets.is_empty() {
        println!("⚠️  No valid cleanup targets specified");
        println!("   Valid targets: rust, docker, node, git, logs, caches, all");
        return Ok(());
    }

    let has_all = parsed_targets.contains(&CleanupTarget::All);

    println!("🧹 PMAT Resource Cleanup");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!("📁 Scanning: {}", project_dir.display());
    println!("🎯 Targets: {:?}", targets);
    println!("⚡ Mode: {}", if execute { "EXECUTE" } else { "DRY-RUN" });
    println!();

    let mut result = CleanupResult::default();

    // Scan for cleanup candidates
    if has_all || parsed_targets.contains(&CleanupTarget::Rust) {
        scan_rust_targets(project_dir, exclude, min_age_days, &mut result)?;
    }

    if has_all || parsed_targets.contains(&CleanupTarget::Node) {
        scan_node_targets(project_dir, exclude, min_age_days, &mut result)?;
    }

    if has_all || parsed_targets.contains(&CleanupTarget::Git) {
        scan_git_targets(project_dir, &mut result)?;
    }

    if has_all || parsed_targets.contains(&CleanupTarget::Logs) {
        scan_log_targets(project_dir, exclude, min_age_days, &mut result)?;
    }

    // Print results
    print_results(&result, format)?;

    // Execute cleanup if requested
    if execute && !result.candidates.is_empty() {
        println!();
        println!("🔥 Executing cleanup...");
        execute_cleanup(&mut result)?;
        println!();
        println!(
            "✅ Cleaned {} items, freed {} MB",
            result.items_cleaned,
            result.space_freed_bytes / (1024 * 1024)
        );
    } else if !execute && !result.candidates.is_empty() {
        println!();
        println!("💡 Run with --execute to perform cleanup");
    }

    Ok(())
}

/// Scan for Rust target directories
fn scan_rust_targets(
    project_dir: &Path,
    exclude: &[String],
    _min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("🦀 Scanning Rust target directories...");

    for entry in WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()) && !is_excluded(e.path(), exclude))
        .flatten()
    {
        let path = entry.path();

        // Look for target directories with Cargo.toml sibling
        if path.is_dir() && path.file_name().is_some_and(|n| n == "target") {
            let parent = path.parent();
            if parent.is_some_and(|p| p.join("Cargo.toml").exists()) {
                let size = calculate_dir_size(path);
                result.candidates.push(CleanupCandidate {
                    path: path.to_path_buf(),
                    size_bytes: size,
                    category: "rust".to_string(),
                    description: "Rust build artifacts".to_string(),
                    age_days: 0,
                });
                result.total_size_bytes += size;
                result.items_found += 1;
            }
        }
    }

    println!(
        "   Found {} Rust target directories ({} MB)",
        result
            .candidates
            .iter()
            .filter(|c| c.category == "rust")
            .count(),
        result
            .candidates
            .iter()
            .filter(|c| c.category == "rust")
            .map(|c| c.size_bytes)
            .sum::<u64>()
            / (1024 * 1024)
    );

    Ok(())
}

/// Scan for Node.js node_modules directories
fn scan_node_targets(
    project_dir: &Path,
    exclude: &[String],
    _min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("📦 Scanning Node.js node_modules...");

    let mut node_count = 0;
    let mut node_size: u64 = 0;

    for entry in WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| {
            let path = e.path();
            !is_hidden(path)
                && !is_excluded(path, exclude)
                && path
                    .file_name()
                    .map(|n| n != "node_modules")
                    .unwrap_or(true)
        })
        .flatten()
    {
        let path = entry.path();

        // Look for node_modules directories with package.json sibling
        if path.is_dir() && path.file_name().is_some_and(|n| n == "node_modules") {
            let parent = path.parent();
            if parent.is_some_and(|p| p.join("package.json").exists()) {
                let size = calculate_dir_size(path);
                result.candidates.push(CleanupCandidate {
                    path: path.to_path_buf(),
                    size_bytes: size,
                    category: "node".to_string(),
                    description: "Node.js dependencies".to_string(),
                    age_days: 0,
                });
                result.total_size_bytes += size;
                result.items_found += 1;
                node_count += 1;
                node_size += size;
            }
        }
    }

    println!(
        "   Found {} node_modules directories ({} MB)",
        node_count,
        node_size / (1024 * 1024)
    );

    Ok(())
}

/// Scan for Git garbage collection opportunities
fn scan_git_targets(project_dir: &Path, result: &mut CleanupResult) -> Result<()> {
    println!("📚 Scanning Git repositories...");

    let git_dir = project_dir.join(".git");
    if git_dir.exists() {
        // Check loose objects
        let objects_dir = git_dir.join("objects");
        if objects_dir.exists() {
            let loose_count = count_loose_objects(&objects_dir);
            if loose_count > 100 {
                result.candidates.push(CleanupCandidate {
                    path: objects_dir.clone(),
                    size_bytes: 0, // Git gc will compact, not delete
                    category: "git".to_string(),
                    description: format!("{} loose objects (run git gc)", loose_count),
                    age_days: 0,
                });
                result.items_found += 1;
            }
        }
    }

    println!(
        "   Found {} Git optimization opportunities",
        result
            .candidates
            .iter()
            .filter(|c| c.category == "git")
            .count()
    );

    Ok(())
}

/// Scan for log files
fn scan_log_targets(
    project_dir: &Path,
    exclude: &[String],
    min_age_days: u32,
    result: &mut CleanupResult,
) -> Result<()> {
    println!("📝 Scanning log files...");

    let mut log_count = 0;
    let mut log_size: u64 = 0;

    for entry in WalkDir::new(project_dir)
        .max_depth(5)
        .into_iter()
        .filter_entry(|e| !is_hidden(e.path()) && !is_excluded(e.path(), exclude))
        .flatten()
    {
        let path = entry.path();

        if path.is_file() {
            let ext = path.extension().and_then(|e| e.to_str());
            if ext == Some("log") {
                // Check age if specified
                if min_age_days > 0 {
                    if let Ok(metadata) = path.metadata() {
                        if let Ok(modified) = metadata.modified() {
                            let age = modified.elapsed().unwrap_or_default();
                            let age_days = age.as_secs() / 86400;
                            if age_days < min_age_days as u64 {
                                continue;
                            }
                        }
                    }
                }

                let size = path.metadata().map(|m| m.len()).unwrap_or(0);
                result.candidates.push(CleanupCandidate {
                    path: path.to_path_buf(),
                    size_bytes: size,
                    category: "logs".to_string(),
                    description: "Log file".to_string(),
                    age_days: 0,
                });
                result.total_size_bytes += size;
                result.items_found += 1;
                log_count += 1;
                log_size += size;
            }
        }
    }

    println!(
        "   Found {} log files ({} MB)",
        log_count,
        log_size / (1024 * 1024)
    );

    Ok(())
}

/// Print cleanup results
fn print_results(result: &CleanupResult, format: OutputFormat) -> Result<()> {
    println!();
    println!("📊 Cleanup Summary");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");

    match format {
        OutputFormat::Json => {
            let json = serde_json::json!({
                "items_found": result.items_found,
                "total_size_mb": result.total_size_bytes / (1024 * 1024),
                "candidates": result.candidates.iter().map(|c| {
                    serde_json::json!({
                        "path": c.path.display().to_string(),
                        "size_mb": c.size_bytes / (1024 * 1024),
                        "category": c.category,
                        "description": c.description
                    })
                }).collect::<Vec<_>>()
            });
            println!("{}", serde_json::to_string_pretty(&json)?);
        }
        _ => {
            println!("   Items found:  {}", result.items_found);
            println!(
                "   Total size:   {} MB",
                result.total_size_bytes / (1024 * 1024)
            );
            println!();

            if !result.candidates.is_empty() {
                println!("📁 Candidates:");
                for candidate in result.candidates.iter().take(20) {
                    println!(
                        "   [{:6}] {:>8} MB  {}",
                        candidate.category,
                        candidate.size_bytes / (1024 * 1024),
                        candidate.path.display()
                    );
                }
                if result.candidates.len() > 20 {
                    println!("   ... and {} more", result.candidates.len() - 20);
                }
            }
        }
    }

    Ok(())
}

/// Execute cleanup operations
fn execute_cleanup(result: &mut CleanupResult) -> Result<()> {
    for candidate in &result.candidates {
        match candidate.category.as_str() {
            "rust" | "node" => {
                // Remove directory
                if candidate.path.is_dir() {
                    match std::fs::remove_dir_all(&candidate.path) {
                        Ok(_) => {
                            result.items_cleaned += 1;
                            result.space_freed_bytes += candidate.size_bytes;
                            println!("   ✓ Removed: {}", candidate.path.display());
                        }
                        Err(e) => {
                            result.errors.push(format!(
                                "Failed to remove {}: {}",
                                candidate.path.display(),
                                e
                            ));
                        }
                    }
                }
            }
            "git" => {
                // Run git gc
                let git_dir = candidate.path.parent().and_then(|p| p.parent());
                if let Some(repo_path) = git_dir {
                    let output = std::process::Command::new("git")
                        .args(["gc", "--aggressive"])
                        .current_dir(repo_path)
                        .output();

                    match output {
                        Ok(o) if o.status.success() => {
                            result.items_cleaned += 1;
                            println!("   ✓ Git gc: {}", repo_path.display());
                        }
                        Ok(o) => {
                            result.errors.push(format!(
                                "Git gc failed: {}",
                                String::from_utf8_lossy(&o.stderr)
                            ));
                        }
                        Err(e) => {
                            result.errors.push(format!("Git gc error: {}", e));
                        }
                    }
                }
            }
            "logs" => {
                // Remove log file
                if candidate.path.is_file() {
                    match std::fs::remove_file(&candidate.path) {
                        Ok(_) => {
                            result.items_cleaned += 1;
                            result.space_freed_bytes += candidate.size_bytes;
                            println!("   ✓ Removed: {}", candidate.path.display());
                        }
                        Err(e) => {
                            result.errors.push(format!(
                                "Failed to remove {}: {}",
                                candidate.path.display(),
                                e
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if !result.errors.is_empty() {
        println!();
        println!("⚠️  Errors:");
        for error in &result.errors {
            println!("   {}", error);
        }
    }

    Ok(())
}

// Helper functions

fn is_hidden(path: &Path) -> bool {
    path.file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.starts_with('.') && n != ".git")
        .unwrap_or(false)
}

fn is_excluded(path: &Path, exclude: &[String]) -> bool {
    let path_str = path.to_string_lossy();
    exclude.iter().any(|pattern| {
        if pattern.contains('*') {
            // Simple glob matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                path_str.starts_with(parts[0]) && path_str.ends_with(parts[1])
            } else {
                path_str.contains(pattern.trim_matches('*'))
            }
        } else {
            path_str.contains(pattern)
        }
    })
}

fn calculate_dir_size(path: &Path) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter_map(|e| e.metadata().ok())
        .map(|m| m.len())
        .sum()
}

fn count_loose_objects(objects_dir: &Path) -> usize {
    WalkDir::new(objects_dir)
        .max_depth(2)
        .into_iter()
        .flatten()
        .filter(|e| e.file_type().is_file())
        .filter(|e| {
            e.path()
                .parent()
                .and_then(|p| p.file_name())
                .and_then(|n| n.to_str())
                .map(|n| n.len() == 2 && n.chars().all(|c| c.is_ascii_hexdigit()))
                .unwrap_or(false)
        })
        .count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ============================================================================
    // CleanupTarget::parse tests
    // ============================================================================

    #[test]
    fn test_cleanup_target_parse_rust() {
        assert_eq!(CleanupTarget::parse("rust"), Some(CleanupTarget::Rust));
        assert_eq!(CleanupTarget::parse("RUST"), Some(CleanupTarget::Rust));
        assert_eq!(CleanupTarget::parse("Rust"), Some(CleanupTarget::Rust));
    }

    #[test]
    fn test_cleanup_target_parse_docker() {
        assert_eq!(CleanupTarget::parse("docker"), Some(CleanupTarget::Docker));
        assert_eq!(CleanupTarget::parse("DOCKER"), Some(CleanupTarget::Docker));
    }

    #[test]
    fn test_cleanup_target_parse_node() {
        assert_eq!(CleanupTarget::parse("node"), Some(CleanupTarget::Node));
        assert_eq!(CleanupTarget::parse("NODE"), Some(CleanupTarget::Node));
    }

    #[test]
    fn test_cleanup_target_parse_git() {
        assert_eq!(CleanupTarget::parse("git"), Some(CleanupTarget::Git));
        assert_eq!(CleanupTarget::parse("GIT"), Some(CleanupTarget::Git));
    }

    #[test]
    fn test_cleanup_target_parse_logs() {
        assert_eq!(CleanupTarget::parse("logs"), Some(CleanupTarget::Logs));
        assert_eq!(CleanupTarget::parse("LOGS"), Some(CleanupTarget::Logs));
    }

    #[test]
    fn test_cleanup_target_parse_caches() {
        assert_eq!(CleanupTarget::parse("caches"), Some(CleanupTarget::Caches));
        assert_eq!(CleanupTarget::parse("CACHES"), Some(CleanupTarget::Caches));
    }

    #[test]
    fn test_cleanup_target_parse_all() {
        assert_eq!(CleanupTarget::parse("all"), Some(CleanupTarget::All));
        assert_eq!(CleanupTarget::parse("ALL"), Some(CleanupTarget::All));
    }

    #[test]
    fn test_cleanup_target_parse_invalid() {
        assert_eq!(CleanupTarget::parse("invalid"), None);
        assert_eq!(CleanupTarget::parse(""), None);
        assert_eq!(CleanupTarget::parse("foo"), None);
    }

    // ============================================================================
    // is_hidden tests
    // ============================================================================

    #[test]
    fn test_is_hidden() {
        assert!(is_hidden(Path::new("/foo/.hidden")));
        assert!(!is_hidden(Path::new("/foo/.git"))); // .git is not hidden
        assert!(!is_hidden(Path::new("/foo/bar")));
    }

    #[test]
    fn test_is_hidden_dotfiles() {
        assert!(is_hidden(Path::new(".bashrc")));
        assert!(is_hidden(Path::new("/home/.profile")));
        assert!(is_hidden(Path::new(".env")));
    }

    #[test]
    fn test_is_hidden_normal_files() {
        assert!(!is_hidden(Path::new("foo.txt")));
        assert!(!is_hidden(Path::new("/path/to/file.rs")));
    }

    #[test]
    fn test_is_hidden_empty_path() {
        assert!(!is_hidden(Path::new("")));
    }

    // ============================================================================
    // is_excluded tests
    // ============================================================================

    #[test]
    fn test_is_excluded() {
        let exclude = vec!["node_modules".to_string(), "*.log".to_string()];
        assert!(is_excluded(Path::new("/foo/node_modules"), &exclude));
        assert!(is_excluded(Path::new("/foo/bar.log"), &exclude));
        assert!(!is_excluded(Path::new("/foo/bar"), &exclude));
    }

    #[test]
    fn test_is_excluded_empty_patterns() {
        let exclude: Vec<String> = vec![];
        assert!(!is_excluded(Path::new("/foo/bar"), &exclude));
    }

    #[test]
    fn test_is_excluded_glob_patterns() {
        let exclude = vec!["*.tmp".to_string()];
        assert!(is_excluded(Path::new("/foo/test.tmp"), &exclude));
        // The is_excluded function does simple matching, not full glob
        let exclude2 = vec!["build".to_string()];
        assert!(is_excluded(Path::new("/foo/build/output"), &exclude2));
    }

    #[test]
    fn test_is_excluded_partial_match() {
        let exclude = vec!["target".to_string()];
        assert!(is_excluded(Path::new("/project/target/debug"), &exclude));
        assert!(is_excluded(Path::new("/target/release"), &exclude));
    }

    // ============================================================================
    // calculate_dir_size tests
    // ============================================================================

    #[test]
    fn test_calculate_dir_size_empty() {
        let temp_dir = TempDir::new().unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 0);
    }

    #[test]
    fn test_calculate_dir_size_with_files() {
        let temp_dir = TempDir::new().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        std::fs::write(&file_path, "hello world").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert!(size > 0);
        assert_eq!(size, 11); // "hello world" is 11 bytes
    }

    #[test]
    fn test_calculate_dir_size_nested() {
        let temp_dir = TempDir::new().unwrap();
        let subdir = temp_dir.path().join("sub");
        std::fs::create_dir(&subdir).unwrap();
        std::fs::write(subdir.join("file.txt"), "content").unwrap();
        let size = calculate_dir_size(temp_dir.path());
        assert_eq!(size, 7); // "content" is 7 bytes
    }

    #[test]
    fn test_calculate_dir_size_nonexistent() {
        let size = calculate_dir_size(Path::new("/nonexistent/path"));
        assert_eq!(size, 0);
    }

    // ============================================================================
    // count_loose_objects tests
    // ============================================================================

    #[test]
    fn test_count_loose_objects_empty() {
        let temp_dir = TempDir::new().unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_loose_objects_nonexistent() {
        let count = count_loose_objects(Path::new("/nonexistent/path"));
        assert_eq!(count, 0);
    }

    #[test]
    fn test_count_loose_objects_with_hex_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // Create directories that look like git object dirs (2-char hex)
        let hex_dir = temp_dir.path().join("ab");
        std::fs::create_dir(&hex_dir).unwrap();
        std::fs::write(hex_dir.join("cdef1234"), "object content").unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 1);
    }

    #[test]
    fn test_count_loose_objects_non_hex_dirs() {
        let temp_dir = TempDir::new().unwrap();
        // Create directories that don't look like git object dirs
        let non_hex_dir = temp_dir.path().join("info");
        std::fs::create_dir(&non_hex_dir).unwrap();
        std::fs::write(non_hex_dir.join("packs"), "pack info").unwrap();
        let count = count_loose_objects(temp_dir.path());
        assert_eq!(count, 0); // "info" is not 2-char hex
    }

    // ============================================================================
    // CleanupCandidate tests
    // ============================================================================

    #[test]
    fn test_cleanup_candidate_clone() {
        let candidate = CleanupCandidate {
            path: PathBuf::from("/test/path"),
            size_bytes: 1024,
            category: "rust".to_string(),
            description: "Test candidate".to_string(),
            age_days: 5,
        };
        let cloned = candidate.clone();
        assert_eq!(cloned.path, candidate.path);
        assert_eq!(cloned.size_bytes, candidate.size_bytes);
        assert_eq!(cloned.category, candidate.category);
    }

    #[test]
    fn test_cleanup_candidate_debug() {
        let candidate = CleanupCandidate {
            path: PathBuf::from("/test/path"),
            size_bytes: 1024,
            category: "rust".to_string(),
            description: "Test candidate".to_string(),
            age_days: 5,
        };
        let debug = format!("{:?}", candidate);
        assert!(debug.contains("CleanupCandidate"));
        assert!(debug.contains("/test/path"));
    }

    // ============================================================================
    // CleanupResult tests
    // ============================================================================

    #[test]
    fn test_cleanup_result_default() {
        let result = CleanupResult::default();
        assert!(result.candidates.is_empty());
        assert_eq!(result.total_size_bytes, 0);
        assert_eq!(result.items_found, 0);
        assert_eq!(result.items_cleaned, 0);
        assert_eq!(result.space_freed_bytes, 0);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_cleanup_result_debug() {
        let result = CleanupResult::default();
        let debug = format!("{:?}", result);
        assert!(debug.contains("CleanupResult"));
    }

    // ============================================================================
    // CleanupTarget equality tests
    // ============================================================================

    #[test]
    fn test_cleanup_target_equality() {
        assert_eq!(CleanupTarget::Rust, CleanupTarget::Rust);
        assert_ne!(CleanupTarget::Rust, CleanupTarget::Docker);
    }

    #[test]
    fn test_cleanup_target_clone() {
        let target = CleanupTarget::Node;
        let cloned = target.clone();
        assert_eq!(target, cloned);
    }

    #[test]
    fn test_cleanup_target_debug() {
        let target = CleanupTarget::All;
        let debug = format!("{:?}", target);
        assert!(debug.contains("All"));
    }
}
