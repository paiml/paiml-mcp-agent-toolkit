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
    // PMAT-687: the fleet scan excludes exactly one pmat file, the
    // hardcoded-path analyzer's own source (paiml/.github#65 — its recognisers
    // and fixtures name the strings the scan bans). Everything else is scanned,
    // and the tree must be clean: the debt pinned here at 3.39.0 (14 lines in
    // the analyzer, one comment in check.rs) is paid or excluded.
    const FLEET_EXCLUDED: [&str; 1] = ["src/services/hardcoded_paths.rs"];
    let files = scanned_files(&root);
    assert!(
        files.len() > 100,
        "the listing must see the tree, saw {}",
        files.len()
    );
    let mut hits = Vec::new();
    let mut excluded_seen = 0usize;
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .display()
            .to_string();
        if FLEET_EXCLUDED.contains(&rel.as_str()) {
            excluded_seen += 1;
            continue;
        }
        for (n, line) in text.lines().enumerate() {
            for banned in BANNED {
                if line.contains(banned) {
                    hits.push(format!("{rel}:{}: {banned}", n + 1));
                }
            }
        }
    }
    assert_eq!(
        excluded_seen,
        FLEET_EXCLUDED.len(),
        "every fleet-excluded file is still tracked (an exclusion for a file that no longer exists is dead)"
    );
    assert!(
        hits.is_empty(),
        "the fleet gate would fail on {} line(s):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
