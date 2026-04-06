// analyzer_simple_helpers.rs — Estimation and file discovery helpers for TdgAnalyzer
// Included by analyzer_simple.rs — shares parent module scope

impl TdgAnalyzer {
    fn estimate_cyclomatic_complexity(&self, lines: &[&str]) -> u32 {
        let mut complexity = 1;

        for line in lines {
            let trimmed = line.trim();
            complexity += count_control_flow_keywords(trimmed);
            complexity += count_logical_operators(trimmed);
        }

        complexity
    }

    fn estimate_nesting_depth(&self, source: &str) -> usize {
        let mut max_depth = 0;
        let mut current_depth = 0;

        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed.contains('{') {
                current_depth += trimmed.matches('{').count();
                max_depth = max_depth.max(current_depth);
            }
            if trimmed.contains('}') {
                current_depth = current_depth.saturating_sub(trimmed.matches('}').count());
            }
        }

        max_depth
    }

    fn estimate_duplication_ratio(&self, source: &str) -> f32 {
        let lines: Vec<&str> = source
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty() && !l.starts_with("//") && !l.starts_with("/*"))
            .collect();

        if lines.len() < 3 {
            return 0.0;
        }

        let mut duplicates = 0;
        for i in 0..lines.len() {
            for j in i + 1..lines.len() {
                if lines[i] == lines[j] && lines[i].len() > 10 {
                    duplicates += 1;
                }
            }
        }

        duplicates as f32 / lines.len() as f32
    }

    fn discover_files(&self, dir: &Path) -> Result<Vec<PathBuf>> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        let mut files = Vec::new();
        self.discover_files_recursive(dir, &mut files)?;
        Ok(files)
    }

    fn discover_files_recursive(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        debug_assert!(dir.exists(), "dir must exist: {}", dir.display());
        if !dir.is_dir() {
            return Ok(());
        }

        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                if !self.should_skip_directory(&path) {
                    self.discover_files_recursive(&path, files)?;
                }
            } else if self.should_analyze_file(&path) {
                files.push(path);
            }
        }

        Ok(())
    }

    fn should_skip_directory(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            matches!(
                name,
                "node_modules"
                    | "target"
                    | "build"
                    | "dist"
                    | ".git"
                    | "__pycache__"
                    | ".pytest_cache"
                    | "venv"
                    | ".venv"
                    | "vendor"
                    | ".idea"
                    | ".vscode"
                    | ".lake"
            )
        } else {
            false
        }
    }

    fn should_analyze_file(&self, path: &Path) -> bool {
        debug_assert!(path.exists(), "path must exist: {}", path.display());
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            matches!(
                ext,
                "rs" | "py"
                    | "js"
                    | "ts"
                    | "jsx"
                    | "tsx"
                    | "go"
                    | "java"
                    | "c"
                    | "h"
                    | "cpp"
                    | "cc"
                    | "cxx"
                    | "hpp"
                    | "rb"
                    | "swift"
                    | "kt"
                    | "kts"
                    | "lean"
            )
        } else {
            false
        }
    }
}

/// Count `sorry` occurrences in Lean source (proof incompleteness).
/// Skips line comments (`--`) and nested block comments (`/- ... -/`).
/// Uses word-boundary checking to avoid false positives from identifiers.
fn count_lean_sorry(source: &str) -> usize {
    let mut count = 0;
    let mut in_block_comment: i32 = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        // Skip line comments
        if trimmed.starts_with("--") {
            continue;
        }

        // Strip block comments
        let cleaned = strip_lean_block_comments(trimmed, &mut in_block_comment);

        // If still inside a block comment, skip
        if in_block_comment > 0 {
            continue;
        }

        // Word-boundary check: sorry must be a standalone word
        if contains_lean_sorry_word(&cleaned) {
            count += 1;
        }
    }

    count
}

/// Strips Lean block comment content (`/- ... -/`) from a line.
fn strip_lean_block_comments(line: &str, depth: &mut i32) -> String {
    let bytes = line.as_bytes();
    let mut result = String::with_capacity(line.len());
    let mut i = 0;

    while i < bytes.len() {
        if i + 1 < bytes.len() && bytes[i] == b'/' && bytes[i + 1] == b'-' {
            *depth += 1;
            i += 2;
            continue;
        }
        if i + 1 < bytes.len() && bytes[i] == b'-' && bytes[i + 1] == b'/' && *depth > 0 {
            *depth -= 1;
            i += 2;
            continue;
        }
        if *depth == 0 {
            result.push(bytes[i] as char);
        }
        i += 1;
    }

    result
}

/// Checks if a line contains "sorry" as a standalone word.
fn contains_lean_sorry_word(line: &str) -> bool {
    let bytes = line.as_bytes();
    let sorry = b"sorry";

    let mut pos = 0;
    while pos + sorry.len() <= bytes.len() {
        if let Some(idx) = line[pos..].find("sorry") {
            let abs_idx = pos + idx;
            let before_ok =
                abs_idx == 0 || !bytes[abs_idx - 1].is_ascii_alphanumeric() && bytes[abs_idx - 1] != b'_';
            let after_ok = abs_idx + sorry.len() >= bytes.len()
                || !bytes[abs_idx + sorry.len()].is_ascii_alphanumeric()
                    && bytes[abs_idx + sorry.len()] != b'_';
            if before_ok && after_ok {
                return true;
            }
            pos = abs_idx + 1;
        } else {
            break;
        }
    }
    false
}

/// Count control flow keywords in a single trimmed line.
fn count_control_flow_keywords(trimmed: &str) -> u32 {
    let mut count = 0;
    if trimmed.starts_with("if ") || trimmed.contains(" if ") {
        count += 1;
    }
    if trimmed.starts_with("for ") || trimmed.contains(" for ") {
        count += 1;
    }
    if trimmed.starts_with("while ") || trimmed.contains(" while ") {
        count += 1;
    }
    if trimmed.starts_with("match ") || trimmed.contains(" match ") {
        count += 1;
    }
    count
}

/// Count logical operators (&& and ||) in a single trimmed line.
fn count_logical_operators(trimmed: &str) -> u32 {
    if trimmed.contains(" && ") || trimmed.contains(" || ") {
        trimmed.matches(" && ").count() as u32 + trimmed.matches(" || ").count() as u32
    } else {
        0
    }
}
