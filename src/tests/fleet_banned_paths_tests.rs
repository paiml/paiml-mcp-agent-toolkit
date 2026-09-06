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
    // PMAT-687: two files still carry the literals — the hardcoded-path
    // analyzer's own fixtures and one comment in check.rs. Both owe complexity
    // debt that `pmat verify` measures the moment a line in them changes
    // (classify: cognitive 33 > 25; check.rs's included helpers: 25 functions
    // over), so their scrub rides with that refactor. Until then the fleet
    // gate fails on them and the tag's prerelease is created by hand. The
    // debt is PINNED AS A COUNT (quorum lane 2 on #1204): it may only go down.
    const PINNED_DEBT: [(&str, usize); 2] = [
        ("src/services/hardcoded_paths.rs", 14),
        (
            "src/cli/handlers/comply_handlers/check_handlers/check.rs",
            1,
        ),
    ];
    let files = scanned_files(&root);
    assert!(
        files.len() > 100,
        "the listing must see the tree, saw {}",
        files.len()
    );
    let mut hits = Vec::new();
    let mut pinned_seen = 0usize;
    for file in &files {
        let Ok(text) = std::fs::read_to_string(file) else {
            continue;
        };
        let rel = file
            .strip_prefix(&root)
            .unwrap_or(file)
            .display()
            .to_string();
        let pin = PINNED_DEBT.iter().find(|(name, _)| *name == rel);
        let mut count = 0usize;
        for (n, line) in text.lines().enumerate() {
            for banned in BANNED {
                if line.contains(banned) {
                    count += 1;
                    if pin.is_none() {
                        hits.push(format!("{rel}:{}: {banned}", n + 1));
                    }
                }
            }
        }
        if let Some((name, allowed)) = pin {
            pinned_seen += 1;
            assert!(
                count <= *allowed,
                "{name}: {count} banned-path line(s), pinned at {allowed} — the debt may only go down (PMAT-687)"
            );
        }
    }
    assert_eq!(
        pinned_seen,
        PINNED_DEBT.len(),
        "every pinned file is still tracked"
    );
    assert!(
        hits.is_empty(),
        "the fleet gate would fail on {} line(s):\n{}",
        hits.len(),
        hits.join("\n")
    );
}
