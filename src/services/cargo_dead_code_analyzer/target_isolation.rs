// Target-directory isolation for the analyzer's `cargo check` (#1305)
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

/// The subdirectory of cargo's target directory the analyzer builds under.
pub(crate) const ISOLATED_TARGET_SUBDIR: &str = "pmat-dead-code";

/// The `--target-dir` the analyzer's `cargo check` uses for the crate at
/// `cargo_root`: a directory no other workspace root can build into.
///
/// #1305 (duplicate report #1284). cargo fingerprints a workspace member by
/// its path RELATIVE to the workspace root, so two different roots holding a
/// package of the same name and version share one fingerprint in any target
/// directory they share — and it decides freshness by mtime. A source older
/// than the other root's last check is "fresh": cargo replays THAT root's
/// diagnostics and exits 0 without compiling a line. The analyzer inherited
/// whatever target directory the environment named (`CARGO_TARGET_DIR`,
/// `build.target-dir`), so an uncompilable crate was reported as a clean,
/// full measurement whenever a same-named crate had been checked more
/// recently into the same place: `ci / test` and `ci / coverage` mount one
/// per-PR `/workspace/target`, and every worktree of one repository behind a
/// shared `CARGO_TARGET_DIR` has every package name in common.
///
/// The directory still lives inside the target directory cargo resolves for
/// the crate, so build output stays where the user put it and `cargo clean`
/// removes it; it is keyed on the canonical workspace root, so the fingerprints
/// in it can only ever have come from this root.
pub(crate) fn isolated_target_dir(cargo_root: &Path) -> PathBuf {
    let (workspace_root, target_directory) = cargo_layout(cargo_root)
        .unwrap_or_else(|| (cargo_root.to_path_buf(), cargo_root.join("target")));
    isolated_target_dir_in(&target_directory, &workspace_root)
}

/// `<target_directory>/pmat-dead-code/<workspace root key>` — the pure half of
/// [`isolated_target_dir`], so the one property that matters (distinct roots
/// never share a directory) is testable without running cargo.
pub(crate) fn isolated_target_dir_in(target_directory: &Path, workspace_root: &Path) -> PathBuf {
    target_directory
        .join(ISOLATED_TARGET_SUBDIR)
        .join(workspace_root_key(workspace_root))
}

/// A directory name for a workspace root: its last component (readable, at
/// most 32 filename-safe characters) and the 64-bit FNV-1a hash of its
/// canonical path. FNV-1a rather than `DefaultHasher`, whose output the
/// standard library does not promise to keep across releases — a key that
/// moved on a toolchain bump would orphan every existing directory.
pub(crate) fn workspace_root_key(workspace_root: &Path) -> String {
    let canonical = absolutize(workspace_root);
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in canonical.as_os_str().as_encoded_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    let name: String = canonical
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .take(32)
        .collect();
    format!("{name}-{hash:016x}")
}

/// The workspace root and target directory cargo resolves for `cargo_root`,
/// honouring `CARGO_TARGET_DIR` and `build.target-dir`; `None` when
/// `cargo metadata` cannot answer (a manifest cargo cannot read — the check
/// that follows fails on it anyway).
fn cargo_layout(cargo_root: &Path) -> Option<(PathBuf, PathBuf)> {
    let output = Command::new("cargo")
        .current_dir(cargo_root)
        .args(["metadata", "--no-deps", "--format-version", "1"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let meta: Value = serde_json::from_slice(&output.stdout).ok()?;
    let workspace_root = meta["workspace_root"].as_str()?;
    let target_directory = meta["target_directory"].as_str()?;
    Some((
        PathBuf::from(workspace_root),
        PathBuf::from(target_directory),
    ))
}
