//! `pmat init` — route the workspace bootstrap.
//!
//! All the interesting logic lives in `crate::services::workspace_init`, which
//! is pure up to the single [`apply`](crate::services::workspace_init::apply)
//! call; this file only resolves the root, picks a renderer, and decides the
//! exit code.

use crate::cli::commands::{InitFormat, InitTarget};
use crate::services::workspace_init::{self, Report};
use std::path::{Path, PathBuf};

/// Run `pmat init`.
///
/// # Exit status
///
/// Zero when every artifact in the target's plan was created, left alone
/// because it already held the template bytes, preserved because the user had
/// changed it, or replaced under `--force`. All four are successful outcomes:
/// preserving a user's file is the command working, not failing.
///
/// Non-zero only when the root cannot be used, when a write fails, or when a
/// target's plan is empty — the last being the `pmat agy sync` case, where
/// every artifact a target would produce is undefined and the honest thing is
/// to refuse the whole target rather than create a directory of guesses.
///
/// Refusals of *individual* artifacts do not fail the run. They are printed in
/// full, counted in the summary, and carried in `--format json` under
/// `refused[]`, so a gate that wants to be strict can assert on them; failing
/// the command outright would mean `pmat init --target agy` exited non-zero
/// after correctly writing five files, which would break every `pmat init &&
/// …` a user writes.
pub fn handle_init(
    target: InitTarget,
    path: &Path,
    force: bool,
    format: InitFormat,
) -> anyhow::Result<()> {
    let root = resolve_root(path)?;
    let plan = workspace_init::plan(target.to_service());

    if plan.artifacts.is_empty() {
        anyhow::bail!("{}", empty_plan_refusal(&plan));
    }

    let report = workspace_init::apply(&plan, &root, force)
        .map_err(|e| anyhow::anyhow!("pmat init: writing under {}: {e}", root.display()))?;

    print!("{}", render(&report, format)?);
    Ok(())
}

/// The root must exist and be a directory. Creating it would turn a typo in
/// `--path` into a stray directory tree, which is the kind of quiet damage
/// this command is otherwise built to avoid.
fn resolve_root(path: &Path) -> anyhow::Result<PathBuf> {
    if !path.exists() {
        anyhow::bail!(
            "pmat init: --path does not exist: {}\n\
             Create the directory first; init will not create a workspace root for you, \
             because a typo would then silently produce one.",
            path.display()
        );
    }
    if !path.is_dir() {
        anyhow::bail!("pmat init: --path is not a directory: {}", path.display());
    }
    // Canonicalize so the report names one unambiguous location.
    Ok(std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
}

fn render(report: &Report, format: InitFormat) -> anyhow::Result<String> {
    Ok(match format {
        InitFormat::Human => workspace_init::render_human(report),
        InitFormat::Json => format!(
            "{}\n",
            serde_json::to_string_pretty(&workspace_init::render_json(report))?
        ),
    })
}

/// The whole-target refusal, modelled on `pmat agy sync`: say what was read,
/// say exactly which facts are missing, write nothing.
fn empty_plan_refusal(plan: &workspace_init::Plan) -> String {
    let mut out = format!(
        "pmat init --target {} is not implemented: every artifact this target would produce \
         has an undefined format, so nothing was written.\n",
        plan.target
    );
    for r in &plan.refusals {
        out.push_str(&format!("\n  {}\n    {}\n", r.artifact, r.reason));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_root_is_refused_rather_than_created() {
        let dir = tempfile::tempdir().expect("tempdir");
        let ghost = dir.path().join("no-such-dir");
        let err = resolve_root(&ghost).expect_err("must not succeed");
        assert!(err.to_string().contains("does not exist"), "{err}");
        assert!(
            !ghost.exists(),
            "init must not create the root it was pointed at"
        );
    }

    #[test]
    fn a_file_is_not_a_workspace_root() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("f");
        std::fs::write(&file, "x").expect("write");
        let err = resolve_root(&file).expect_err("must not succeed");
        assert!(err.to_string().contains("not a directory"), "{err}");
    }

    /// The empty-plan path is the `agy sync` refusal. No shipped target hits
    /// it today — every one of the three has at least one defined artifact —
    /// so it is asserted directly, rather than left as text nobody has read.
    #[test]
    fn an_all_undefined_target_refuses_with_its_reasons() {
        let mut plan = workspace_init::plan(workspace_init::Target::Ultracode);
        plan.artifacts.clear();
        let msg = empty_plan_refusal(&plan);
        assert!(msg.contains("is not implemented"), "{msg}");
        assert!(msg.contains("nothing was written"), "{msg}");
        assert!(msg.contains("issues/1032"), "{msg}");
    }

    #[test]
    fn shipped_targets_all_have_something_defined_to_write() {
        for t in [
            workspace_init::Target::Agy,
            workspace_init::Target::Claude,
            workspace_init::Target::Ultracode,
        ] {
            assert!(
                !workspace_init::plan(t).artifacts.is_empty(),
                "{t} would refuse wholesale"
            );
        }
    }

    #[test]
    fn json_render_is_parseable_and_human_render_is_not_empty() {
        let dir = tempfile::tempdir().expect("tempdir");
        let plan = workspace_init::plan(workspace_init::Target::Claude);
        let report = workspace_init::apply(&plan, dir.path(), false).expect("apply");

        let json = render(&report, InitFormat::Json).expect("json");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("valid json");
        assert_eq!(parsed["target"], "claude");

        let human = render(&report, InitFormat::Human).expect("human");
        assert!(human.contains("AGENTS.md"), "{human}");
    }

    #[test]
    fn handle_init_is_idempotent_end_to_end() {
        let dir = tempfile::tempdir().expect("tempdir");
        handle_init(InitTarget::Agy, dir.path(), false, InitFormat::Json).expect("first run");
        let before = std::fs::read(dir.path().join(".agents/hooks.json")).expect("read");
        handle_init(InitTarget::Agy, dir.path(), false, InitFormat::Json).expect("second run");
        let after = std::fs::read(dir.path().join(".agents/hooks.json")).expect("read");
        assert_eq!(before, after, "second run mutated the workspace");
    }
}
