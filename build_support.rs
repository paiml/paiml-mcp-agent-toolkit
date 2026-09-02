// Testable core of `build.rs`.
//
// #976. A build script is not part of any cargo test target: `cargo test`
// never compiles `build.rs`, so a `#[cfg(test)] mod tests` written there is
// dead text. The only honest way to test build-script logic is to move it into
// a file that *two* compilation units share — `build.rs` (via `include!`) and
// the library's test target (via `#[cfg(test)] #[path] mod`, see the bottom of
// `src/lib.rs`). This is that file.
//
// Deliberately self-contained: every path is fully qualified, because
// `build.rs` already has `use std::{env, fs, path::Path}` at its top and the
// library denies unused imports. Nothing here may be duplicated back into
// `build.rs` — one rule, one implementation.

/// What `main` decides to do about the generated MCP discovery tables, from
/// the environment alone.
///
/// Extracted so the decision can be tested. In `build.rs` it was three
/// `env::var(..).is_ok()` calls threaded through two `if`s, half of which no
/// ordinary build ever takes: a coverage run reported `main` at 56.1%
/// precisely because the fast-build and coverage branches never execute
/// together with the normal one.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BuildPlan {
    /// `PMAT_FAST_BUILD`: emit stubs and skip every heavy step.
    FastStubs,
    /// Coverage run or `SKIP_MCP_TABLES`: do the heavy steps, stub the tables.
    StubTables,
    /// The ordinary build: generate the real tables.
    GenerateTables,
}

/// Decide the build plan. `PMAT_FAST_BUILD` dominates: it short-circuits
/// before any heavy work, so the table flags cannot alter its outcome.
const fn build_plan(fast_build: bool, coverage: bool, skip_tables: bool) -> BuildPlan {
    if fast_build {
        BuildPlan::FastStubs
    } else if coverage || skip_tables {
        BuildPlan::StubTables
    } else {
        BuildPlan::GenerateTables
    }
}

/// GH-283: does `rustc --version` output report Rust >= 1.94?
///
/// Split out of `rustc_is_at_least_1_94` so the parse can be tested without
/// spawning a compiler. Any unparsable input returns `false`, which keeps the
/// `#![feature(coverage_attribute)]` gate in place — the safe direction, since
/// using the feature on a compiler that stabilised it is a hard error (E0554).
fn version_output_is_at_least_1_94(version_output: &str) -> bool {
    let Some(version) = version_output.split_whitespace().nth(1) else {
        return false;
    };
    let mut parts = version.split('.');
    let Some(major) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    let Some(minor) = parts.next().and_then(|s| s.parse::<u32>().ok()) else {
        return false;
    };
    major > 1 || (major == 1 && minor >= 94)
}

/// Is this directory the extracted copy `cargo publish` builds in?
///
/// `cargo publish` unpacks the crate under `target/package/<name>-<version>`;
/// asset downloads must not run there because they would write into the
/// packaged source tree.
fn path_is_publish_dir(dir: &std::path::Path) -> bool {
    dir.to_string_lossy().contains("/target/package/")
}

/// Gzip `input` at maximum compression, or `None` if the encoder fails.
fn create_compressed_data(input: &[u8]) -> Option<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use std::io::Write as _;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
    encoder.write_all(input).ok()?;
    encoder.finish().ok()
}

/// Write compressed bytes to `gz_path`, reporting the size delta.
///
/// Returns `true` only when the bytes actually reached disk, so a caller can
/// tell "compressed" from "silently dropped". The original returned `()` and
/// swallowed the write error inside `if fs::write(..).is_ok()`.
fn write_compressed_file(
    gz_path: &std::path::Path,
    compressed: &[u8],
    filename: &str,
    original_size: usize,
) -> bool {
    if std::fs::write(gz_path, compressed).is_err() {
        println!("cargo:warning=Failed to write compressed {filename}");
        return false;
    }
    if let Ok(metadata) = std::fs::metadata(gz_path) {
        println!(
            "cargo:warning=Compressed {} ({} -> {} bytes)",
            filename,
            original_size,
            metadata.len()
        );
    }
    true
}

/// Compress `path` to `gz_path` and record its hash in `hash_path`.
///
/// Returns `true` when a `.gz` was produced. A missing or unreadable source,
/// or a failed encode, is a no-op — the build carries on with whatever the
/// previous run left behind.
fn compress_asset(
    path: &std::path::Path,
    gz_path: &std::path::Path,
    hash_path: &std::path::Path,
    filename: &str,
) -> bool {
    if !path.exists() {
        return false;
    }
    let Ok(input) = std::fs::read(path) else {
        return false;
    };
    let Some(compressed) = create_compressed_data(&input) else {
        return false;
    };
    if !write_compressed_file(gz_path, &compressed, filename, input.len()) {
        return false;
    }
    // Save hash for O(1) skip detection on next build.
    if let Some(hash) = calculate_file_hash(path) {
        let _ = write_hash_file(hash_path, &hash);
    }
    true
}

/// Calculate the SHA256 of a file for change detection.
///
/// Returns `None` under `--no-default-features` (no `sha2` in build-deps);
/// callers treat that as "changed" and reprocess unconditionally.
#[cfg(feature = "standard-deps")]
fn calculate_file_hash(path: &std::path::Path) -> Option<String> {
    use sha2::{Digest, Sha256};

    let content = std::fs::read(path).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(&content);
    // sha2 0.11's finalize() returns an Array<u8> that no longer impls
    // LowerHex; encode the digest bytes to lowercase hex explicitly.
    Some(
        hasher
            .finalize()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect(),
    )
}

#[cfg(not(feature = "standard-deps"))]
fn calculate_file_hash(_path: &std::path::Path) -> Option<String> {
    None
}

/// Read the hash recorded beside a compressed asset.
fn read_stored_hash(hash_path: &std::path::Path) -> Option<String> {
    std::fs::read_to_string(hash_path)
        .ok()
        .map(|s| s.trim().to_string())
}

/// Record a hash beside a compressed asset. `false` if it could not be stored.
fn write_hash_file(hash_path: &std::path::Path, hash: &str) -> bool {
    std::fs::write(hash_path, hash).is_ok()
}

/// Has the source changed since the hash beside it was written?
///
/// Every "cannot tell" answer is `true` — unknown means reprocess. Reporting
/// `false` for an unmeasurable source would silently ship a stale asset.
fn has_file_changed(source_path: &std::path::Path, hash_path: &std::path::Path) -> bool {
    if !hash_path.exists() {
        return true;
    }
    let Some(current_hash) = calculate_file_hash(source_path) else {
        return true;
    };
    let Some(stored_hash) = read_stored_hash(hash_path) else {
        return true;
    };
    current_hash != stored_hash
}

/// O(1) skip check: an asset can be skipped only when its `.gz` exists, its
/// source exists, and the source hashes to the recorded value.
fn should_skip_asset(
    source_path: &std::path::Path,
    gz_path: &std::path::Path,
    hash_path: &std::path::Path,
) -> bool {
    if !gz_path.exists() || !source_path.exists() {
        return false;
    }
    !has_file_changed(source_path, hash_path)
}

#[cfg(test)]
mod tests {
    //! #976. These run in the *library* test target (`cargo test --lib`); the
    //! same source is `include!`d by `build.rs`, so a regression here is a
    //! regression in the build script.
    use super::*;

    fn tmpdir() -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "pmat-build-support-{}-{:?}",
            std::process::id(),
            std::thread::current().id()
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("create test dir");
        dir
    }

    // ---- build_plan -----------------------------------------------------

    #[test]
    fn fast_build_wins_over_every_other_flag() {
        assert_eq!(build_plan(true, false, false), BuildPlan::FastStubs);
        assert_eq!(build_plan(true, true, true), BuildPlan::FastStubs);
    }

    #[test]
    fn coverage_or_skip_tables_stubs_the_tables_but_not_the_build() {
        assert_eq!(build_plan(false, true, false), BuildPlan::StubTables);
        assert_eq!(build_plan(false, false, true), BuildPlan::StubTables);
        assert_eq!(build_plan(false, true, true), BuildPlan::StubTables);
    }

    #[test]
    fn an_unflagged_build_generates_the_real_tables() {
        assert_eq!(build_plan(false, false, false), BuildPlan::GenerateTables);
    }

    // ---- rustc version parsing -----------------------------------------

    #[test]
    fn rustc_versions_are_compared_numerically_not_lexically() {
        assert!(version_output_is_at_least_1_94(
            "rustc 1.94.0 (e408947bf 2026-03-25)"
        ));
        assert!(version_output_is_at_least_1_94("rustc 1.101.0"));
        assert!(version_output_is_at_least_1_94("rustc 2.0.0"));
        // "1.9" > "1.94" as a string, but 9 < 94 as a number.
        assert!(!version_output_is_at_least_1_94("rustc 1.9.0"));
        assert!(!version_output_is_at_least_1_94("rustc 1.91.0 (msrv)"));
    }

    #[test]
    fn an_unparsable_rustc_version_keeps_the_feature_gate() {
        // false is the safe answer: it leaves `#![feature(coverage_attribute)]`
        // in place rather than risking E0554 on a stabilised compiler.
        assert!(!version_output_is_at_least_1_94(""));
        assert!(!version_output_is_at_least_1_94("rustc"));
        assert!(!version_output_is_at_least_1_94("rustc nightly"));
        assert!(!version_output_is_at_least_1_94("rustc 1"));
        assert!(!version_output_is_at_least_1_94("rustc x.y.z"));
    }

    // ---- publish detection ---------------------------------------------

    #[test]
    fn only_the_cargo_publish_staging_directory_counts_as_publishing() {
        assert!(path_is_publish_dir(std::path::Path::new(
            "/home/u/proj/target/package/pmat-3.30.0"
        )));
        assert!(!path_is_publish_dir(std::path::Path::new("/home/u/proj")));
        // A *package* directory that is not under target/ is a normal checkout.
        assert!(!path_is_publish_dir(std::path::Path::new(
            "/home/u/package/pmat"
        )));
    }

    // ---- compression ----------------------------------------------------

    #[test]
    fn compressed_bytes_are_a_gzip_stream_that_round_trips() {
        let input = b"pub fn hello() {}\n".repeat(64);
        let compressed = create_compressed_data(&input).expect("gzip must succeed");
        assert_eq!(
            &compressed[..2],
            &[0x1f, 0x8b],
            "output must carry the gzip magic"
        );
        assert!(
            compressed.len() < input.len(),
            "repetitive input must actually shrink: {} -> {}",
            input.len(),
            compressed.len()
        );

        use std::io::Read as _;
        let mut decoded = Vec::new();
        flate2::read::GzDecoder::new(&compressed[..])
            .read_to_end(&mut decoded)
            .expect("gunzip must succeed");
        assert_eq!(decoded, input, "compression must be lossless");
    }

    #[test]
    fn write_compressed_file_reports_failure_instead_of_pretending() {
        let dir = tmpdir();
        let ok = write_compressed_file(&dir.join("out.gz"), b"payload", "out", 7);
        assert!(ok);
        assert_eq!(
            std::fs::read(dir.join("out.gz")).expect("read back"),
            b"payload"
        );

        // A path whose parent does not exist cannot be written.
        let bad = dir.join("no-such-dir").join("out.gz");
        assert!(
            !write_compressed_file(&bad, b"payload", "out", 7),
            "an unwritable target must report false, not success"
        );
        assert!(!bad.exists());
    }

    #[test]
    fn compress_asset_writes_the_gz_and_records_the_source_hash() {
        let dir = tmpdir();
        let src = dir.join("asset.js");
        let gz = dir.join("asset.js.gz");
        let hash = dir.join("asset.js.hash");
        std::fs::write(&src, b"console.log(1);\n".repeat(32)).expect("write source");

        assert!(compress_asset(&src, &gz, &hash, "asset.js"));
        assert!(gz.exists(), "the .gz must exist");
        assert_eq!(
            std::fs::read(&gz).expect("read gz")[..2],
            [0x1f, 0x8b],
            "the .gz must be a gzip stream"
        );
        assert_eq!(
            read_stored_hash(&hash),
            calculate_file_hash(&src),
            "the recorded hash must be the source's hash"
        );
    }

    #[test]
    fn compress_asset_is_a_no_op_when_the_source_is_missing() {
        let dir = tmpdir();
        let gz = dir.join("missing.js.gz");
        assert!(!compress_asset(
            &dir.join("missing.js"),
            &gz,
            &dir.join("missing.js.hash"),
            "missing.js"
        ));
        assert!(!gz.exists(), "no source means no output file at all");
    }

    // ---- the O(1) hash cache -------------------------------------------

    #[cfg(feature = "standard-deps")]
    #[test]
    fn the_hash_cache_skips_an_unchanged_asset_and_reprocesses_a_changed_one() {
        let dir = tmpdir();
        let src = dir.join("v.js");
        let gz = dir.join("v.js.gz");
        let hash = dir.join("v.js.hash");
        std::fs::write(&src, b"version one").expect("write v1");

        // First build: no .gz yet, so nothing can be skipped.
        assert!(!should_skip_asset(&src, &gz, &hash));
        assert!(compress_asset(&src, &gz, &hash, "v.js"));

        // Second build, source untouched: skip.
        assert!(
            should_skip_asset(&src, &gz, &hash),
            "an unchanged source must be skipped"
        );

        // Source edited: must NOT be skipped.
        std::fs::write(&src, b"version two, quite different").expect("write v2");
        assert!(
            !should_skip_asset(&src, &gz, &hash),
            "an edited source must be reprocessed"
        );

        // Hash record lost: unknown means reprocess, never skip.
        std::fs::write(&src, b"version one").expect("restore v1");
        assert!(should_skip_asset(&src, &gz, &hash), "restored content");
        std::fs::remove_file(&hash).expect("drop the hash record");
        assert!(
            !should_skip_asset(&src, &gz, &hash),
            "a missing hash record is `not measured`, so the asset is rebuilt"
        );
    }

    #[test]
    fn a_missing_gz_or_source_is_never_skippable() {
        let dir = tmpdir();
        let src = dir.join("s.js");
        let gz = dir.join("s.js.gz");
        let hash = dir.join("s.js.hash");
        std::fs::write(&src, b"x").expect("write source");
        std::fs::write(&hash, "deadbeef").expect("write stale hash");

        assert!(!should_skip_asset(&src, &gz, &hash), "no .gz produced yet");
        std::fs::write(&gz, b"").expect("write gz");
        std::fs::remove_file(&src).expect("drop source");
        assert!(!should_skip_asset(&src, &gz, &hash), "source is gone");
    }

    #[test]
    fn read_stored_hash_trims_and_reports_a_missing_record_as_none() {
        let dir = tmpdir();
        let path = dir.join("h");
        assert_eq!(read_stored_hash(&path), None);
        assert!(write_hash_file(&path, "abc123"));
        std::fs::write(&path, "  abc123\n").expect("rewrite with whitespace");
        assert_eq!(read_stored_hash(&path), Some("abc123".to_string()));
    }

    #[cfg(feature = "standard-deps")]
    #[test]
    fn the_file_hash_is_content_addressed() {
        let dir = tmpdir();
        let a = dir.join("a");
        let b = dir.join("b");
        std::fs::write(&a, b"same").expect("write a");
        std::fs::write(&b, b"same").expect("write b");
        let ha = calculate_file_hash(&a).expect("hash a");
        assert_eq!(ha.len(), 64, "SHA256 renders as 64 hex characters");
        assert!(ha.chars().all(|c| c.is_ascii_hexdigit()));
        assert_eq!(ha, calculate_file_hash(&b).expect("hash b"));

        std::fs::write(&b, b"different").expect("rewrite b");
        assert_ne!(ha, calculate_file_hash(&b).expect("rehash b"));

        // An unreadable path is "not measured", not a hash of nothing.
        assert_eq!(calculate_file_hash(&dir.join("nope")), None);
    }

    #[test]
    fn an_unhashable_source_counts_as_changed() {
        let dir = tmpdir();
        let hash = dir.join("gone.hash");
        std::fs::write(&hash, "whatever").expect("write hash record");
        assert!(
            has_file_changed(&dir.join("gone"), &hash),
            "a source that cannot be read must be treated as changed"
        );
    }

    // ---- rerun-if-changed hygiene (CRUX-06) ------------------------------

    /// Every literal `cargo:rerun-if-changed=` path in `build.rs` must sit inside
    /// `CARGO_MANIFEST_DIR` and exist there. A declared-but-missing path makes
    /// cargo mark the build script permanently stale, so the whole crate
    /// relinks on every invocation — `../assets/demo/` did exactly that from the
    /// deleted `server/` layout until 3.36.0 (55 s / 265 CPU-s per no-op
    /// release build). The shape check runs BEFORE the existence check, so
    /// materialising a sibling directory next to the checkout cannot pass it.
    #[test]
    fn rerun_if_changed_paths_exist_inside_the_tree() {
        let manifest_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
        let build_rs = include_str!("build.rs");
        let literal: std::collections::BTreeSet<&str> = build_rs
            .lines()
            .filter_map(|l| l.split("cargo:rerun-if-changed=").nth(1))
            .filter_map(|rest| rest.split('"').next())
            .filter(|p| !p.contains('{') && !p.contains('$'))
            .collect();

        // Anti-vacuity: the extractor must still find the directives. A rotted
        // pattern must fail here rather than certify an empty set.
        assert!(
            literal.len() >= 8,
            "extractor found only {} distinct literal watches: {literal:?}",
            literal.len()
        );
        // The required watches, by NAME — a deleted watch cannot be padded back
        // with a duplicate of another, and the provenance watches on `.git/`
        // (which keep PMAT_GIT_SHA honest) cannot be dropped quietly.
        for req in [
            "assets/vendor/",
            "assets/demo/",
            "templates/",
            "src/schema/refactor_state.capnp",
            "contracts/binding.yaml",
            "mcp_tool_schemas",
            ".git/HEAD",
            ".git/index",
        ] {
            assert!(literal.contains(req), "build.rs no longer watches {req}");
        }

        let mut bad = Vec::new();
        for p in &literal {
            let path = std::path::Path::new(p);
            if path.is_absolute() || p.split('/').any(|seg| seg == "..") {
                bad.push(format!("escapes the manifest dir: {p}"));
            } else if !manifest_dir.join(path).exists() {
                bad.push(format!("missing: {p}"));
            }
        }
        assert!(
            bad.is_empty(),
            "rerun-if-changed defects in build.rs: {bad:?}"
        );

        // Exactly one interpolated directive is expected — the per-file schema
        // walk — and it must be manifest-relative before the `{}`.
        let dynamic: Vec<&str> = build_rs
            .lines()
            .filter(|l| {
                l.contains("cargo:rerun-if-changed=") && (l.contains('{') || l.contains('$'))
            })
            .collect();
        assert_eq!(
            dynamic.len(),
            1,
            "unexpected interpolated rerun-if-changed directives: {dynamic:?}"
        );
        assert!(
            dynamic[0].contains("path.display()"),
            "the one interpolated directive is not the schema walk: {}",
            dynamic[0]
        );
    }
}
