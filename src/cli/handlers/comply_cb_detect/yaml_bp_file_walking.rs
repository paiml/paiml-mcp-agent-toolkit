// CB-950 YAML best practices: file walking and production line utilities.
// Included by yaml_best_practices.rs — do NOT add `use` imports or `#!` attributes.

/// Walk directory recursively for `.yaml`/`.yml` files.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn walkdir_yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_yaml_recursive(dir, &mut files);
    files
}

fn walk_yaml_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            let dir_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            if !SKIP_DIRS.contains(&dir_name) {
                walk_yaml_recursive(&path, files);
            }
        } else if path
            .extension()
            .and_then(|e| e.to_str())
            .map(|e| matches!(e, "yaml" | "yml"))
            .unwrap_or(false)
        {
            files.push(path);
        }
    }
}

/// Compute production lines (strip YAML comments).
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "score_range")]
pub fn compute_yaml_production_lines(content: &str) -> Vec<(usize, String)> {
    let mut result = Vec::new();
    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        // Strip inline comments (but not inside quotes)
        let line_content = strip_yaml_inline_comment(trimmed);
        if !line_content.is_empty() {
            result.push((i + 1, line_content));
        }
    }
    result
}

fn strip_yaml_inline_comment(line: &str) -> String {
    let mut in_single = false;
    let mut in_double = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        match bytes[i] {
            b'\'' if !in_double => in_single = !in_single,
            b'"' if !in_single => in_double = !in_double,
            b'#' if !in_single && !in_double
                // Must be preceded by whitespace
                && i > 0 && bytes[i - 1] == b' ' => {
                    return line[..i].trim_end().to_string();
                }
            _ => {}
        }
    }
    line.to_string()
}
