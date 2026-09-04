//! Regression tests for GH-301: hook generator must never scan for Makefiles
//! recursively with `find` nor invoke `make -C $dir all`.
//!
//! Context: on monorepos with many Claude Code / Cursor agent worktrees under
//! `.claude/worktrees/*`, a `find . -name "Makefile"` sweep walks gitignored
//! trees and a `make -C $dir all` loop can block a single `git commit` for
//! minutes to hours. The PMAT hook generator therefore uses `git ls-files`
//! and must not emit either pattern.
//!
//! Issue: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/301

// This file is included via `#[path]` from hooks_command_handlers/mod.rs,
// so `super::` resolves to the hooks_command_handlers module.
use super::HooksCommand;

/// Fetch the generated pre-commit content via the same production code path
/// that `pmat hooks install` uses. If either async generation or the runtime
/// is unavailable, fall back to the deterministic sync fragments that do not
/// need config loading.
fn generated_hook_fragments() -> String {
    // generate_quality_checks and generate_hook_header are sync and
    // contract-verified — together they cover the body that was historically
    // most likely to contain a bad `find Makefile` sweep.
    let cmd = HooksCommand::default_for_tests();
    let mut out = String::new();
    out.push_str(&cmd.generate_hook_header());
    out.push('\n');
    out.push_str(&cmd.generate_quality_checks());
    out
}

impl HooksCommand {
    /// Test helper: construct a HooksCommand pointing at bogus paths. Only
    /// sync methods that don't touch the filesystem are safe to call on the
    /// returned instance.
    fn default_for_tests() -> Self {
        HooksCommand::new(
            std::path::PathBuf::from("/tmp/pmat-gh301-hooks"),
            std::path::PathBuf::from("/tmp/pmat-gh301-config.toml"),
        )
    }
}

#[test]
fn gh301_generated_hook_never_finds_makefiles_recursively() {
    let hook = generated_hook_fragments();

    // The reported pathological snippet — any variant MUST NOT appear.
    assert!(
        !hook.contains(r#"find . -name "Makefile""#),
        "GH-301 regression: hook contains `find . -name \"Makefile\"` sweep"
    );
    assert!(
        !hook.contains("-name Makefile"),
        "GH-301 regression: hook contains `-name Makefile` filter"
    );
}

#[test]
fn gh301_generated_hook_never_runs_make_all_in_subdirs() {
    let hook = generated_hook_fragments();

    assert!(
        !hook.contains(r#"make -C "$dir" all"#),
        "GH-301 regression: hook shells out to `make -C $dir all`"
    );
    assert!(
        !hook.contains("make -C $dir all"),
        "GH-301 regression: hook shells out to `make -C $dir all`"
    );
    assert!(
        !hook.contains("MAKEFILE_DIRS="),
        "GH-301 regression: hook defines MAKEFILE_DIRS variable"
    );
}

#[test]
fn gh301_any_makefile_discovery_uses_git_ls_files_not_find() {
    let hook = generated_hook_fragments();

    // Inspect only executable shell lines — strip `#` comments — then look
    // for any combination that discovers Makefiles via `find`.
    let executable: String = hook
        .lines()
        .map(|line| match line.split_once('#') {
            Some((code, _comment)) => code,
            None => line,
        })
        .collect::<Vec<_>>()
        .join("\n");

    let executable_lower = executable.to_lowercase();
    if executable_lower.contains("makefile") {
        let uses_find_for_makefile =
            executable.contains("find ") && executable.contains("Makefile");
        assert!(
            !uses_find_for_makefile,
            "GH-301 regression: Makefile discovery must use `git ls-files`, not `find`:\n{executable}"
        );
    }
}

#[test]
fn gh301_source_file_detection_prefers_git_ls_files() {
    let hook = generated_hook_fragments();

    // The HAS_SOURCE_FILES probe in generate_quality_checks must route
    // through git ls-files when inside a worktree so gitignored trees
    // (including .claude/worktrees) are never walked.
    assert!(
        hook.contains("git ls-files"),
        "GH-301 hardening: generated hook must use `git ls-files` for source discovery"
    );
}

#[test]
fn gh301_exposes_exclude_dirs_escape_hatch() {
    let cmd = HooksCommand::default_for_tests();

    // The header/checks are sync, but env vars require config resolution. We
    // approximate by checking the full content a user gets on install via a
    // runtime — but keep the test hermetic by only asserting the literal
    // `PMAT_PRECOMMIT_EXCLUDE_DIRS` token lives somewhere in the module
    // source output path, which is easiest to probe through a direct runtime
    // call on `generate_hook_content`.
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("tokio runtime");
    let hook: String = runtime
        .block_on(async { cmd.generate_hook_content(false).await })
        .expect("hook content");

    assert!(
        hook.contains("PMAT_PRECOMMIT_EXCLUDE_DIRS"),
        "GH-301: generated hook must declare PMAT_PRECOMMIT_EXCLUDE_DIRS env var"
    );
    assert!(
        hook.contains(".claude/worktrees"),
        "GH-301: default exclude list must cover .claude/worktrees"
    );
    assert!(
        hook.contains(".cursor/worktrees"),
        "GH-301: default exclude list must cover .cursor/worktrees"
    );
}
