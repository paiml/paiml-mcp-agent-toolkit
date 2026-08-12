use super::report_path;

#[test]
fn test_report_path_trims_leading_dot_slash() {
    assert_eq!(report_path("./src/ast/core.rs"), "src/ast/core.rs");
}

#[test]
fn test_report_path_keeps_everything_else() {
    assert_eq!(report_path("src/ast/core.rs"), "src/ast/core.rs");
    assert_eq!(report_path("/abs/src/ast/core.rs"), "/abs/src/ast/core.rs");
    assert_eq!(report_path("core.rs"), "core.rs");
    assert_eq!(report_path(""), "");
}

/// The whole reason this helper exists: two files that share a basename must
/// not render as the same row. `Path::file_name()` collapsed them.
#[test]
fn test_report_path_distinguishes_same_basename_in_different_dirs() {
    let a = "./src/ast/core_tests_properties.rs";
    let b = "./src/ast/core/core_tests_properties.rs";

    assert_ne!(report_path(a), report_path(b));

    let basename = |p: &str| {
        std::path::Path::new(p)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(p)
            .to_string()
    };
    assert_eq!(
        basename(a),
        basename(b),
        "precondition: these differ only in directory"
    );
}
