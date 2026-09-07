//! PMAT-695 (#1156): the build must be hermetic.
//!
//! `build.rs` used to fetch four JavaScript/CSS assets from unpkg at build
//! time — two of them pinned to `@latest` — into the gitignored
//! `assets/vendor/`, gzip them and then treat its own output as a build
//! input. That is a network input in a sovereign build, non-deterministic by
//! construction, and it means a clean-room build that "passed" only passed
//! because the machine had a route to the internet.
//!
//! These four tests pin the fixed shape: the assets are committed files
//! carried with a committed `SHA256SUMS`, the build script reads and verifies
//! them, and it declares no watch on a path git does not know.
//!
//! They read the repository, not a fixture, so they fail the moment someone
//! re-adds a fetch. Run: `cargo test --lib -- hermetic_assets`.

use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The repository root, as the compiler saw it.
fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read_build_script() -> String {
    let path = repo_root().join("build.rs");
    let read = std::fs::read_to_string(&path);
    assert!(
        read.is_ok(),
        "build.rs must be readable at {}: {read:?}",
        path.display()
    );
    read.expect("readability asserted on the line above")
}

/// `assets/vendor/SHA256SUMS` in `sha256sum` format: `<hex>  <relative path>`.
///
/// Returns `(hex, path-relative-to-assets/vendor)` pairs. A malformed line is
/// a failure of the file, not something to skip: skipping is how a checksum
/// file quietly comes to cover nothing.
fn parse_checksums(text: &str) -> Vec<(String, String)> {
    let mut rows = Vec::new();
    for line in text.lines() {
        let line = line.trim_end();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let split = line.split_once("  ");
        assert!(
            split.is_some(),
            "SHA256SUMS line is not `<hex>  <name>`: {line:?}"
        );
        let (hex, name) = split.expect("shape asserted on the line above");
        assert_eq!(
            hex.len(),
            64,
            "SHA256SUMS digest is not 64 hex characters: {line:?}"
        );
        let name = name.trim().trim_start_matches("./").to_string();
        assert!(
            name.starts_with("assets/vendor/"),
            "SHA256SUMS must name repository-root relative paths under assets/vendor/: {name}"
        );
        rows.push((hex.to_string(), name));
    }
    rows
}

fn checksums_path() -> PathBuf {
    repo_root().join("assets/vendor/SHA256SUMS")
}

fn sha256_hex(path: &Path) -> String {
    let read = std::fs::read(path);
    assert!(
        read.is_ok(),
        "vendored asset must be readable at {}: {read:?}",
        path.display()
    );
    let bytes = read.expect("readability asserted on the line above");
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    hasher
        .finalize()
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `git ls-files <pathspec>`, or `None` when this is not a git checkout (a
/// crates.io unpack is not one, and must not fail the suite there).
fn git_ls_files(pathspec: &str) -> Option<Vec<String>> {
    let out = Command::new("git")
        .args(["ls-files", "--", pathspec])
        .current_dir(repo_root())
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    Some(
        String::from_utf8_lossy(&out.stdout)
            .lines()
            .map(str::to_string)
            .collect(),
    )
}

/// (a) The build script names no remote at all.
///
/// Substring-level, deliberately: a URL assembled from parts, a comment
/// describing the old fetch, or a helper still called `download_asset` are all
/// evidence the path is still there or is one edit from returning.
#[test]
fn build_script_has_no_network_input() {
    let script = read_build_script();
    for banned in ["unpkg.com", "https://", "http://", "download"] {
        let hits: Vec<&str> = script
            .lines()
            .filter(|l| l.contains(banned))
            .take(5)
            .collect();
        assert!(
            hits.is_empty(),
            "build.rs must contain no network input, found {banned:?} on: {hits:#?}"
        );
    }
}

/// (b) Every file SHA256SUMS names is present and hashes to the recorded
/// digest — the same check `build.rs` makes, run from the test target.
#[test]
fn vendored_assets_match_the_committed_checksums() {
    let path = checksums_path();
    let read = std::fs::read_to_string(&path);
    assert!(
        read.is_ok(),
        "assets/vendor/SHA256SUMS must be committed at {}: {read:?}",
        path.display()
    );
    let text = read.expect("presence asserted on the line above");
    let rows = parse_checksums(&text);
    assert!(
        rows.len() >= 4,
        "SHA256SUMS must cover the four vendored assets, covers {}",
        rows.len()
    );
    for (expected, name) in rows {
        let file = repo_root().join(&name);
        assert!(
            file.exists(),
            "SHA256SUMS names {name}, which is not present at {}",
            file.display()
        );
        assert_eq!(
            sha256_hex(&file),
            expected,
            "vendored asset {name} does not match its committed checksum"
        );
    }
}

/// (c) Those files are tracked. A vendored asset that is gitignored is
/// regenerated silently and ships in no package.
#[test]
fn vendored_assets_are_tracked() {
    let Some(tracked) = git_ls_files("assets/vendor") else {
        return; // not a git checkout: nothing to assert
    };
    assert!(
        tracked.iter().any(|p| p.ends_with("SHA256SUMS")),
        "assets/vendor/SHA256SUMS must be tracked, git knows: {tracked:#?}"
    );
    let text = std::fs::read_to_string(checksums_path())
        .expect("assets/vendor/SHA256SUMS must be committed");
    for (_, name) in parse_checksums(&text) {
        let want = name.clone();
        assert!(
            tracked.contains(&want),
            "{want} is named by SHA256SUMS but is not tracked; git knows: {tracked:#?}"
        );
    }
}

/// (d) Every `cargo:rerun-if-changed=` the script declares names a path git
/// knows. A watch on a gitignored directory the script itself writes makes the
/// crate permanently stale (or, in a clean checkout, watches nothing at all).
#[test]
fn no_rerun_if_changed_on_ignored_paths() {
    let script = read_build_script();
    let mut declared = Vec::new();
    for line in script.lines() {
        let Some(rest) = line.split("cargo:rerun-if-changed=").nth(1) else {
            continue;
        };
        let Some(path) = rest.split('"').next() else {
            continue;
        };
        // The two interpolated sites (the schema walk and the git provenance
        // watches, both resolved at build time) are pinned by
        // `rerun_if_changed_paths_exist_inside_the_tree` in build_support.rs.
        if path.contains('{') || path.contains('$') {
            continue;
        }
        declared.push(path.to_string());
    }
    // Anti-vacuity: a rotted extractor must fail here, not certify an empty set.
    assert!(
        declared.len() >= 5,
        "extractor found only {} literal watches in build.rs: {declared:?}",
        declared.len()
    );
    // Every committed asset is watched by name: a vendored file that changes
    // without re-running the script is a build that ships a stale asset, and a
    // watch on the enclosing directory would name an ignored path.
    let read = std::fs::read_to_string(checksums_path());
    assert!(
        read.is_ok(),
        "assets/vendor/SHA256SUMS must be committed: {read:?}"
    );
    let text = read.expect("presence asserted on the line above");
    let mut required = vec!["assets/vendor/SHA256SUMS".to_string()];
    for (_, name) in parse_checksums(&text) {
        required.push(name);
    }
    for want in required {
        assert!(
            declared.contains(&want),
            "build.rs does not watch {want}; it watches {declared:?}"
        );
    }

    if git_ls_files("build.rs").is_none() {
        return; // not a git checkout
    }
    for path in declared {
        let listed = git_ls_files(&path).unwrap_or_default();
        assert!(
            !listed.is_empty(),
            "build.rs watches {path:?}, which git does not track (ignored or missing)"
        );
    }
}
