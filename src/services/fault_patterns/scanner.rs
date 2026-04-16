//! Line-level pattern scanner (PMAT-613).
//!
//! Ported from batuta/src/bug_hunter/patterns.rs. Classifies whether a literal
//! pattern match in a line represents real debt or a false positive (inside a
//! string literal, doc comment, test-assertion context, etc.).
//!
//! Pure functions — no I/O, no subprocess. Safe to call per-line during index
//! build.

use super::types::{DefectCategory, Finding, PatternRule};
use std::collections::HashSet;

/// Determine which 1-indexed lines fall inside `#[cfg(test)]` or `#[test]` blocks.
///
/// Naive brace-balancing: tracks depth after `#[cfg(test)] mod { ... }` or
/// `#[test] fn { ... }`. Good enough for standard Rust layouts; imperfect on
/// strings/chars containing braces, matching the original batuta heuristic.
pub fn compute_test_lines(content: &str) -> HashSet<usize> {
    let mut test_lines = HashSet::new();
    let mut in_test_module = false;
    let mut test_module_start_depth: i32 = 0;
    let mut brace_depth: i32 = 0;
    let mut waiting_for_brace = false;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        let trimmed = line.trim();
        let open = line.matches('{').count() as i32;
        let close = line.matches('}').count() as i32;

        if trimmed == "#[cfg(test)]" {
            waiting_for_brace = true;
            test_lines.insert(line_num);
        }
        if trimmed == "#[test]" || trimmed.starts_with("#[test]") {
            waiting_for_brace = true;
            test_lines.insert(line_num);
        }

        if waiting_for_brace && open > 0 {
            in_test_module = true;
            test_module_start_depth = brace_depth;
            waiting_for_brace = false;
        }

        brace_depth += open - close;

        if in_test_module {
            test_lines.insert(line_num);
            if brace_depth <= test_module_start_depth {
                in_test_module = false;
            }
        }
    }
    test_lines
}

/// SATD markers (TODO/FIXME/HACK/XXX) count as real only when inside a
/// single-line comment and not within a string. Multiple markers on one line
/// indicate meta-commentary about patterns — also skipped.
fn check_tech_debt_real(line: &str, before: &str, trimmed: &str) -> bool {
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    let marker_count = ["TODO", "FIXME", "HACK", "XXX"]
        .iter()
        .filter(|m| line.contains(*m))
        .count();
    if marker_count >= 2 {
        return false;
    }
    let has_comment = before.contains("//") || before.contains("/*");
    let in_string = before.matches('"').count() % 2 == 1;
    let prev = before.chars().last();
    let boundary_ok = matches!(prev, Some(' ' | '\t' | '/' | '*') | None);
    has_comment && !in_string && boundary_ok
}

fn check_comment_pattern_real(line: &str, before: &str, trimmed: &str) -> bool {
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    let in_string = before.matches('"').count() % 2 == 1;
    let lower = line.to_lowercase();
    if lower.contains("debug:")
        || lower.contains("for debugging")
        || lower.contains("diagnostic")
        || lower.contains("returns cuda_error")
        || lower.contains("fix:")
        || lower.contains("via ")
        || lower.contains("sentinel")
        || lower.contains("recreates")
    {
        return false;
    }
    trimmed.starts_with("//") && !in_string
}

fn is_not_implemented_test_context(lower: &str) -> bool {
    lower.contains("assert")
        || lower.contains("expect")
        || lower.contains("returns error")
        || lower.contains("should fail")
        || lower.contains("should panic")
        || lower.contains("test_")
        || lower.contains("_test")
        || lower.contains("is_err")
}

fn is_not_implemented_in_string(line: &str, trimmed: &str) -> bool {
    let end = trimmed.trim_end();
    end.ends_with("\",") || end.ends_with('"') || line.contains("{}") || line.contains("{:")
}

fn is_not_implemented_benign_comment(lower: &str, trimmed: &str) -> bool {
    if !trimmed.starts_with("//") {
        return false;
    }
    lower.contains("fails")
        || lower.contains("error")
        || lower.contains("but not implemented")
        || trimmed.len() < 50
}

fn check_unimplemented_exclusions(line: &str, trimmed: &str) -> bool {
    let lower = line.to_lowercase();
    if lower.contains("does not support")
        || lower.contains("not supported")
        || lower.contains("use minimize")
        || lower.contains("by design")
    {
        return true;
    }
    let tl = trimmed.to_lowercase();
    if tl == "unimplemented!(" || (tl.starts_with("unimplemented!(") && !tl.contains(')')) {
        return true;
    }
    lower.contains("_unimplemented")
        || lower.contains("should_panic")
        || lower.contains("// test unimplemented")
}

fn check_not_implemented_exclusions(line: &str, trimmed: &str) -> bool {
    let lower = line.to_lowercase();
    is_not_implemented_test_context(&lower)
        || is_not_implemented_in_string(line, trimmed)
        || is_not_implemented_benign_comment(&lower, trimmed)
}

fn is_mid_identifier_euphemism(pattern: &str, before: &str) -> bool {
    const SINGLE_WORD: [&str; 7] = [
        "placeholder",
        "stub",
        "dummy",
        "fake",
        "mock",
        "temporary",
        "hardcoded",
    ];
    if !SINGLE_WORD.contains(&pattern) {
        return false;
    }
    before
        .chars()
        .last()
        .is_some_and(|c| c == '_' || c.is_alphanumeric())
}

fn is_hardcoded_descriptive(line: &str, pattern: &str, trimmed: &str) -> bool {
    if pattern != "hardcoded" && pattern != "hard-coded" {
        return false;
    }
    let lower = line.to_lowercase();
    lower.contains("from the hardcoded")
        || lower.contains("uses hardcoded")
        || lower.contains("using hardcoded")
        || (trimmed.starts_with("//") && lower.contains("should"))
}

fn check_euphemism_real(line: &str, pattern: &str, before: &str, trimmed: &str) -> bool {
    if trimmed.starts_with("///") || trimmed.starts_with("//!") {
        return false;
    }
    if before.matches('"').count() % 2 == 1 {
        return false;
    }
    if pattern == "unimplemented" && check_unimplemented_exclusions(line, trimmed) {
        return false;
    }
    if pattern == "not implemented" && check_not_implemented_exclusions(line, trimmed) {
        return false;
    }
    if is_mid_identifier_euphemism(pattern, before) {
        return false;
    }
    if is_hardcoded_descriptive(line, pattern, trimmed) {
        return false;
    }
    true
}

fn check_code_pattern_real(before: &str, pattern: &str, trimmed: &str) -> bool {
    let in_string = before.matches('"').count() % 2 == 1;
    let is_doc = trimmed.starts_with("///") || trimmed.starts_with("//!");
    let is_comment = trimmed.starts_with("//");
    // SAFETY: the literals below are string constants for pattern matching against scanned code.
    let keyword_patterns = ["unsafe {", "transmute", "panic!"];
    if keyword_patterns
        .iter()
        .any(|kw| pattern.starts_with(kw.split_whitespace().next().unwrap_or(kw)))
    {
        if let Some(c) = before.chars().last() {
            if c.is_alphanumeric() || c == '_' {
                return false;
            }
        }
    }
    !in_string && !is_doc && !is_comment
}

/// Main dispatcher: is `pattern` a real match in `line`?
pub fn is_real_pattern(line: &str, pattern: &str) -> bool {
    let Some(pos) = line.find(pattern) else {
        return false;
    };
    let trimmed = line.trim();
    let before = &line[..pos];

    if matches!(pattern, "TODO" | "FIXME" | "HACK" | "XXX") {
        return check_tech_debt_real(line, before, trimmed);
    }

    if is_comment_pattern(pattern) {
        return check_comment_pattern_real(line, before, trimmed);
    }

    if is_euphemism_pattern(pattern) {
        return check_euphemism_real(line, pattern, before, trimmed);
    }

    check_code_pattern_real(before, pattern, trimmed)
}

fn is_comment_pattern(pattern: &str) -> bool {
    matches!(
        pattern,
        "were removed"
            | "tests hang"
            | "hang during"
            | "compilation hang"
            | "// skip"
            | "// skipped"
            | "// broken"
            | "// fails"
            | "// disabled"
            | "// fallback"
            | "// degraded"
            | "CUDA_ERROR"
            | "INVALID_PTX"
            | "PTX error"
            | "kernel fail"
    )
}

fn is_euphemism_pattern(pattern: &str) -> bool {
    matches!(
        pattern,
        "placeholder"
            | "stub"
            | "dummy"
            | "fake"
            | "mock"
            | "simplified"
            | "for demonstration"
            | "demo only"
            | "not implemented"
            | "unimplemented"
            | "temporary"
            | "hardcoded"
            | "hard-coded"
            | "magic number"
            | "workaround"
            | "quick fix"
            | "quick-fix"
            | "bandaid"
            | "band-aid"
            | "kludge"
            | "tech debt"
            | "technical debt"
    )
}

/// Scan a single file's text for all configured patterns.
pub fn scan_file(file_path: &str, content: &str, rules: &[PatternRule]) -> Vec<Finding> {
    let test_lines = compute_test_lines(content);
    let mut findings = Vec::new();
    let mut finding_counter: usize = 0;

    for (idx, line) in content.lines().enumerate() {
        let line_num = idx + 1;
        if test_lines.contains(&line_num) {
            continue;
        }
        for rule in rules {
            if !is_real_pattern(line, rule.literal) {
                continue;
            }
            finding_counter += 1;
            findings.push(Finding {
                id: format!(
                    "BH-{}-{:04}",
                    category_prefix(rule.category),
                    finding_counter
                ),
                file: file_path.to_string(),
                line: line_num,
                column: None,
                title: describe_pattern(rule),
                description: format!("Pattern `{}` detected", rule.literal),
                severity: rule.severity,
                category: rule.category,
                suspiciousness: rule.suspiciousness,
                discovered_by: "Pattern".to_string(),
            });
        }
    }
    findings
}

fn category_prefix(cat: DefectCategory) -> &'static str {
    match cat {
        DefectCategory::LogicErrors => "LOGIC",
        DefectCategory::MemorySafety => "MEM",
        DefectCategory::SilentDegradation => "SILENT",
        DefectCategory::TestDebt => "TEST",
        DefectCategory::HiddenDebt => "HIDE",
        DefectCategory::GpuKernelBugs => "GPU",
        DefectCategory::ConfigurationErrors => "CONFIG",
        DefectCategory::Unknown => "UNK",
    }
}

fn describe_pattern(rule: &PatternRule) -> String {
    let kind = match rule.category {
        DefectCategory::LogicErrors => "Logic / runtime fault",
        DefectCategory::MemorySafety => "Memory safety concern",
        DefectCategory::SilentDegradation => "Silent degradation",
        DefectCategory::TestDebt => "Test debt marker",
        DefectCategory::HiddenDebt => "Hidden tech debt",
        DefectCategory::GpuKernelBugs => "GPU kernel bug signal",
        DefectCategory::ConfigurationErrors => "Configuration error",
        DefectCategory::Unknown => "Pattern",
    };
    format!(
        "{kind} ({severity}): `{literal}`",
        kind = kind,
        severity = rule.severity,
        literal = rule.literal
    )
}

#[cfg(test)]
mod tests {
    use super::super::types::FindingSeverity;
    use super::*;

    #[test]
    fn todo_in_comment_is_real() {
        assert!(is_real_pattern("// TODO: fix this", "TODO"));
    }

    #[test]
    fn todo_in_string_is_not_real() {
        assert!(!is_real_pattern(r#"let msg = "TODO: implement";"#, "TODO"));
    }

    #[test]
    fn todo_in_doc_comment_is_not_real() {
        assert!(!is_real_pattern("/// TODO: document this", "TODO"));
    }

    #[test]
    fn compute_test_lines_marks_cfg_test_blocks() {
        let src = "fn prod() {}\n\n#[cfg(test)]\nmod t {\n    fn test_a() {}\n}\n";
        let lines = compute_test_lines(src);
        assert!(lines.contains(&3));
        assert!(lines.contains(&4));
        assert!(lines.contains(&5));
        assert!(!lines.contains(&1));
    }

    #[test]
    fn scan_file_skips_test_blocks() {
        let src =
            "fn prod() { x.unwrap() }\n#[cfg(test)]\nmod t {\n    fn test() { y.unwrap() }\n}\n";
        let rules = super::super::taxonomy::RUNTIME_PATTERNS;
        let findings = scan_file("lib.rs", src, rules);
        assert_eq!(findings.len(), 1, "should find the production unwrap only");
        assert_eq!(findings[0].line, 1);
    }

    #[test]
    fn scan_file_flags_memory_safety() {
        let src = "fn f() {\n    unsafe { core::mem::transmute::<u8, u8>(0) }\n}\n";
        let rules = super::super::taxonomy::RUNTIME_PATTERNS;
        let findings = scan_file("lib.rs", src, rules);
        assert!(findings
            .iter()
            .any(|f| matches!(f.category, DefectCategory::MemorySafety)));
    }

    #[test]
    fn scan_file_produces_stable_shape() {
        let src = "fn f() { x.unwrap() }\n";
        let rules = super::super::taxonomy::RUNTIME_PATTERNS;
        let findings = scan_file("lib.rs", src, rules);
        assert_eq!(findings.len(), 1);
        let f = &findings[0];
        assert_eq!(f.file, "lib.rs");
        assert_eq!(f.line, 1);
        assert!(f.id.starts_with("BH-LOGIC-"));
        assert_eq!(f.severity, FindingSeverity::Medium);
    }
}
