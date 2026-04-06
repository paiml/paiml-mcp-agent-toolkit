// CB-803: Return Statement
// CB-804: var Declaration
// CB-805: Blocking in Future
//
// Included from scala_best_practices.rs — no `use` imports or `#!` attributes.

// =============================================================================
// CB-803: Return Statement
// =============================================================================

pub fn detect_cb803_return_statement(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if contains_return_keyword(line) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-803".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Explicit `return` — anti-idiomatic in Scala".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Check for `return` as a standalone keyword, not inside string literals or comments.
fn contains_return_keyword(line: &str) -> bool {
    let bytes = line.as_bytes();
    let ret = b"return";
    let len = bytes.len();
    if len < 6 {
        return false;
    }
    for i in 0..=len - 6 {
        if &bytes[i..i + 6] == ret {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after_ok =
                i + 6 >= len || !bytes[i + 6].is_ascii_alphanumeric() && bytes[i + 6] != b'_';
            if before_ok && after_ok {
                // Skip if inside a string literal (simple heuristic)
                let before_text = &line[..i];
                let double_quotes = before_text.chars().filter(|&c| c == '"').count();
                if double_quotes % 2 == 0 {
                    return true;
                }
            }
        }
    }
    false
}

// =============================================================================
// CB-804: var Declaration
// =============================================================================

pub fn detect_cb804_var_declaration(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if contains_var_keyword(line) {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-804".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "`var` declaration — prefer `val` for immutability".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// Check for `var` as a standalone keyword at declaration position.
fn contains_var_keyword(line: &str) -> bool {
    // Match lines starting with var or containing var after whitespace/keywords
    let trimmed = line.trim();
    if trimmed.starts_with("var ") {
        return true;
    }
    // Also catch: `private var`, `protected var`, `override var`, etc.
    let modifiers = [
        "private var ",
        "protected var ",
        "override var ",
        "lazy var ",
    ];
    modifiers.iter().any(|m| trimmed.contains(m))
}

// =============================================================================
// CB-805: Blocking in Future
// =============================================================================

/// Blocking calls that should not appear inside Future blocks.
const BLOCKING_CALLS: &[&str] = &[
    "Thread.sleep",
    "Await.result",
    "Await.ready",
    ".wait()",
    "synchronized",
];

pub fn detect_cb805_blocking_in_future(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        detect_blocking_in_future_content(&content, &rel, &mut violations);
    }

    violations
}

fn detect_blocking_in_future_content(
    content: &str,
    rel: &str,
    violations: &mut Vec<CbPatternViolation>,
) {
    let mut in_future_block = false;
    let mut brace_depth: i32 = 0;
    let mut future_start_depth: i32 = 0;

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();

        // Skip comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with("*") {
            continue;
        }

        // Detect Future block start
        if (trimmed.contains("Future {") || trimmed.contains("Future.apply {")) && !in_future_block
        {
            in_future_block = true;
            future_start_depth = brace_depth;
        }

        // Track brace depth
        for c in trimmed.chars() {
            match c {
                '{' => brace_depth += 1,
                '}' => {
                    brace_depth -= 1;
                    if in_future_block && brace_depth <= future_start_depth {
                        in_future_block = false;
                    }
                }
                _ => {}
            }
        }

        // Check for blocking calls inside Future
        if in_future_block {
            for blocking in BLOCKING_CALLS {
                if trimmed.contains(blocking) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-805".to_string(),
                        file: rel.to_string(),
                        line: i + 1,
                        description: format!(
                            "Blocking call `{}` inside Future — use non-blocking alternative",
                            blocking
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}
