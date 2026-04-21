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

/// Prefixes of pmat-owned state paths to ignore when counting dirty files.
pub(crate) const PMAT_OWNED_STATE_PREFIXES: &[&str] = &[".pmat/", ".pmat-work/", ".pmat-metrics/"];

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
    PMAT_OWNED_STATE_PREFIXES
        .iter()
        .any(|p| path.starts_with(p))
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
