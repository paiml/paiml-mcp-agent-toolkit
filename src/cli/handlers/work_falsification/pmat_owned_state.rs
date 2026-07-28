#![cfg_attr(coverage_nightly, coverage(off))]
//! PMAT-154: pmat-owned state detection for the github-sync check.
//!
//! `pmat work complete` writes to `.pmat-work/ledger.jsonl`, `.pmat/context.db`,
//! `.pmat/context.idx/*`, and `.pmat-metrics/commit-*.json` during its own
//! execution. Counting these paths as "uncommitted changes" causes the
//! github-sync falsifier to fail on retries — the self-dirty loop.
//!
//! This module enumerates the prefixes pmat is the sole author of, and exposes
//! `is_pmat_owned_state()` for the `test_github_sync()` filter. User-owned
//! files under look-alike paths (`.pmat-baseline.json`, `pmat/…`) are
//! preserved.

/// Directory names pmat owns outright, wherever they appear.
///
/// Matched as a path *segment*, not only as a prefix: porcelain paths are
/// relative to the repository root, so a project analysed inside a subdirectory
/// yields `sub/.pmat/context.db`, which a prefix test misses — pmat's own cache
/// then counted as the user's uncommitted work.
pub(crate) const PMAT_OWNED_DIRS: &[&str] = &[
    ".pmat",
    ".pmat-work",
    ".pmat-metrics",
    // `pmat qa` writes generated checklists here; one was committed and shipped
    // in the published crate before this was covered.
    ".pmat-qa",
];

/// Files pmat creates in user-owned locations and cannot clean up.
///
/// `RoadmapService` writes `<roadmap>.yaml.lock` on every load, including
/// reads. It cannot be unlinked on drop: the read lock is shared, so removing
/// it would race another holder.
pub(crate) const PMAT_OWNED_SUFFIXES: &[&str] = &[".yaml.lock"];

/// True if a `git status --porcelain -b` line refers to a pmat-owned state file.
///
/// Porcelain format is `XY PATH` (or `XY OLD -> NEW` for renames). The
/// destination path is what the working tree actually carries, so we inspect
/// it for rename lines.
pub(crate) fn is_pmat_owned_state(porcelain_line: &str) -> bool {
    let Some(after_status) = porcelain_line.get(3..) else {
        return false;
    };
    let path = after_status
        .rsplit(" -> ")
        .next()
        .unwrap_or(after_status)
        .trim();
    let path = unquote_porcelain_path(path);
    is_pmat_owned_path(path)
}

/// True if a repo-relative path is pmat-owned state rather than the user's work.
pub(crate) fn is_pmat_owned_path(path: &str) -> bool {
    if PMAT_OWNED_SUFFIXES.iter().any(|s| path.ends_with(s)) {
        return true;
    }
    // A directory match must consume a whole segment, so `.pmat-baseline.json`
    // and `pmat/context.db` stay the user's.
    path.split('/')
        .any(|segment| PMAT_OWNED_DIRS.contains(&segment))
        && path.contains('/')
}

/// Strip the quoting git applies to paths containing spaces or specials.
///
/// `git status --porcelain` renders such paths as `"…"` with C-style escapes.
/// Matching the prefix against the raw field meant the leading quote made every
/// quoted pmat path miss the filter, so `.pmat/some cache.db` counted as the
/// user's uncommitted work — a false failure of the very claim this filter
/// exists to keep honest.
pub(crate) fn unquote_porcelain_path(path: &str) -> &str {
    path.strip_prefix('"')
        .and_then(|p| p.strip_suffix('"'))
        .unwrap_or(path)
}

#[cfg(test)]
mod tests {
    use super::is_pmat_owned_state;

    #[test]
    fn matches_owned_dirs() {
        assert!(is_pmat_owned_state(" M .pmat/context.db"));
        assert!(is_pmat_owned_state(" M .pmat/context.idx/manifest.json"));
        assert!(is_pmat_owned_state(" M .pmat/project.toml"));
        assert!(is_pmat_owned_state("M  .pmat-work/ledger.jsonl"));
        assert!(is_pmat_owned_state("?? .pmat-work/PMAT-154/evidence.json"));
        assert!(is_pmat_owned_state(
            "A  .pmat-metrics/commit-abc123-meta.json"
        ));
    }

    #[test]
    fn skips_user_files() {
        assert!(!is_pmat_owned_state(" M src/lib.rs"));
        assert!(!is_pmat_owned_state("M  docs/roadmaps/roadmap.yaml"));
        assert!(!is_pmat_owned_state(" M Cargo.toml"));
        // Look-alike paths that aren't pmat-owned.
        assert!(!is_pmat_owned_state(" M .pmat-baseline.json"));
        assert!(!is_pmat_owned_state(" M pmat/context.db"));
    }

    #[test]
    fn rename_destination() {
        // `R  OLD -> NEW` — destination is what the worktree carries.
        assert!(is_pmat_owned_state(
            "R  src/old.rs -> .pmat-work/archived.rs"
        ));
        assert!(!is_pmat_owned_state("R  .pmat-work/old.rs -> src/new.rs"));
    }

    #[test]
    fn nested_and_qa_state_is_owned() {
        // Porcelain paths are repo-root-relative, so a project analysed in a
        // subdirectory yields `sub/.pmat/...`; a prefix test missed those and
        // counted pmat's own cache as the user's uncommitted work.
        assert!(is_pmat_owned_state(" M sub/.pmat/context.db"));
        assert!(is_pmat_owned_state(
            " M a/b/.pmat-metrics/commit-x-meta.json"
        ));
        assert!(is_pmat_owned_state("?? .pmat-qa/GH-102/checklist.yaml"));
        // The roadmap lock is created on every load and cannot be unlinked
        // safely (the read lock is shared), so it must never count as dirt.
        assert!(is_pmat_owned_state(" M docs/roadmaps/roadmap.yaml.lock"));
    }

    #[test]
    fn lookalikes_in_nested_paths_stay_user_owned() {
        assert!(!is_pmat_owned_state(" M sub/pmat/context.db"));
        assert!(!is_pmat_owned_state(" M sub/.pmat-baseline.json"));
        assert!(!is_pmat_owned_state(" M docs/roadmaps/roadmap.yaml"));
    }

    #[test]
    fn quoted_paths_are_still_recognised() {
        // git quotes paths containing spaces or specials. Matching the prefix
        // against the raw field made the leading `"` miss, so pmat's own cache
        // counted as the user's uncommitted work.
        assert!(is_pmat_owned_state(" M \".pmat/we ird.db\""));
        assert!(is_pmat_owned_state("?? \".pmat-work/PMAT-1/a b.json\""));
        assert!(is_pmat_owned_state(
            "R  \"src/o ld.rs\" -> \".pmat-work/n ew.rs\""
        ));
        // A quoted user path must still count.
        assert!(!is_pmat_owned_state(" M \"src/we ird.rs\""));
    }

    #[test]
    fn short_line_defense() {
        assert!(!is_pmat_owned_state(""));
        assert!(!is_pmat_owned_state("XY"));
        assert!(!is_pmat_owned_state("XY "));
    }

    #[test]
    fn full_github_sync_scenario() {
        // PMAT-154 full scenario: ledger + context + metrics files produced
        // by pmat itself must not inflate dirty_count.
        let status = concat!(
            "## master...origin/master\n",
            "M  .pmat-work/ledger.jsonl\n",
            " M .pmat/context.db\n",
            " M .pmat/context.idx/manifest.json\n",
            "A  .pmat-metrics/commit-abc123-meta.json\n",
            " M src/lib.rs\n",
            "?? untracked.txt"
        );
        let dirty_count = status
            .lines()
            .skip(1)
            .filter(|l| !l.is_empty() && !l.starts_with("??"))
            .filter(|l| !is_pmat_owned_state(l))
            .count();
        assert_eq!(
            dirty_count, 1,
            "Only user-owned src/lib.rs should count; pmat state excluded"
        );
    }
}
