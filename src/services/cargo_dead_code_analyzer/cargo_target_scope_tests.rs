//! Regression tests for the cargo target set the dead-code analyzer asks for.
//!
//! `examples/` and `benches/` are in scope by default, and the totals walk
//! counted them, but `run_cargo_check` only ever passed `--lib`/`--bins`:
//! rustc never compiled those trees, so nothing in them could be reported.
//! A crate whose only dead code lived in `examples/` and `benches/` read
//! "Files analyzed: 3 / Total dead lines: 0" while `cargo check --all-targets`
//! on the same crate emitted 8 `is never used` warnings — the percentage was
//! not computed over the set it named.

use super::named_targets;

const MANIFEST: &str = "[package]\nname = \"scopefix\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";

/// A package with a lib plus the given `dir/file` sources.
fn crate_with(dir: &str, files: &[&str], extra_manifest: &str) -> tempfile::TempDir {
    let tmp = tempfile::tempdir().expect("tempdir");
    std::fs::create_dir_all(tmp.path().join("src")).expect("mkdir src");
    std::fs::write(tmp.path().join("src/lib.rs"), "pub fn used() {}\n").expect("write lib");
    std::fs::write(
        tmp.path().join("Cargo.toml"),
        format!("{MANIFEST}{extra_manifest}"),
    )
    .expect("write manifest");
    std::fs::create_dir_all(tmp.path().join(dir)).expect("mkdir");
    for f in files {
        let p = tmp.path().join(dir).join(f);
        if let Some(parent) = p.parent() {
            std::fs::create_dir_all(parent).expect("mkdir parent");
        }
        std::fs::write(&p, "fn main() {}\n").expect("write");
    }
    tmp
}

/// Every auto-discovered example becomes a named cargo target, so the compile
/// scope can be made equal to the walk scope.
#[test]
fn examples_are_discovered_as_named_targets() {
    let tmp = crate_with("examples", &["demo.rs", "other.rs"], "");
    assert_eq!(
        named_targets(tmp.path(), "example"),
        vec!["demo".to_string(), "other".to_string()],
        "examples/ must yield one cargo target per file"
    );
}

/// Same for `benches/`. Note the LIB's implicit bench target must not appear:
/// naming it via the blanket `--benches` flag re-compiles the lib under
/// `cfg(test)` and drags `#[cfg(test)]` items into a report that
/// `--include-tests` was never given.
#[test]
fn benches_are_named_without_the_libs_implicit_bench_target() {
    let tmp = crate_with(
        "benches",
        &["b.rs"],
        "\n[[bench]]\nname = \"b\"\nharness = false\n",
    );
    assert_eq!(named_targets(tmp.path(), "bench"), vec!["b".to_string()]);
}

/// A target gated behind `required-features` is skipped. Naming one whose
/// features are off is a hard cargo error ("target `x` requires the features:
/// `demo`") that would abort the entire dead-code analysis.
#[test]
fn feature_gated_targets_are_skipped() {
    let tmp = crate_with(
        "examples",
        &["plain.rs", "gated.rs"],
        "\n[features]\ndemo = []\n\n[[example]]\nname = \"gated\"\nrequired-features = [\"demo\"]\n",
    );
    assert_eq!(
        named_targets(tmp.path(), "example"),
        vec!["plain".to_string()],
        "a feature-gated example must not be named on the cargo command line"
    );
}

/// A crate with no such tree contributes no arguments at all.
#[test]
fn missing_directory_yields_no_targets() {
    let tmp = crate_with("src", &[], "");
    assert!(named_targets(tmp.path(), "example").is_empty());
    assert!(named_targets(tmp.path(), "bench").is_empty());
}

/// Not a cargo package at all: no metadata, no targets, no panic.
#[test]
fn a_non_package_directory_yields_no_targets() {
    let tmp = tempfile::tempdir().expect("tempdir");
    assert!(named_targets(tmp.path(), "example").is_empty());
}
