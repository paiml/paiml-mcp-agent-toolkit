//! PMAT-686 — the fleet clean-room gate (paiml/.github unified-gate.yml,
//! "Banned path scan") greps every `*.rs`, `*.toml` and `*.sh` in the tree for
//! four machine-specific prefixes and fails the release gate on a hit. On the
//! v3.38.0 tag it hit 40 lines of pmat's own tree — mostly the hardcoded-path
//! analyzer's fixtures, plus script defaults pointing at one workstation — so
//! no pmat tag could ever pass the fleet gate. This test is that scan, in-repo,
//! so the tree cannot drift back.
//!
//! Registered from `cli/handlers/work_handlers/mod.rs` with `#[path]`.

use std::path::{Path, PathBuf};

// Built with `concat!` so this file carries none of the literals itself —
// the fleet gate would flag the test that mirrors it.
const BANNED: [&str; 4] = [
    concat!("/mnt/", "nvme-raid0"),
    concat!("/home/", "noah"),
    concat!("/home/", "ubuntu"),
    concat!("/tmp/", "clean-room"),
];

/// The files the fleet gate scans: every TRACKED `*.rs`, `*.toml`, `*.sh`
/// (`git grep … -- '*.rs' '*.toml' '*.sh' ':!CLAUDE.md'`). Tracked, not on
/// disk: a gitignored `.cargo/config.toml` or a scratch script is not shipped
/// and the gate never sees it.
fn scanned_files(root: &Path) -> Vec<PathBuf> {
    let listed = std::process::Command::new("git")
        .args([
            "-C",
            &root.display().to_string(),
            "ls-files",
            "-z",
            "--",
            "*.rs",
            "*.toml",
            "*.sh",
            ":!CLAUDE.md",
        ])
        .output();
    let listed = listed.expect("git ls-files runs (the fleet gate runs on a git checkout too)");
    assert!(
        listed.status.success(),
        "git ls-files failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    String::from_utf8_lossy(&listed.stdout)
        .split('\0')
        .filter(|s| !s.is_empty())
        .map(|s| root.join(s))
        .collect()
}

#[test]
fn fleet_banned_path_scan_is_clean() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    // PMAT-687: nothing is excluded and nothing is pinned. The analyzer's own
    // source once carried 14 of the banned literals (comments and fixtures);
    // its fixtures now name a user the fleet does not ban, and its doc comments
    // say `<user>`. The fleet gate scans this file like any other.
    let files = scanned_files(&root);
    assert!(
        files.len() > 100,
        "the listing must see the tree, saw {}",
        files.len()
    );
    let mut hits = Vec::new();
    for file in &files {
        // The fleet gate is `git grep`, which reads bytes; a file that is not
        // UTF-8 must not be skipped silently (quorum lane 1 on PMAT-687).
        let bytes = std::fs::read(file).expect("a tracked file is readable");
        let text = String::from_utf8_lossy(&bytes).into_owned();
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .display()
            .to_string();
        for (n, line) in text.lines().enumerate() {
            for banned in BANNED {
                if line.contains(banned) {
                    hits.push(format!("{rel}:{}: {banned}", n + 1));
                }
            }
        }
    }
    assert!(
        hits.is_empty(),
        "the fleet gate would fail on {} line(s):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
