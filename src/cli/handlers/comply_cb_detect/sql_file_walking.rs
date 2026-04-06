// sql_file_walking.rs — File walking and SQL line utilities for CB-700 series
// Included by sql_best_practices.rs — do NOT add `use` imports or `#!` attrs here.

/// Walk directory recursively for `.sql` files.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn walkdir_sql_files(dir: &Path) -> Vec<PathBuf> {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let mut files = Vec::new();
    walk_sql_recursive(dir, &mut files);
    files
}

fn walk_sql_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_sql_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "sql" | "ddl" | "dml"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Check if a SQL file is a test file.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn is_sql_test_file(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    if stem.ends_with("_test") || stem.starts_with("test_") {
        return true;
    }
    path.components().any(|c| {
        let s = c.as_os_str().to_str().unwrap_or("");
        s == "tests" || s == "test" || s == "fixtures" || s == "testdata"
    })
}

/// Compute production lines (strip SQL comments).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn compute_sql_production_lines(content: &str) -> Vec<(usize, String)> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut result = Vec::new();
    let mut in_block_comment = false;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        if in_block_comment {
            if trimmed.contains("*/") {
                in_block_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("/*") {
            if trimmed.contains("*/") {
                continue;
            }
            in_block_comment = true;
            continue;
        }

        if trimmed.starts_with("--") || trimmed.is_empty() {
            continue;
        }

        // Strip inline comments
        let line_content = if let Some(pos) = trimmed.find("--") {
            trimmed[..pos].trim()
        } else {
            trimmed
        };

        if !line_content.is_empty() {
            result.push((i + 1, line_content.to_string()));
        }
    }

    result
}

fn collect_code_files(dir: &Path, extensions: &[&str], files: &mut Vec<PathBuf>) {
    debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
    for entry in fs::read_dir(dir).into_iter().flatten().flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        if path.is_dir() && !SKIP_DIRS.contains(&name) {
            collect_code_files(&path, extensions, files);
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| extensions.contains(&e))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

fn is_test_context(path: &Path) -> bool {
    debug_assert!(path.exists(), "path must exist: {}", path.display());
    let s = path.to_str().unwrap_or("");
    s.contains("test") || s.contains("spec") || s.contains("fixture")
}
