// Code analysis and inspection utility functions for CLI handlers
// Included by handler_utils.rs - shares parent module scope

/// Check if content has heavy cfg/target_feature attributes (SIMD/arch-specific code)
///
/// Files with many cfg attributes are often SIMD or architecture-specific code
/// that only compiles on certain platforms, leading to false positives in dead code analysis.
#[must_use]
pub fn is_heavily_cfg_gated(content: &str) -> bool {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let cfg_count = content.matches("#[cfg(target").count()
        + content.matches("#[target_feature").count()
        + content.matches("#[cfg(feature").count();
    cfg_count > 3
}

/// Check if a path should be excluded from production code analysis
///
/// Excludes test files, falsification modules, SIMD-heavy directories
#[must_use]
pub fn is_excluded_from_analysis(path_str: &str) -> bool {
    debug_assert!(!path_str.is_empty(), "path_str must not be empty");
    path_str.ends_with("_tests.rs")
        || path_str.contains("/tests/")
        || path_str.contains("/falsification/")
        || path_str.contains("_falsification")
        || path_str.contains("/quantize/")
        || path_str.contains("/simd/")
        || path_str.contains("/benches/")
        || path_str.contains("/examples/")
}

/// Check if a line looks like a test module marker
#[must_use]
pub fn is_test_module_marker(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let trimmed = line.trim();
    trimmed == "#[cfg(test)]" || trimmed.starts_with("#[cfg(test)]")
}

/// Check if a line is a dead code annotation
#[must_use]
pub fn is_dead_code_annotation(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let trimmed = line.trim();
    trimmed.starts_with("#[allow(dead_code)]") || trimmed.starts_with("#[allow(unused")
}

/// Check if a line is a code item declaration (fn, struct, enum, etc.)
#[must_use]
pub fn is_code_item_declaration(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let trimmed = line.trim();
    trimmed.starts_with("pub fn ")
        || trimmed.starts_with("fn ")
        || trimmed.starts_with("pub struct ")
        || trimmed.starts_with("struct ")
        || trimmed.starts_with("pub enum ")
        || trimmed.starts_with("enum ")
        || trimmed.starts_with("pub const ")
        || trimmed.starts_with("const ")
        || trimmed.starts_with("pub static ")
        || trimmed.starts_with("static ")
        || trimmed.starts_with("pub type ")
        || trimmed.starts_with("type ")
        || trimmed.starts_with("pub trait ")
        || trimmed.starts_with("trait ")
        || trimmed.starts_with("impl ")
}
