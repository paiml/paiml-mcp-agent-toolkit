//! Sprint 85 TDD Tests: Code Entropy Reduction (GH-1486)
//!
//! Sprint 85 split the recursive file collector in
//! `src/cli/analysis/symbol_table_extraction.rs` into small helpers. These
//! tests drive the shipped entry point, `pmat::cli::analysis::handle_analyze_symbol_table`,
//! over a scratch tree and read back the files it walked (every fixture source
//! file defines one uniquely named function, so the JSON `symbols[].file` set is
//! exactly the set the collector kept). No walker or filter is re-implemented
//! here: a regression in the production collector turns these red.

use pmat::cli::analysis::handle_analyze_symbol_table;
use pmat::cli::SymbolTableOutputFormat;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use tempfile::TempDir;

/// Source files the collector must keep, with the function each defines.
const COLLECTED: [&str; 4] = [
    "src/main.rs",
    "src/handlers/mod.rs",
    "src/handlers/complexity.rs",
    "tests/integration_test.rs",
];

/// Files the collector must skip: not a source file, or inside a skipped directory.
const SKIPPED: [&str; 5] = [
    "README.md",
    "target/debug/app.rs",
    ".git/hooks.rs",
    "node_modules/pkg/index.js",
    ".hidden/secret.rs",
];

fn fn_name(rel: &str) -> String {
    let stem: String = rel
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("probe_{stem}")
}

fn write_file(base: &Path, rel: &str) {
    let path = base.join(rel);
    std::fs::create_dir_all(path.parent().unwrap()).expect("create dir");
    std::fs::write(&path, format!("pub fn {}() {{}}\n", fn_name(rel))).expect("write file");
}

fn fixture() -> TempDir {
    let dir = TempDir::new().expect("temp dir");
    for rel in COLLECTED.iter().chain(SKIPPED.iter()) {
        write_file(dir.path(), rel);
    }
    dir
}

/// Run the production handler and return the relative paths of the files it scanned.
async fn scanned(
    root: &Path,
    include: &[String],
    exclude: &[String],
) -> anyhow::Result<BTreeSet<String>> {
    let out = root.join("..").join(format!(
        "sprint85-out-{}.json",
        root.file_name().unwrap().to_string_lossy()
    ));
    let result = handle_analyze_symbol_table(
        root.to_path_buf(),
        SymbolTableOutputFormat::Json,
        None,
        None,
        include,
        exclude,
        false,
        false,
        Some(out.clone()),
        false,
        10,
    )
    .await;
    let parsed = result.and_then(|()| {
        let text = std::fs::read_to_string(&out)?;
        let json: serde_json::Value = serde_json::from_str(&text)?;
        Ok(json)
    });
    let _ = std::fs::remove_file(&out);
    let json = parsed?;
    let files = json["symbols"]
        .as_array()
        .expect("symbols array")
        .iter()
        .map(|s| {
            let f = PathBuf::from(s["file"].as_str().expect("file"));
            f.strip_prefix(root)
                .unwrap_or(&f)
                .to_string_lossy()
                .to_string()
        })
        .collect();
    Ok(files)
}

fn set(items: &[&str]) -> BTreeSet<String> {
    items.iter().map(|s| (*s).to_string()).collect()
}

#[tokio::test]
async fn test_collect_files_recursive_basic_functionality() {
    let dir = fixture();
    let got = scanned(dir.path(), &[], &[]).await.expect("scan succeeds");
    assert_eq!(
        got,
        set(&COLLECTED),
        "exactly the source files outside skipped dirs"
    );
}

#[tokio::test]
async fn test_exclude_pattern_filtering() {
    let dir = fixture();
    let got = scanned(dir.path(), &[], &["handlers".to_string()])
        .await
        .expect("scan succeeds");
    assert_eq!(got, set(&["src/main.rs", "tests/integration_test.rs"]));
}

#[tokio::test]
async fn test_include_pattern_filtering() {
    let dir = fixture();
    let got = scanned(dir.path(), &["main".to_string()], &[])
        .await
        .expect("scan succeeds");
    assert_eq!(got, set(&["src/main.rs"]));
}

#[tokio::test]
async fn test_directory_exclusion_logic() {
    let dir = fixture();
    let got = scanned(dir.path(), &[], &[]).await.expect("scan succeeds");
    for skipped in SKIPPED {
        assert!(!got.contains(skipped), "{skipped} must not be collected");
    }
}

#[tokio::test]
async fn test_source_file_detection() {
    let dir = TempDir::new().expect("temp dir");
    let kept = ["a.rs", "b.js", "c.ts", "d.py", "e.java"];
    let dropped = ["README.md", "config.toml", "image.png"];
    for rel in kept.iter().chain(dropped.iter()) {
        write_file(dir.path(), rel);
    }
    let got = scanned(dir.path(), &[], &[]).await.expect("scan succeeds");
    assert_eq!(got, set(&kept));
}

#[tokio::test]
async fn test_empty_directory_handling() {
    let dir = TempDir::new().expect("temp dir");
    std::fs::create_dir(dir.path().join("empty")).expect("mkdir");
    // The production handler refuses a tree with no analysable source rather
    // than printing a plausible-looking zero table (#1015).
    let err = scanned(&dir.path().join("empty"), &[], &[])
        .await
        .expect_err("an empty directory yields no analysed files");
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn test_nonexistent_directory_error_handling() {
    let err = scanned(Path::new("/nonexistent/directory"), &[], &[])
        .await
        .expect_err("nonexistent directory must error");
    assert!(err.to_string().contains("/nonexistent/directory"), "{err}");
}

#[tokio::test]
async fn test_complex_directory_structure_integration() {
    let dir = fixture();
    let deep = [
        "src/deeply/nested/modules/test.rs",
        "src/another/branch/test.rs",
    ];
    for rel in deep {
        write_file(dir.path(), rel);
    }
    let got = scanned(dir.path(), &[], &[]).await.expect("scan succeeds");
    let mut want = set(&COLLECTED);
    want.extend(deep.iter().map(|s| (*s).to_string()));
    assert_eq!(got, want, "nested directories are walked");
}
