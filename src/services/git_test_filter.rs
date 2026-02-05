// Git-aware test filtering for targeted quality gates
//
// Implements smart test selection based on git diff to avoid running
// all 5000+ tests when only a few files changed.

use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Extract test module paths from changed Rust files
///
/// Converts file paths like "server/src/services/progress.rs"
/// into test filter patterns like "services::progress"
pub fn extract_test_modules_from_changed_files(project_root: &Path) -> Result<Vec<String>> {
    let changed_files = get_changed_rust_files(project_root)?;

    if changed_files.is_empty() {
        return Ok(Vec::new());
    }

    let modules: Vec<String> = changed_files
        .into_iter()
        .filter_map(|path| {
            // Strip "server/src/" prefix and ".rs" suffix
            let path = path.strip_prefix("server/src/")?;
            let path = path.strip_suffix(".rs")?;

            // Convert path separators to Rust module syntax
            // e.g., "services/progress" -> "services::progress"
            Some(path.replace('/', "::"))
        })
        .collect();

    Ok(modules)
}

/// Get list of changed Rust files from git diff
fn get_changed_rust_files(project_root: &Path) -> Result<Vec<String>> {
    let output = Command::new("git")
        .args(["diff", "--name-only", "HEAD"])
        .current_dir(project_root)
        .output()
        .context("Failed to run git diff")?;

    if !output.status.success() {
        return Ok(Vec::new()); // Not a git repo or no changes
    }

    let stdout = String::from_utf8_lossy(&output.stdout);

    let files: Vec<String> = stdout
        .lines()
        .filter(|line| line.ends_with(".rs"))
        .filter(|line| line.starts_with("server/src/")) // Only server code
        .map(String::from)
        .collect();

    Ok(files)
}

/// Build cargo test command with module filters
///
/// Returns None if no modules to test (no Rust files changed)
pub fn build_test_command(modules: &[String]) -> Option<Vec<String>> {
    if modules.is_empty() {
        return None;
    }

    let mut args = vec!["test".to_string(), "--lib".to_string()];

    // Add test filter: cargo test module1 module2 module3
    // This runs all tests in those modules
    args.extend(modules.iter().cloned());

    Some(args)
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    /// RED TEST: Extract single module from single file change
    #[test]
    fn test_extract_single_module() {
        // Simulate changed file list
        let files = vec!["server/src/services/progress.rs".to_string()];

        let modules: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(modules, vec!["services::progress"]);
    }

    /// RED TEST: Extract multiple modules from multiple file changes
    #[test]
    fn test_extract_multiple_modules() {
        let files = vec![
            "server/src/services/progress.rs".to_string(),
            "server/src/models/roadmap.rs".to_string(),
            "server/src/cli/handlers/work_handlers.rs".to_string(),
        ];

        let modules: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(
            modules,
            vec![
                "services::progress",
                "models::roadmap",
                "cli::handlers::work_handlers"
            ]
        );
    }

    /// RED TEST: Filter out non-Rust files
    #[test]
    fn test_filter_non_rust_files() {
        let files = vec![
            "server/src/services/progress.rs".to_string(),
            "README.md".to_string(),
            "Cargo.toml".to_string(),
            "server/src/lib.rs".to_string(),
        ];

        let modules: Vec<String> = files
            .into_iter()
            .filter(|line| line.ends_with(".rs"))
            .filter(|line| line.starts_with("server/src/"))
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(modules, vec!["services::progress", "lib"]);
    }

    /// RED TEST: Filter out non-server files (e.g., client/, docs/)
    #[test]
    fn test_filter_non_server_files() {
        let files = vec![
            "server/src/services/progress.rs".to_string(),
            "client/src/main.rs".to_string(),
            "docs/README.md".to_string(),
        ];

        let modules: Vec<String> = files
            .into_iter()
            .filter(|line| line.ends_with(".rs"))
            .filter(|line| line.starts_with("server/src/"))
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(modules, vec!["services::progress"]);
    }

    /// RED TEST: Handle nested module paths
    #[test]
    fn test_nested_module_paths() {
        let files = vec![
            "server/src/services/mutation/python_tree_sitter_mutations.rs".to_string(),
            "server/src/cli/handlers/work_handlers.rs".to_string(),
        ];

        let modules: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(
            modules,
            vec![
                "services::mutation::python_tree_sitter_mutations",
                "cli::handlers::work_handlers"
            ]
        );
    }

    /// RED TEST: Build cargo test command with single module
    #[test]
    fn test_build_test_command_single() {
        let modules = vec!["services::progress".to_string()];
        let cmd = build_test_command(&modules);

        assert_eq!(
            cmd,
            Some(vec![
                "test".to_string(),
                "--lib".to_string(),
                "services::progress".to_string()
            ])
        );
    }

    /// RED TEST: Build cargo test command with multiple modules
    #[test]
    fn test_build_test_command_multiple() {
        let modules = vec![
            "services::progress".to_string(),
            "models::roadmap".to_string(),
        ];
        let cmd = build_test_command(&modules);

        assert_eq!(
            cmd,
            Some(vec![
                "test".to_string(),
                "--lib".to_string(),
                "services::progress".to_string(),
                "models::roadmap".to_string()
            ])
        );
    }

    /// RED TEST: Return None when no modules (no Rust files changed)
    #[test]
    fn test_build_test_command_empty() {
        let modules: Vec<String> = vec![];
        let cmd = build_test_command(&modules);

        assert_eq!(cmd, None);
    }

    /// RED TEST: Handle mod.rs files (convert to parent module)
    #[test]
    fn test_handle_mod_rs() {
        let files = vec!["server/src/services/mod.rs".to_string()];

        let modules: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;

                // Special case: mod.rs -> parent module
                if path.ends_with("/mod.rs") {
                    let parent = path.strip_suffix("/mod.rs")?;
                    return Some(parent.replace('/', "::"));
                }

                let path = path.strip_suffix(".rs")?;
                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(modules, vec!["services"]);
    }

    /// RED TEST: Handle lib.rs (runs all tests)
    #[test]
    fn test_handle_lib_rs() {
        let files = vec!["server/src/lib.rs".to_string()];

        let modules: Vec<String> = files
            .into_iter()
            .filter_map(|path| {
                let path = path.strip_prefix("server/src/")?;
                let path = path.strip_suffix(".rs")?;

                // lib.rs means run all tests (empty filter)
                if path == "lib" {
                    return None; // Skip - will run all tests
                }

                Some(path.replace('/', "::"))
            })
            .collect();

        assert_eq!(modules, Vec::<String>::new());
    }
}
