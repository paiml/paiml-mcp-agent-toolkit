// CB-528 and CB-530: Numeric safety detectors
// Detects: division-by-length without empty guards (CB-528),
// and log function calls without clamp/max guards (CB-530).
// Includes helper functions for guard detection and expression analysis.

/// CB-528: Division-by-length Without Empty Guard
///
/// Detects `x / collection.len()` without preceding `is_empty()` or `len() > 0` guard.
/// In ML/numerical code, dividing by `len()` of an empty collection causes division-by-zero
/// (panic for integers, Inf/NaN for floats).
pub fn detect_cb528_division_by_length(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();
    // Match: `/ identifier.len()` or `/ identifier.len() as TYPE`
    // Also: `/ (identifier.len() ...)`
    let div_len_markers = [".len()", ".len() as "];

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            // Skip comments and string literals
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }

            // Check for division-by-len pattern
            let has_div_len = div_len_markers
                .iter()
                .any(|marker| has_division_by_len(trimmed, marker));
            if !has_div_len {
                continue;
            }

            // Check if already guarded: look back up to 8 lines for is_empty/len check
            // Also check if `.max(1)` is on the same line (wrapping the len)
            if has_len_guard(&lines, i) {
                continue;
            }

            violations.push(CbPatternViolation {
                pattern_id: "CB-528".to_string(),
                file: file.clone(),
                line: i + 1,
                description:
                    "Division by .len() without empty collection guard (is_empty/len>0/.max(1))"
                        .to_string(),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// Check if a line contains `/ something.len()` pattern
fn has_division_by_len(line: &str, marker: &str) -> bool {
    let mut search_from = 0;
    while let Some(pos) = line[search_from..].find(marker) {
        let abs_pos = search_from + pos;
        if check_division_before(line, abs_pos) {
            return true;
        }
        search_from = abs_pos + marker.len();
    }
    false
}

/// Check if there's a `/` division operator before position `pos` in the line
fn check_division_before(line: &str, pos: usize) -> bool {
    let before = &line[..pos];
    let before_trimmed = before.trim_end();
    // Direct `/ expr.len()` — slash immediately before the expression
    if before_trimmed.ends_with('/') && !before_trimmed.ends_with("//") {
        return true;
    }
    // `/ expr.len()` with space — e.g. `sum / items.len()`
    let Some(slash_pos) = before.rfind("/ ") else {
        return false;
    };
    let pre_slash = &before[..slash_pos];
    // Ensure it's division, not a comment (`// ...`)
    if pre_slash.trim_end().ends_with('/') {
        return false;
    }
    // Check there's no other arithmetic operator between `/` and `.len()`
    let between = &before[slash_pos + 2..];
    !between.contains('+')
        && !between.contains('-')
        && !between.contains('*')
        && !between.contains('/')
}

/// Check if the surrounding context has a guard against empty len
fn has_len_guard(lines: &[&str], line_idx: usize) -> bool {
    let lookback = 8;
    let start = line_idx.saturating_sub(lookback);

    // Check current line for .max(1) wrapping the len
    let current = lines[line_idx];
    if current.contains(".len().max(1)")
        || current.contains(".len()).max(1)")
        || current.contains(".max(1)")
    {
        return true;
    }

    // Check surrounding context for guard patterns
    for line in &lines[start..=line_idx] {
        let t = line.trim();
        if t.contains("is_empty()")
            || t.contains(".len() > 0")
            || t.contains(".len() >= 1")
            || t.contains(".len() != 0")
            || t.contains(".len() == 0")
            || t.contains("!.is_empty()")
        {
            return true;
        }
    }

    false
}

/// CB-530: Log Without Clamp Guard
///
/// Detects `.ln()`, `.log2()`, `.log10()` calls without preceding `.max(epsilon)` or `.clamp()`.
/// Passing zero or negative values to log functions produces -Inf or NaN, which silently
/// corrupts ML training losses, probability calculations, and information-theoretic metrics.
pub fn detect_cb530_log_without_clamp(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let src_dir = project_path.join("src");
    let entries = match walkdir_rs_files(&src_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    let mut violations = Vec::new();
    let log_fns = [".ln()", ".log2()", ".log10()"];

    for entry in &entries {
        if is_test_file(entry) {
            continue;
        }
        let content = match fs::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let lines: Vec<&str> = content.lines().collect();
        let test_lines = compute_test_code_lines(&lines);
        let file = entry
            .strip_prefix(project_path)
            .unwrap_or(entry)
            .display()
            .to_string();

        for (i, line) in lines.iter().enumerate() {
            if test_lines.contains(&i) {
                continue;
            }
            let trimmed = line.trim();

            // Skip comments
            if trimmed.starts_with("//") || trimmed.starts_with("///") {
                continue;
            }

            // Check for log function calls
            let log_fn = match log_fns.iter().find(|&&f| trimmed.contains(f)) {
                Some(f) => *f,
                None => continue,
            };

            // Skip if it's inside a string literal
            if is_log_in_string(trimmed, log_fn) {
                continue;
            }

            // Check if guarded: `.max(eps).ln()` or `.clamp(...).ln()` on same line
            if has_log_guard(trimmed, log_fn) {
                continue;
            }

            // Check if the expression is a known-positive constant like `2.0_f64.ln()`
            if is_positive_literal_log(trimmed, log_fn) {
                continue;
            }

            // Check preceding line for guard (e.g. `let x = val.max(1e-10);` then `x.ln()`)
            if i > 0 && has_log_guard_context(&lines, i) {
                continue;
            }

            violations.push(CbPatternViolation {
                pattern_id: "CB-530".to_string(),
                file: file.clone(),
                line: i + 1,
                description: format!(
                    "{} without .max(epsilon) or .clamp() guard — risk of -Inf/NaN",
                    log_fn.trim_start_matches('.')
                ),
                severity: Severity::Warning,
            });
        }
    }

    violations
}

/// Check if the log call is guarded by .max() or .clamp() on the same expression
fn has_log_guard(line: &str, log_fn: &str) -> bool {
    // Find the log call position
    if let Some(pos) = line.find(log_fn) {
        let before = &line[..pos];
        // Check for `.max(` or `.clamp(` before the log call
        // The guard must be on the same expression (no semicolons between)
        let last_stmt = before.rfind(';').map_or(before, |p| &before[p + 1..]);
        if last_stmt.contains(".max(") || last_stmt.contains(".clamp(") {
            return true;
        }
        // Also check for `(1.0 + x)` pattern — log of sum with positive constant
        if last_stmt.contains("(1.0 +") || last_stmt.contains("(1.0+") {
            return true;
        }
    }
    false
}

/// Check if the log is applied to a known positive literal like `2.0_f64.ln()`
fn is_positive_literal_log(line: &str, log_fn: &str) -> bool {
    if let Some(pos) = line.find(log_fn) {
        let before = &line[..pos];
        let expr = before.trim_end();
        // Check for numeric literal ending: `2.0.ln()`, `2.0_f64.ln()`, etc.
        let expr_end = expr.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_' || c == '.');
        let suffix = &expr[expr_end.len()..];
        // Try to parse as a positive number
        let num_str = suffix
            .trim_end_matches("_f32")
            .trim_end_matches("_f64")
            .trim_end_matches("f32")
            .trim_end_matches("f64");
        if let Ok(val) = num_str.parse::<f64>() {
            if val > 0.0 {
                return true;
            }
        }
    }
    false
}

/// Check if the log call appears inside a string literal
fn is_log_in_string(line: &str, log_fn: &str) -> bool {
    if let Some(pos) = line.find(log_fn) {
        let before = &line[..pos];
        // Count unescaped quotes before the log call
        let quote_count = before.chars().filter(|&c| c == '"').count();
        // Odd number means we're inside a string
        quote_count % 2 != 0
    } else {
        false
    }
}

/// Check preceding lines for variable-level guards
fn has_log_guard_context(lines: &[&str], line_idx: usize) -> bool {
    let lookback = 3;
    let start = line_idx.saturating_sub(lookback);
    for line in &lines[start..line_idx] {
        let t = line.trim();
        // Pattern: `let x = expr.max(eps);` or `let x = expr.clamp(low, high);`
        if (t.contains(".max(") || t.contains(".clamp("))
            && (t.contains("1e-") || t.contains("f32::EPSILON") || t.contains("f64::EPSILON"))
        {
            return true;
        }
    }
    false
}
