//! Process-unique scratch files for atomic write-then-rename persistence.
//!
//! A fixed scratch name (e.g. `<db>.db.tmp`) lets two concurrent writers
//! build into the same scratch file and rename each other's half-built
//! output into place. Embedding the PID makes the scratch unique per
//! process — but scratch files orphaned by a crashed process then need
//! explicit sweeping, since no later invocation reuses their name.

use std::path::{Path, PathBuf};
use std::time::Duration;

/// Build the process-unique scratch path for `target`:
/// `<target_file_name>.tmp.<pid>` in the target's directory.
pub fn scratch_path(target: &Path) -> PathBuf {
    let file_name = target
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| "scratch".to_string());
    target.with_file_name(format!("{file_name}.tmp.{}", std::process::id()))
}

/// Remove scratch siblings of `target` (files named `<file_name>.tmp.*`)
/// whose mtime is at least `max_age` old.
///
/// Crash-orphaned scratch files are never reclaimed by their owning
/// process; callers sweep before (or after) each save. The age guard
/// protects a concurrent live writer, whose scratch is by definition
/// fresh. Best-effort: IO errors are ignored.
pub fn sweep_stale_scratch(target: &Path, max_age: Duration) {
    let Some(file_name) = target.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return;
    };
    let prefix = format!("{file_name}.tmp.");
    let Some(parent) = target.parent() else {
        return;
    };
    let dir = if parent.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent
    };
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with(&prefix) {
            continue;
        }
        let is_stale = entry
            .metadata()
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_some_and(|age| age >= max_age);
        if is_stale {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scratch_path_embeds_pid_and_target_name() {
        let p = scratch_path(Path::new("/some/dir/context.db"));
        let name = p.file_name().unwrap().to_string_lossy().to_string();
        assert!(name.starts_with("context.db.tmp."));
        assert!(name.ends_with(&std::process::id().to_string()));
        assert_eq!(p.parent().unwrap(), Path::new("/some/dir"));
    }

    #[test]
    fn test_sweep_removes_stale_scratch_but_not_target_or_unrelated() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("data.json");
        std::fs::write(&target, "{}").unwrap();
        let orphan = dir.path().join("data.json.tmp.99999");
        std::fs::write(&orphan, "half-written").unwrap();
        let unrelated = dir.path().join("other.json.tmp.99999");
        std::fs::write(&unrelated, "x").unwrap();

        // max_age zero treats every matching scratch as stale
        sweep_stale_scratch(&target, Duration::ZERO);

        assert!(!orphan.exists(), "stale scratch must be swept");
        assert!(target.exists(), "target itself must survive");
        assert!(unrelated.exists(), "other targets' scratch must survive");
    }

    #[test]
    fn test_sweep_spares_fresh_scratch() {
        let dir = tempfile::TempDir::new().unwrap();
        let target = dir.path().join("data.json");
        let fresh = dir.path().join("data.json.tmp.12345");
        std::fs::write(&fresh, "live writer").unwrap();

        sweep_stale_scratch(&target, Duration::from_secs(3600));

        assert!(fresh.exists(), "fresh scratch (live writer) must be spared");
    }
}
