// The one thing that makes "read-only analysis" true of the lockfile.
// Included from cargo_dead_code_analyzer.rs - shares parent module scope

/// The analysed project's `Cargo.lock` as it stood before cargo was allowed
/// near it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LockfileBefore {
    /// There was no lockfile. Whether a tree carries one is the project's
    /// decision — a library deliberately omits it, a binary deliberately
    /// commits it — so creating one is a change of policy, not a detail.
    Absent,
    /// There was a lockfile, holding exactly these bytes.
    Present(Vec<u8>),
    /// The path EXISTED and could not be read, so there are no bytes to put
    /// back.
    ///
    /// Recorded apart from `Absent` because the two are opposites and
    /// `fs::read` cannot tell them apart. Collapsed into `Absent`, an
    /// unreadable lockfile is "not there before, there after" at restore time,
    /// and the guard DELETES the project's own file -- silent data loss
    /// committed by the mechanism whose whole purpose is to leave the tree
    /// alone. Nothing this analysis never saw may be removed by it.
    PresentUnreadable,
}

/// What the guard had to do, reported so the caller can react rather than
/// assume.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum LockfileRestore {
    /// cargo left the lockfile exactly as it found it. The overwhelmingly
    /// common case, and the only one in which nothing was written at all.
    Untouched,
    /// cargo changed or created it, and the original state was put back
    /// byte-for-byte.
    Restored,
    /// cargo changed or created it and the change could NOT be undone. The
    /// analysis has modified the tree it was only asked to measure.
    Failed(String),
}

/// Restores the analysed project's `Cargo.lock` to the state it was in before
/// `cargo check` ran.
///
/// # Why this exists rather than `--locked`
///
/// `cargo check` is the only layer that finds dead code nobody admitted was
/// dead, and cargo is free to rewrite the lockfile while doing it. The obvious
/// defence, `--locked`, was added, MEASURED and reverted in 2bdc6b90c: cargo
/// then REFUSES on any repo whose lockfile is absent or stale, and the compiler
/// layer silently disappears with it (80 dead functions -> 0 on the
/// differential corpus). Trading "writes a Cargo.lock" for "reports no dead
/// code on most libraries" is the worse deal, so #1076 stayed open.
///
/// So the write is permitted and then UNDONE. That keeps the scan at full
/// fidelity and still leaves the tree byte-identical.
///
/// # Why an ambient `[patch]` is the general case, not a clean-room quirk
///
/// The rewrite was found because the clean room installs a publish overlay —
/// one `[patch.crates-io]` entry per workspace member — into
/// `$CARGO_HOME/config.toml`, where EVERY child cargo process inherits it
/// whatever its cwd (infra#653). A `cargo check` on an unrelated crate then
/// records the unused patch by appending `[[patch.unused]]` to that crate's
/// lockfile. Measured identically on cargo 1.98.0 and on the container's
/// 1.95, so this is not a toolchain difference: an ambient `[patch]` is just
/// one of several resolutions that legitimately differ from the bytes on disk
/// (a stale lockfile, a different lockfile `version`, a `[replace]`). Cargo
/// writing the file is correct behaviour; pmat leaving the write behind is not.
///
/// # Coverage
///
/// `restore` is called explicitly on every return from `run_cargo_check`,
/// including the deadline path — which kills and reaps the child before it
/// returns, so no cargo process is still running when the bytes go back. The
/// `Drop` impl is the belt for a panic between the two. It is best effort: a
/// panic that skips `wait_for_cargo_check`'s own kill would leave cargo running
/// while `Drop` restores, and a `SIGKILL` of pmat itself reaches no destructor
/// at all. Those two are out of reach of any in-process mechanism — including
/// `--locked`, which buys them only by not scanning.
pub(crate) struct LockfileGuard {
    /// The lockfile cargo will actually write: the WORKSPACE root's, which for
    /// a workspace member is above `cargo_root`.
    path: PathBuf,
    before: LockfileBefore,
    /// False once `restore` has run, so `Drop` does not do it twice.
    armed: bool,
}

impl LockfileGuard {
    /// Snapshot the lockfile. Call this BEFORE the cargo child is spawned.
    pub(crate) fn acquire(cargo_root: &Path) -> Self {
        let path = workspace_lockfile_path(cargo_root);
        // A read error is never propagated: refusing to analyse because a
        // lockfile could not be READ would be the `--locked` mistake in a new
        // costume. But it is not flattened into `Absent` either -- presence and
        // readability are different questions, and `fs::read` only answers the
        // second. `Present` is recorded only when the bytes are in hand, so the
        // restore can never write a guess.
        let before = match std::fs::read(&path) {
            Ok(bytes) => LockfileBefore::Present(bytes),
            Err(_) if path.exists() => LockfileBefore::PresentUnreadable,
            Err(_) => LockfileBefore::Absent,
        };
        Self {
            path,
            before,
            armed: true,
        }
    }

    /// Put the lockfile back and say what that took. Disarms the `Drop` belt.
    pub(crate) fn restore(mut self) -> LockfileRestore {
        let outcome = self.restore_inner();
        self.armed = false;
        outcome
    }

    /// The path this guard is protecting.
    #[cfg(test)]
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }

    fn restore_inner(&self) -> LockfileRestore {
        // Presence and content are asked separately on purpose: a path that
        // EXISTS but cannot be read (it became a directory, its permissions
        // changed) reads as absent through `fs::read` alone, and the guard
        // would then report `Untouched` over something it had not looked at.
        let now_exists = self.path.exists();
        let now = std::fs::read(&self.path).ok();
        match &self.before {
            LockfileBefore::Present(before) => self.restore_present(before, now.as_deref()),
            LockfileBefore::PresentUnreadable => self.restore_unreadable(now_exists),
            LockfileBefore::Absent => self.restore_absent(now_exists),
        }
    }

    /// There were bytes before. There must be exactly those bytes after.
    ///
    /// `now` is `None` when the file is unreadable NOW, which is not equal to
    /// `before` and therefore correctly routes to `put_back`.
    fn restore_present(&self, before: &[u8], now: Option<&[u8]>) -> LockfileRestore {
        if now == Some(before) {
            // Unchanged: touch nothing. Rewriting identical bytes would still
            // move the mtime, and every "is the tree clean" gate downstream
            // reads more than content.
            LockfileRestore::Untouched
        } else {
            self.put_back(before)
        }
    }

    /// It was there and could not be read, so there is nothing to put back.
    ///
    /// Still there: leave it exactly alone -- no bytes to compare, none to
    /// write, and it is the project's file, not ours to remove. Gone: this
    /// analysis destroyed something it could not snapshot, and must say so.
    fn restore_unreadable(&self, now_exists: bool) -> LockfileRestore {
        if now_exists {
            LockfileRestore::Untouched
        } else {
            self.failed("it existed but could not be read, and is now gone".into())
        }
    }

    /// There was no lockfile. If cargo made one, take it away again: whether a
    /// tree carries a lockfile is the project's decision (#1076).
    fn restore_absent(&self, now_exists: bool) -> LockfileRestore {
        if now_exists {
            self.take_away()
        } else {
            LockfileRestore::Untouched
        }
    }

    /// Write the snapshot back over whatever cargo left, and VERIFY it: a
    /// restore that silently did nothing is the same shape as the bug.
    fn put_back(&self, before: &[u8]) -> LockfileRestore {
        if let Err(e) = std::fs::write(&self.path, before) {
            return self.failed(format!("the original bytes could not be written back: {e}"));
        }
        match std::fs::read(&self.path) {
            Ok(after) if after == before => LockfileRestore::Restored,
            Ok(_) => self.failed("the original bytes went in and different ones came back".into()),
            Err(e) => self.failed(format!("the original bytes went in and cannot be read: {e}")),
        }
    }

    /// Remove the lockfile cargo generated, and VERIFY it is gone.
    fn take_away(&self) -> LockfileRestore {
        match std::fs::remove_file(&self.path) {
            Ok(()) if !self.path.exists() => LockfileRestore::Restored,
            Ok(()) => self.failed("cargo's lockfile was removed and is still there".into()),
            Err(e) => self.failed(format!("cargo generated it and it cannot be removed: {e}")),
        }
    }

    /// Every failure carries the stable token and the path, from one place, so
    /// a consumer never has to match on which branch produced it.
    fn failed(&self, what: String) -> LockfileRestore {
        LockfileRestore::Failed(format!(
            "{}: {}: {what}",
            crate::models::dead_code::COMPILER_SCAN_REASON_LOCKFILE_RESTORE_FAILED,
            self.path.display()
        ))
    }
}

impl Drop for LockfileGuard {
    fn drop(&mut self) {
        if self.armed {
            let _ = self.restore_inner();
        }
    }
}

/// The lockfile `cargo check` run in `cargo_root` would write.
///
/// Cargo keeps ONE lockfile per workspace, at the workspace root — which for a
/// member crate is an ancestor of `cargo_root`. Guarding `cargo_root/Cargo.lock`
/// alone would leave every workspace member unprotected while looking correct,
/// so the root is asked of cargo itself.
///
/// `locate-project` parses manifests and performs no dependency resolution, so
/// it cannot write the very file it is here to find (measured: it leaves a
/// lockfile-less crate lockfile-less). If it cannot answer — no cargo on PATH,
/// an unreadable manifest — the guard falls back to `cargo_root`, which is
/// where a single-crate project's lockfile lives.
fn workspace_lockfile_path(cargo_root: &Path) -> PathBuf {
    let manifest = Command::new("cargo")
        .current_dir(cargo_root)
        .args(["locate-project", "--workspace", "--message-format", "plain"])
        .output()
        .ok()
        .filter(|out| out.status.success())
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .filter(|s| !s.is_empty())
        .map(PathBuf::from);

    match manifest.as_ref().and_then(|m| m.parent()) {
        Some(root) => root.join("Cargo.lock"),
        None => cargo_root.join("Cargo.lock"),
    }
}
