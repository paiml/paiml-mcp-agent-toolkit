#![cfg_attr(coverage_nightly, coverage(off))]
//! The single implementation of "`--clear-cache` / `--force-refresh` empties a
//! cache directory".
//!
//! Two commands advertise a cache-clearing flag and both used to fake it:
//!
//! * `enforce extreme --clear-cache --cache-dir DIR` printed
//!   "🧹 Clearing cache at: DIR" and then ran
//!   `// In real implementation, would clear cache` — the directory's contents
//!   survived, so the message was a lie about work never done.
//! * `analyze incremental-coverage --force-refresh` printed
//!   "🧹 Clearing coverage cache..." above the same comment, and on the wired
//!   route printed nothing at all.
//!
//! Both now call [`clear_cache_directory`], which really deletes and reports
//! what it deleted, and returns an error rather than shrugging when it cannot:
//! a stale cache silently left in place is exactly the state the flag exists to
//! escape.

use anyhow::{Context, Result};
use std::path::Path;

/// Outcome of clearing one cache directory. Counts describe entries actually
/// removed, so "0 removed" and "5 removed" are distinguishable in output.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CacheClearOutcome {
    /// Entries (files or subdirectories) deleted from the directory.
    pub entries_removed: usize,
    /// True when the directory did not exist, so there was nothing to clear.
    pub was_absent: bool,
}

/// Delete every entry inside `cache_dir`, keeping the directory itself.
///
/// A missing directory is not an error (there is nothing cached), but it is
/// reported as `was_absent` so callers never print "cleared" for it. A path
/// that exists and is not a directory IS an error: silently treating a file
/// named as a cache directory as "nothing to do" is how a stale cache survives
/// the flag meant to remove it.
///
/// # Errors
///
/// Returns an error when `cache_dir` exists but is not a directory, when it
/// cannot be read, or when an entry cannot be removed.
pub fn clear_cache_directory(cache_dir: &Path) -> Result<CacheClearOutcome> {
    if !cache_dir.exists() {
        return Ok(CacheClearOutcome {
            entries_removed: 0,
            was_absent: true,
        });
    }

    if !cache_dir.is_dir() {
        anyhow::bail!(
            "cache path {} is not a directory; refusing to clear it",
            cache_dir.display()
        );
    }

    let mut entries_removed = 0usize;
    let read = std::fs::read_dir(cache_dir)
        .with_context(|| format!("cannot read cache directory {}", cache_dir.display()))?;

    for entry in read {
        let entry =
            entry.with_context(|| format!("cannot list cache entry in {}", cache_dir.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("cannot stat cache entry {}", path.display()))?;

        if file_type.is_dir() {
            std::fs::remove_dir_all(&path)
                .with_context(|| format!("cannot remove cache directory {}", path.display()))?;
        } else {
            std::fs::remove_file(&path)
                .with_context(|| format!("cannot remove cache file {}", path.display()))?;
        }
        entries_removed += 1;
    }

    Ok(CacheClearOutcome {
        entries_removed,
        was_absent: false,
    })
}

/// Clear `cache_dir` and print exactly what happened.
///
/// Every branch prints something: a flag whose only visible effect depends on
/// whether the user also passed `--cache-dir` reads as "did nothing" in the
/// common case, which is the defect this replaces.
///
/// # Errors
///
/// Propagates the errors of [`clear_cache_directory`].
pub fn clear_cache_directory_reporting(cache_dir: &Path, label: &str) -> Result<CacheClearOutcome> {
    let outcome = clear_cache_directory(cache_dir)?;

    if outcome.was_absent {
        eprintln!(
            "🧹 {label}: {} does not exist — nothing cached to clear",
            cache_dir.display()
        );
    } else {
        eprintln!(
            "🧹 {label}: removed {} entr{} from {}",
            outcome.entries_removed,
            if outcome.entries_removed == 1 {
                "y"
            } else {
                "ies"
            },
            cache_dir.display()
        );
    }

    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clear_removes_files_and_subdirectories() {
        let dir = tempfile::tempdir().expect("tempdir");
        std::fs::write(dir.path().join("entry.bin"), b"stale").expect("write entry");
        std::fs::create_dir(dir.path().join("sub")).expect("mkdir sub");
        std::fs::write(dir.path().join("sub/nested.bin"), b"stale").expect("write nested");

        let outcome = clear_cache_directory(dir.path()).expect("clear");

        assert_eq!(outcome.entries_removed, 2);
        assert!(!outcome.was_absent);
        assert!(
            !dir.path().join("entry.bin").exists(),
            "--clear-cache printed that it cleared the cache; the entry must be gone"
        );
        assert!(!dir.path().join("sub").exists());
        assert!(dir.path().is_dir(), "the cache directory itself is kept");
    }

    #[test]
    fn test_absent_directory_is_reported_not_cleared() {
        let dir = tempfile::tempdir().expect("tempdir");
        let missing = dir.path().join("never-created");

        let outcome = clear_cache_directory(&missing).expect("clear");

        assert!(outcome.was_absent);
        assert_eq!(outcome.entries_removed, 0);
    }

    #[test]
    fn test_non_directory_path_is_an_error() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("not-a-dir");
        std::fs::write(&file, b"x").expect("write");

        let err = clear_cache_directory(&file).expect_err("a file is not a cache directory");
        assert!(
            err.to_string().contains("not a directory"),
            "unexpected error: {err}"
        );
        assert!(file.exists(), "the file must not be deleted");
    }
}
