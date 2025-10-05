//! RED tests for smart test filtering
//!
//! ROOT CAUSE FIX: Don't run entire test suite for every mutant
//! SOLUTION: Extract module path from file, run only relevant tests

use std::path::PathBuf;

/// Extract module path from file path for test filtering
///
/// Examples:
/// - "server/src/services/mutation/types.rs" -> "services::mutation"
/// - "src/cli/handlers/mod.rs" -> "cli::handlers"
/// - "src/lib.rs" -> "" (run all tests)
fn extract_module_path(file_path: &str) -> String {
    // Handle external crates (paths starting with ../)
    if file_path.starts_with("../") || file_path.starts_with("..\\") {
        return String::new(); // Use package-level testing for external crates
    }

    // Remove "server/src/" or "src/" prefix
    let relative = file_path
        .strip_prefix("server/src/")
        .or_else(|| file_path.strip_prefix("src/"))
        .unwrap_or(file_path);

    // Remove ".rs" suffix
    let without_ext = relative.strip_suffix(".rs").unwrap_or(relative);

    // Handle lib.rs and main.rs - run all tests
    if without_ext == "lib" || without_ext == "main" {
        return String::new();
    }

    // Check if this is a mod.rs file
    let is_mod_file = without_ext.ends_with("/mod");

    // Remove "/mod" at end for processing
    let without_mod = without_ext
        .strip_suffix("/mod")
        .unwrap_or(without_ext);

    // Split into parts
    let parts: Vec<&str> = without_mod.split('/').collect();

    // Determine which parts to use
    let module_parts = if is_mod_file {
        // For mod.rs files, keep full path (it's the module itself)
        // e.g., "cli/handlers/mod" -> "cli::handlers"
        &parts[..]
    } else if parts.len() > 3 {
        // For deep paths, use parent module for broader coverage
        // e.g., "services/mutation/operators/arithmetic" -> "services::mutation::operators"
        &parts[..parts.len() - 1]
    } else if parts.len() > 1 {
        // For 2-3 levels, use parent module
        // e.g., "services/mutation/types" -> "services::mutation"
        &parts[..parts.len() - 1]
    } else {
        // Single level, use as-is
        // e.g., "parser" -> "parser"
        &parts[..]
    };

    // Join with "::"
    module_parts.join("::")
}

#[test]
fn test_extract_module_from_nested_file() {
    // RED: This will fail until we implement
    let path = "server/src/services/mutation/types.rs";
    let module = extract_module_path(path);
    assert_eq!(module, "services::mutation");
}

#[test]
fn test_extract_module_from_mod_file() {
    let path = "server/src/cli/handlers/mod.rs";
    let module = extract_module_path(path);
    assert_eq!(module, "cli::handlers");
}

#[test]
fn test_extract_module_from_src_prefix() {
    let path = "src/services/mutation/types.rs";
    let module = extract_module_path(path);
    assert_eq!(module, "services::mutation");
}

#[test]
fn test_extract_module_from_lib_rs() {
    let path = "src/lib.rs";
    let module = extract_module_path(path);
    assert_eq!(module, ""); // Run all tests
}

#[test]
fn test_extract_module_from_main_rs() {
    let path = "src/main.rs";
    let module = extract_module_path(path);
    assert_eq!(module, ""); // Run all tests
}

#[test]
fn test_extract_module_single_level() {
    let path = "src/parser.rs";
    let module = extract_module_path(path);
    assert_eq!(module, "parser");
}

#[test]
fn test_extract_module_deep_nesting() {
    let path = "server/src/services/mutation/operators/arithmetic.rs";
    let module = extract_module_path(path);
    // For deep paths, use parent module for broader test coverage
    assert_eq!(module, "services::mutation::operators");
}

#[test]
fn test_extract_module_from_external_crate() {
    // External crates (like pforge) should use package-level testing
    let path = "../pforge/crates/pforge-config/src/validator.rs";
    let module = extract_module_path(path);
    // For external crates, return empty to trigger package detection
    assert_eq!(module, "");
}

#[test]
fn test_extract_module_with_underscores() {
    let path = "src/mutation_testing/test_runner.rs";
    let module = extract_module_path(path);
    assert_eq!(module, "mutation_testing");
}
