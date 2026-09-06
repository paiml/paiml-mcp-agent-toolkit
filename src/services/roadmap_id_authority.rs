//! ONE id authority per repository, not per checkout.
//!
//! PMAT-680 (#1193). PMAT-673 closed the two-processes-one-file race by minting
//! under a single exclusive lock, and PMAT-679 made the write an append. Both
//! reasoned about ONE checkout: the id came from `max(this checkout's roadmap
//! text, this checkout's `roadmap.yaml.lock`) + 1`, under a lock on that
//! sibling file. A repository is not one checkout. Two worktrees, or two
//! clones, each hold their own roadmap and their own sibling lock, so they
//! mint the SAME id by construction — and an id already spent on another
//! branch is invisible to both.
//!
//! So the authority moves off the working tree and onto the repository:
//!
//! * the lock and its high-water mark live in the **git common directory**
//!   (`<common-dir>/pmat/roadmap-id.lock`), which every worktree of a
//!   repository shares — one lock, one mark, whichever checkout is minting;
//! * the mint also reads **every ref's** copy of the roadmap
//!   ([`IdAuthority::max_id_across_refs`]), so an id spent on a branch this
//!   checkout has never seen is still spent.
//!
//! Outside a git repository there is nothing to share, so the sibling
//! `<roadmap>.yaml.lock` remains the authority exactly as before.

use crate::services::roadmap_text;
use anyhow::{Context, Result};
use std::collections::BTreeSet;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

/// The lock file's name under `<git-common-dir>/pmat/`.
const LOCK_FILE_NAME: &str = "roadmap-id.lock";

/// The repository a roadmap belongs to, as git describes it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitRepo {
    /// `--git-common-dir`: the directory EVERY worktree of this repository
    /// shares. Both the lock and the refs live here.
    pub common_dir: PathBuf,
    /// `--show-toplevel`: the working tree this roadmap was reached through.
    pub toplevel: PathBuf,
    /// The roadmap's path relative to `toplevel` — the path a ref's tree is
    /// asked for the roadmap by.
    pub roadmap_rel: PathBuf,
}

/// Where a roadmap's ids are minted from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IdAuthority {
    /// The lock file to hold while minting, and the file the high-water mark
    /// is persisted in. Shared by every checkout inside a repository.
    pub lock_path: PathBuf,
    /// `None` when the roadmap is not inside a git repository — then there is
    /// exactly one checkout by definition, and the sibling lock is enough.
    pub repo: Option<GitRepo>,
}

impl IdAuthority {
    /// Resolve the authority for `roadmap_path`.
    ///
    /// One `git rev-parse` decides it. Inside a repository the lock is
    /// `<git-common-dir>/pmat/roadmap-id.lock` — the common dir, not the
    /// worktree's `.git`, because that is the one directory every worktree
    /// shares. On any failure (no git, no repository, an uncreatable
    /// directory) the answer is the sibling `<roadmap>.yaml.lock` this code
    /// used before PMAT-680, so a project outside git behaves exactly as it
    /// always has.
    #[must_use]
    pub fn discover(roadmap_path: &Path) -> Self {
        Self::in_git(roadmap_path).unwrap_or_else(|| Self {
            lock_path: sibling_lock_path(roadmap_path),
            repo: None,
        })
    }

    /// The authority inside a git repository, or `None` if there is not one.
    fn in_git(roadmap_path: &Path) -> Option<Self> {
        let (anchor, tail) = anchor_and_tail(roadmap_path)?;
        let text = git_stdout(
            &anchor,
            &[
                "rev-parse",
                "--git-common-dir",
                "--show-toplevel",
                "--show-prefix",
            ],
        )?;
        let mut lines = text.lines();
        let common_dir = absolutize(&anchor, Path::new(lines.next()?));
        let toplevel = PathBuf::from(lines.next()?);
        let prefix = lines.next().unwrap_or("");

        let lock_dir = common_dir.join("pmat");
        std::fs::create_dir_all(&lock_dir).ok()?;

        Some(Self {
            lock_path: lock_dir.join(LOCK_FILE_NAME),
            repo: Some(GitRepo {
                common_dir,
                toplevel,
                roadmap_rel: Path::new(prefix.trim_end_matches('/')).join(tail),
            }),
        })
    }

    /// The greatest id number this roadmap carries on ANY ref of the
    /// repository — local branches, remote-tracking branches and `HEAD` —
    /// or `None` outside a repository, and when no ref carries the file.
    ///
    /// This is the half of the defect a lock cannot fix: a branch that spent
    /// PMAT-020 is not in this checkout's file, so minting from the file alone
    /// spends PMAT-020 twice. Work is bounded by DISTINCT blobs: hundreds of
    /// refs sharing one roadmap cost one `cat-file`.
    #[must_use]
    pub fn max_id_across_refs(&self) -> Option<u32> {
        let repo = self.repo.as_ref()?;
        let rel = repo.roadmap_rel.to_str()?;

        let mut commits: BTreeSet<String> = BTreeSet::new();
        commits.insert("HEAD".to_string());
        if let Some(refs) = git_stdout(
            &repo.toplevel,
            &[
                "for-each-ref",
                "--format=%(objectname)",
                "refs/heads",
                "refs/remotes",
            ],
        ) {
            commits.extend(non_empty_lines(&refs));
        }

        let mut blobs: BTreeSet<String> = BTreeSet::new();
        for commit in &commits {
            let spec = format!("{commit}:{rel}");
            // `--quiet` so a ref that simply has no roadmap says nothing.
            if let Some(blob) =
                git_stdout(&repo.toplevel, &["rev-parse", "--verify", "--quiet", &spec])
            {
                blobs.extend(non_empty_lines(&blob));
            }
        }

        let mut max: Option<u32> = None;
        for blob in &blobs {
            let Some(text) = git_stdout(&repo.toplevel, &["cat-file", "-p", blob]) else {
                continue;
            };
            if let Some(found) = roadmap_text::max_id_number(&text) {
                max = Some(max.map_or(found, |seen: u32| seen.max(found)));
            }
        }
        max
    }
}

/// The pre-PMAT-680 authority: `<roadmap>.yaml.lock` beside the roadmap.
fn sibling_lock_path(roadmap_path: &Path) -> PathBuf {
    let mut lock_path = roadmap_path.to_path_buf();
    lock_path.set_extension("yaml.lock");
    lock_path
}

/// The nearest existing ancestor directory of `roadmap_path`, and the path of
/// the roadmap below it.
///
/// `git -C` needs a directory that exists, and the roadmap's own directory may
/// not yet (`pmat work init` writes into a fresh `docs/roadmaps/`). Climbing
/// keeps `work init` inside the repository's authority instead of quietly
/// falling back to a sibling lock.
fn anchor_and_tail(roadmap_path: &Path) -> Option<(PathBuf, PathBuf)> {
    let mut tail = PathBuf::from(roadmap_path.file_name()?);
    let mut dir = parent_dir(roadmap_path);
    loop {
        if dir.is_dir() {
            return Some((dir, tail));
        }
        let name = dir.file_name()?;
        tail = Path::new(name).join(tail);
        dir = parent_dir(&dir);
    }
}

/// `path`'s parent, with the empty path spelled `.` — `Path::parent` returns
/// `""` for a bare file name, and `git -C ""` is not the current directory.
fn parent_dir(path: &Path) -> PathBuf {
    match path.parent() {
        Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
        _ => PathBuf::from("."),
    }
}

/// `--git-common-dir` answers relative to the directory git ran in (`.git`),
/// or absolutely for a linked worktree. Both must end up absolute, because the
/// lock is opened from wherever the process happens to be.
fn absolutize(anchor: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        anchor.join(path)
    }
}

fn non_empty_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(ToString::to_string)
        .collect()
}

/// `git -C <dir> <args>`, or `None` when git is missing, the command failed,
/// or its output is not text. Every caller here treats all three the same way:
/// there is nothing to learn, so nothing is claimed.
fn git_stdout(dir: &Path, args: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    String::from_utf8(output.stdout).ok()
}

/// The id high-water mark persisted in an already-held lock file, if it holds
/// one. A fresh (or hand-emptied) lock file simply has no opinion.
///
/// PMAT-673 wrote this beside the roadmap; PMAT-680 writes it in the git
/// common dir, so the mark is now shared by every checkout. The file format —
/// the decimal number and nothing else — is unchanged.
pub fn high_water_mark(lock: &mut File) -> Option<u32> {
    lock.seek(SeekFrom::Start(0)).ok()?;
    let mut text = String::new();
    lock.read_to_string(&mut text).ok()?;
    text.trim().parse::<u32>().ok()
}

/// Record the id just minted in the already-held lock file.
///
/// # Errors
///
/// When the lock file cannot be rewound, truncated, written or flushed.
pub fn write_high_water_mark(lock: &mut File, next: u32) -> Result<()> {
    lock.seek(SeekFrom::Start(0))
        .with_context(|| "Failed to rewind roadmap lock file")?;
    lock.set_len(0)
        .with_context(|| "Failed to clear roadmap lock file")?;
    write!(lock, "{next}").with_context(|| "Failed to write roadmap id high-water mark")?;
    lock.flush()
        .with_context(|| "Failed to flush roadmap lock file")?;
    Ok(())
}
