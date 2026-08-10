//! Regression tests for generated proptest files.
//!
//! Lives in its own file rather than at the end of `test_generation.rs`:
//! that file is `include!`d into `mod.rs` ahead of `mutation_testing.rs`, so a
//! test module at its end put items after a test module in the expanded
//! source (`clippy::items_after_test_module`, denied by `ci / lint`).

use super::*;

const MANIFEST_WITH_PROPTEST: &str = r#"
[package]
name = "tiny-crate"
version = "0.1.0"

[dev-dependencies]
proptest = "1"
"#;

const MANIFEST_WITHOUT_PROPTEST: &str = r#"
[package]
name = "tiny-crate"
version = "0.1.0"
"#;

#[test]
fn test_read_generated_test_targets() {
    let with = read_generated_test_targets(MANIFEST_WITH_PROPTEST).unwrap();
    // cargo turns `-` into `_` for the crate identifier an integration
    // test has to import.
    assert_eq!(with.crate_ident, "tiny_crate");
    assert!(with.has_proptest);

    let without = read_generated_test_targets(MANIFEST_WITHOUT_PROPTEST).unwrap();
    assert!(!without.has_proptest);

    assert!(read_generated_test_targets("not a manifest {{{").is_none());
}

/// The generated file used to open with `use crate::<file_stem>::*;`, which
/// in a `tests/` integration target resolves against the test crate and
/// fails with E0432.
#[test]
fn test_generated_module_imports_the_crate_under_test() {
    let service = CoverageImprovementService::new(CoverageImprovementConfig::default());
    let syntax_tree = syn::parse_file("pub fn keep(x: i32) -> i32 { x }").unwrap();
    let functions = service.extract_public_functions(&syntax_tree);

    let module = service
        .generate_proptest_module("tiny_crate", &PathBuf::from("src/lib.rs"), &functions)
        .unwrap();

    assert!(module.contains("use tiny_crate::*;"), "{module}");
    assert!(!module.contains("use crate::"), "{module}");
}

/// Emitting `use proptest::prelude::*;` into a project that has no proptest
/// dev-dependency broke `cargo test` there, and pmat still reported the
/// tests as generated. Refusing is the honest outcome.
#[tokio::test]
async fn test_no_tests_written_without_a_proptest_dev_dependency() {
    let temp = tempfile::TempDir::new().unwrap();
    let root = temp.path();
    std::fs::write(root.join("Cargo.toml"), MANIFEST_WITHOUT_PROPTEST).unwrap();
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("src/lib.rs"),
        "pub fn keep(x: i32) -> i32 { x }\n",
    )
    .unwrap();

    let service = CoverageImprovementService::new(CoverageImprovementConfig {
        project_path: root.to_path_buf(),
        ..CoverageImprovementConfig::default()
    });

    let generated = service
        .generate_property_tests(&[PathBuf::from("src/lib.rs")])
        .await
        .unwrap();

    assert_eq!(generated, 0, "tests were counted but cannot compile");
    assert!(
        !root.join("tests").join("proptest_lib.rs").exists(),
        "a non-compiling test file was left in the target project"
    );
}
