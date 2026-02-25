// Quality check functions - extracted for file health (CB-040)
// Split into submodules via include!() pattern

/// Check if path is a build artifact that should be excluded from duplicate detection
pub fn is_build_artifact(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/target/")
        || path_str.contains("/build/")
        || path_str.contains("/out/")
        || path_str.contains("/.cargo/")
        || path_str.contains("/node_modules/")
        || path_str.contains("/dist/")
        || path_str.contains("/.git/")
        || path_str.contains("/generated/")
        || path_str.starts_with("./target/")
        || path_str.starts_with("target/")
}

// Complexity checking: check_complexity, load_exclude_paths, is_violation_excluded,
// process_complexity_violation
include!("quality_checks_part1_complexity.rs");

// Dead code detection: check_dead_code
include!("quality_checks_part1_dead_code.rs");

// SATD detection: check_satd
include!("quality_checks_part1_satd.rs");

// Entropy analysis: check_entropy, check_entropy_with_excludes,
// load_max_pattern_repetition
include!("quality_checks_part1_entropy.rs");

// Security scanning: check_security
include!("quality_checks_part1_security.rs");
