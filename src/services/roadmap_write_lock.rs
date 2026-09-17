//! PMAT-1385 (#1385): `RoadmapWriteLock` — the proof that the repository's EXCLUSIVE
//! roadmap lock is held, and the only code that writes a path under `docs/roadmaps/`.
//!
//! `pmat work migrate` rewrote `roadmap.yaml` with a bare `std::fs::write` and no
//! lock, while every `RoadmapService` writer held one. Nothing could tell the two
//! apart: "holds the lock" was a comment on the writers that did, and the one that
//! did not compiled and shipped the same. Every write under `docs/roadmaps/` is now a
//! method of this type, and a value of it exists only while the flock is held, so a
//! write that compiles is a write made under the lock. What the type system cannot
//! say — that no code writes AROUND it with a raw `fs` call — the writer gate says:
//! `src/tests/roadmap_writer_gate_tests.rs` allows raw writes to a roadmap path in
//! this file's functions and nowhere else.

use std::fs::{File, OpenOptions};
use std::path::Path;

use anyhow::{Context, Result};
use fs2::FileExt;

/// The repository's exclusive roadmap lock, held for exactly as long as this value
/// lives. It cannot be cloned and has no public constructor other than taking the
/// lock; dropping it closes the file, which releases the lock.
#[derive(Debug)]
pub struct RoadmapWriteLock {
    file: File,
}

impl RoadmapWriteLock {
    /// Block until the exclusive lock at `lock_path` is taken.
    ///
    /// # Errors
    ///
    /// When the lock file cannot be opened or locked.
    pub(crate) fn acquire(lock_path: &Path) -> Result<Self> {
        let file = open_lock_file(lock_path)?;
        file.lock_exclusive()
            .with_context(|| format!("Failed to acquire exclusive lock: {:?}", lock_path))?;
        Ok(Self { file })
    }

    /// The lock file itself, which carries the id high-water mark (PMAT-673).
    pub(crate) fn file_mut(&mut self) -> &mut File {
        &mut self.file
    }

    /// Write `contents` over `path` in place — the bytes `std::fs::write` leaves.
    ///
    /// # Errors
    ///
    /// The I/O error of the write.
    pub fn write(&self, path: &Path, contents: impl AsRef<[u8]>) -> std::io::Result<()> {
        std::fs::write(path, contents)
    }

    /// Write `contents` to `staging`, then rename it over `path`.
    ///
    /// The lock keeps every pmat process out while this runs, but git, an editor and
    /// a CI gate take no such lock: a rename within one directory is atomic, so none
    /// of them can read half a file, and a crash cannot leave one. The staging file
    /// is removed when the rename fails.
    ///
    /// # Errors
    ///
    /// The I/O error of the write or of the rename.
    pub fn replace(
        &self,
        staging: &Path,
        path: &Path,
        contents: impl AsRef<[u8]>,
    ) -> std::io::Result<()> {
        std::fs::write(staging, contents)?;
        std::fs::rename(staging, path).inspect_err(|_| {
            let _ = std::fs::remove_file(staging);
        })
    }

    /// Create `dir` and its parents.
    ///
    /// A directory is not roadmap content, but `docs/roadmaps/entries/` is the
    /// switch that turns fragment mode on (PMAT-1363), so creating one is a roadmap
    /// write like any other and holds the same lock.
    ///
    /// # Errors
    ///
    /// The I/O error of the creation.
    pub fn create_dir_all(&self, dir: &Path) -> std::io::Result<()> {
        std::fs::create_dir_all(dir)
    }

    /// Delete `path`. `Ok(false)` when there was nothing to delete.
    ///
    /// # Errors
    ///
    /// Any I/O error other than absence.
    pub fn remove(&self, path: &Path) -> std::io::Result<bool> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }
}

/// Open (creating) a roadmap lock file, and its directory.
///
/// Shared by the exclusive lock above and `RoadmapService`'s shared read lock.
/// Opened `read(true)` and `truncate(false)` because the file carries the id
/// high-water mark, which truncating on open would destroy before the first read
/// (PMAT-673). It is the LOCK file — outside git the sibling `roadmap.yaml.lock` —
/// and never roadmap content.
///
/// # Errors
///
/// When the directory cannot be created or the file cannot be opened.
pub(crate) fn open_lock_file(lock_path: &Path) -> Result<File> {
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create lock directory: {:?}", parent))?;
    }
    OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .truncate(false)
        .open(lock_path)
        .with_context(|| format!("Failed to open lock file: {:?}", lock_path))
}
