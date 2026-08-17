//! `pmat init` — bootstrap an agent-ready workspace (PMAT-INIT-001/002/003,
//! issues #1030, #1031, #1032).
//!
//! Three properties are load-bearing and each one is a test in `tests.rs`:
//!
//! 1. **It never destroys work.** An existing file is read and compared before
//!    anything is written. Identical → left alone. Different → left alone and
//!    reported, unless `--force`. A bootstrap that clobbers a hand-written
//!    `hooks.json` is a data-loss bug, not a convenience.
//! 2. **The MCP registration it writes actually speaks MCP.** The template
//!    this repo shipped named a command that exits at argument parsing with
//!    zero bytes of output; `mcp_config_names_a_command_that_actually_speaks_mcp`
//!    spawns whatever is emitted and requires a valid JSON-RPC `initialize`
//!    reply before it passes.
//! 3. **It refuses rather than inventing.** Artifacts whose format nobody has
//!    defined are reported as refusals with the precise reason, on the
//!    precedent of `pmat agy sync` (#984). A guessed schema is
//!    indistinguishable from a working one until it fails silently in a user's
//!    editor.
//!
//! The generator is a pure function: [`plan`] returns the bytes and the
//! refusals with no filesystem access at all, and [`apply`] is the only thing
//! that touches disk. That is what lets the interesting assertions run without
//! a tempdir.

pub mod templates;

#[cfg(test)]
mod tests;

use std::fmt;
use std::path::{Path, PathBuf};

/// Which agent client's layout to write.
///
/// Deliberately not a `clap::ValueEnum`: nothing else under `src/services`
/// depends on clap and this module should stay callable from a test or a
/// library consumer that has no CLI. `crate::cli::commands::InitTarget`
/// carries the clap derive and converts into this.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    /// Antigravity / the cross-client `.agents/` layout.
    Agy,
    /// Claude Code: `.claude/` + the root-level `.mcp.json` project config.
    Claude,
    /// Claude Code plus a committed ultracode judgment workflow. Ultracode is
    /// a session-only harness effort setting (MACS spec E1), not a separate
    /// client with a config format of its own — so this target is `Claude`
    /// plus the one artifact the convention actually defines.
    Ultracode,
}

impl Target {
    /// Stable lowercase name, used by `--format json` and by error text.
    pub fn as_str(self) -> &'static str {
        match self {
            Target::Agy => "agy",
            Target::Claude => "claude",
            Target::Ultracode => "ultracode",
        }
    }
}

impl fmt::Display for Target {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// One file the generator wants to exist, with the exact bytes it should hold.
#[derive(Debug, Clone)]
pub struct Artifact {
    /// Path relative to the workspace root.
    pub path: &'static str,
    /// Exact contents.
    pub contents: String,
    /// Whether the file needs the executable bit (unix only).
    pub executable: bool,
    /// One line explaining why this file exists, printed next to it.
    pub note: &'static str,
}

/// Something the ticket asks for that this command will NOT produce, and why.
///
/// A refusal is printed, is carried in `--format json`, and is counted in the
/// summary. What it is not is a silent omission: the failure mode this whole
/// ticket exists to remove is an artifact that looks right and is not.
#[derive(Debug, Clone)]
pub struct Refusal {
    /// What was asked for.
    pub artifact: &'static str,
    /// Precise reason, ending in a pointer to where the gap is tracked.
    pub reason: &'static str,
}

/// The bytes and the refusals for a target — computed without touching disk.
#[derive(Debug, Clone)]
pub struct Plan {
    /// Target this plan was built for.
    pub target: Target,
    /// Files to create.
    pub artifacts: Vec<Artifact>,
    /// Files deliberately not created.
    pub refusals: Vec<Refusal>,
}

/// What happened to one artifact on disk.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// Did not exist; created.
    Created,
    /// Existed with exactly these bytes; left alone. This is the idempotent
    /// second-run case.
    AlreadyCurrent,
    /// Existed with different bytes; left alone because `--force` was not
    /// given. The user's version survives.
    KeptYours,
    /// Existed with different bytes and `--force` was given; replaced.
    Overwritten,
}

impl Outcome {
    /// Fixed-width label for the human report.
    pub fn label(self) -> &'static str {
        match self {
            Outcome::Created => "created",
            Outcome::AlreadyCurrent => "current",
            Outcome::KeptYours => "kept   ",
            Outcome::Overwritten => "forced ",
        }
    }

    /// Stable machine name for `--format json`.
    pub fn as_str(self) -> &'static str {
        self.label().trim_end()
    }

    /// Did this outcome leave the user's own bytes in place?
    pub fn preserved_user_content(self) -> bool {
        matches!(self, Outcome::KeptYours)
    }
}

/// One artifact after [`apply`].
#[derive(Debug, Clone)]
pub struct Applied {
    /// Path relative to the workspace root.
    pub path: &'static str,
    /// What happened.
    pub outcome: Outcome,
    /// The artifact's one-line rationale.
    pub note: &'static str,
}

/// The result of a full `pmat init` run.
#[derive(Debug, Clone)]
pub struct Report {
    /// Target that was applied.
    pub target: Target,
    /// Absolute workspace root.
    pub root: PathBuf,
    /// Per-artifact outcomes, in plan order.
    pub applied: Vec<Applied>,
    /// Refusals carried through from the plan.
    pub refusals: Vec<Refusal>,
}

impl Report {
    /// Files whose bytes this run wrote.
    pub fn written(&self) -> usize {
        self.applied
            .iter()
            .filter(|a| matches!(a.outcome, Outcome::Created | Outcome::Overwritten))
            .count()
    }

    /// Files that already held exactly the template bytes.
    pub fn already_current(&self) -> usize {
        self.applied
            .iter()
            .filter(|a| a.outcome == Outcome::AlreadyCurrent)
            .count()
    }

    /// Files where the user's differing version was preserved.
    pub fn kept(&self) -> usize {
        self.applied
            .iter()
            .filter(|a| a.outcome.preserved_user_content())
            .count()
    }
}

// ── the plan ───────────────────────────────────────────────────────────────

/// Build the artifact plan for `target`. Pure: no I/O, no ambient state.
///
/// # Refusals
///
/// `agy` refuses `.agents/plugins.json` and `ultracode` refuses an "ultracode
/// schema / tool-registration manifest". Both are named by their tickets and
/// neither has a field list, a required-key set or a version anywhere in this
/// repository or in any document it cites. `pmat agy sync` established the
/// house rule for exactly this situation and it is the right one: report
/// precisely what is missing, write nothing.
pub fn plan(target: Target) -> Plan {
    let mut artifacts = shared_artifacts();
    let mut refusals = Vec::new();

    match target {
        Target::Agy => {
            artifacts.extend(agy_artifacts());
            refusals.push(PLUGINS_JSON_REFUSAL);
        }
        Target::Claude => artifacts.extend(claude_artifacts()),
        Target::Ultracode => {
            artifacts.extend(claude_artifacts());
            artifacts.push(Artifact {
                path: "contracts/workflows/pmat-quality-sweep.ultracode.mjs",
                contents: templates::ULTRACODE_WORKFLOW_MJS.to_string(),
                executable: false,
                note: "committed judgment-layer workflow (plain ESM, `node --check`-able)",
            });
            refusals.push(ULTRACODE_SCHEMA_REFUSAL);
        }
    }

    Plan {
        target,
        artifacts,
        refusals,
    }
}

/// The hook entrypoint and the root rules file are the same for every target:
/// one script, one `AGENTS.md`. Claude Code's own settings in this repository
/// point at `.agents/hooks/`, so the shared location is the measured
/// convention rather than a preference.
fn shared_artifacts() -> Vec<Artifact> {
    vec![
        Artifact {
            path: ".agents/hooks/pmat-quality-feedback.sh",
            contents: templates::QUALITY_FEEDBACK_HOOK.to_string(),
            executable: true,
            note: "shared pre-write quality hook (FEEDBACK, not a gate — see AGENTS.md)",
        },
        Artifact {
            path: "AGENTS.md",
            contents: templates::AGENTS_MD.to_string(),
            executable: false,
            note: "root rules file, read by Claude Code and Antigravity alike",
        },
    ]
}

fn agy_artifacts() -> Vec<Artifact> {
    vec![
        Artifact {
            path: ".agents/hooks.json",
            contents: templates::AGY_HOOKS_JSON.to_string(),
            executable: false,
            note: "PreToolUse manifest (PMAT-INIT-002 claim 1)",
        },
        Artifact {
            path: ".agents/mcp_config.json",
            contents: templates::MCP_CONFIG_JSON.to_string(),
            executable: false,
            note: "MCP server registration — `pmat --mode mcp`, spawn-tested",
        },
        Artifact {
            path: ".agents/skills/pmat-quality/SKILL.md",
            contents: templates::SKILL_MD.to_string(),
            executable: false,
            note: "skill with pinned `effort` frontmatter",
        },
    ]
}

fn claude_artifacts() -> Vec<Artifact> {
    vec![
        Artifact {
            path: ".claude/settings.json",
            contents: templates::CLAUDE_SETTINGS_JSON.to_string(),
            executable: false,
            note: "PreToolUse hook via $CLAUDE_PROJECT_DIR (no cwd hazard)",
        },
        Artifact {
            path: ".mcp.json",
            contents: templates::MCP_CONFIG_JSON.to_string(),
            executable: false,
            note: "MCP server registration — `pmat --mode mcp`, spawn-tested",
        },
        Artifact {
            path: ".claude/skills/pmat-quality/SKILL.md",
            contents: templates::SKILL_MD.to_string(),
            executable: false,
            note: "skill with pinned `effort` frontmatter (the path CB-1650 reads)",
        },
    ]
}

const PLUGINS_JSON_REFUSAL: Refusal = Refusal {
    artifact: ".agents/plugins.json",
    reason: "PMAT-INIT-002 claim 3 asks for a \"plugins.json scaffold\", but no plugins.json \
             schema exists anywhere in this repository or in any document it cites: no field \
             list, no required keys, no version, and no example file to derive one from. \
             Emitting a plausible-looking one would be indistinguishable from working code \
             until it failed silently in the user's editor, so nothing was written. This is \
             the same refusal `pmat agy sync` makes for the same reason (#984). Define the \
             schema in a spec first and this becomes a two-line change. \
             Track: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/1031",
};

const ULTRACODE_SCHEMA_REFUSAL: Refusal = Refusal {
    artifact: "ultracode schema / tool-registration manifest",
    reason: "Issue #1032 claim 1 asks for \"correct Ultracode schemas and tool registrations\", \
             but ultracode has no file format to be correct about. \
             docs/specifications/components/modern-agentic-coding-support.md E1 states it \
             plainly: ultracode is a session-only Claude Code HARNESS setting (xhigh effort + \
             orchestration), not a client with a config of its own; E8 adds that it is \
             triggered by a keyword in a prompt. The only committed ultracode artifact \
             convention in this repository is the judgment-workflow script \
             (contracts/workflows/*.ultracode.mjs), which this target does generate. Tool \
             registration is therefore MCP registration, and that was written to .mcp.json. \
             No separate schema file was invented. \
             Track: https://github.com/paiml/paiml-mcp-agent-toolkit/issues/1032",
};

// ── applying the plan ──────────────────────────────────────────────────────

/// Write `plan` under `root`, preserving anything the user has changed unless
/// `force`.
///
/// Every write is compare-then-act: the existing bytes are read first, so a
/// second run reports `current` for untouched files and `kept` for edited
/// ones, and in neither case is a byte of the user's work lost. `force`
/// upgrades `kept` to `forced` and nothing else — a file that already matches
/// is still not rewritten, so `--force` does not churn mtimes.
pub fn apply(plan: &Plan, root: &Path, force: bool) -> std::io::Result<Report> {
    let mut applied = Vec::with_capacity(plan.artifacts.len());
    for artifact in &plan.artifacts {
        let outcome = apply_one(artifact, root, force)?;
        applied.push(Applied {
            path: artifact.path,
            outcome,
            note: artifact.note,
        });
    }
    Ok(Report {
        target: plan.target,
        root: root.to_path_buf(),
        applied,
        refusals: plan.refusals.clone(),
    })
}

fn apply_one(artifact: &Artifact, root: &Path, force: bool) -> std::io::Result<Outcome> {
    let dest = root.join(artifact.path);

    // Read before write. `read_to_string` failing on a non-UTF8 or unreadable
    // file must NOT be treated as "absent" — that would be the clobber this
    // command exists to avoid — so existence is decided by `try_exists`.
    if dest.try_exists()? {
        let existing = std::fs::read(&dest)?;
        if existing == artifact.contents.as_bytes() {
            ensure_mode(&dest, artifact.executable)?;
            return Ok(Outcome::AlreadyCurrent);
        }
        if !force {
            return Ok(Outcome::KeptYours);
        }
        write_file(&dest, artifact)?;
        return Ok(Outcome::Overwritten);
    }

    write_file(&dest, artifact)?;
    Ok(Outcome::Created)
}

fn write_file(dest: &Path, artifact: &Artifact) -> std::io::Result<()> {
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(dest, artifact.contents.as_bytes())?;
    ensure_mode(dest, artifact.executable)
}

/// Set the executable bit when the artifact needs it.
///
/// A hook script without `+x` is the failure both clients treat as an
/// approval: the command cannot launch, so the edit is allowed and nothing
/// says why. Applied on the `AlreadyCurrent` path too, so a second run repairs
/// a mode that something else stripped.
#[cfg(unix)]
fn ensure_mode(dest: &Path, executable: bool) -> std::io::Result<()> {
    use std::os::unix::fs::PermissionsExt;
    if !executable {
        return Ok(());
    }
    let mut perms = std::fs::metadata(dest)?.permissions();
    if perms.mode() & 0o111 != 0o111 {
        perms.set_mode(0o755);
        std::fs::set_permissions(dest, perms)?;
    }
    Ok(())
}

#[cfg(not(unix))]
fn ensure_mode(_dest: &Path, _executable: bool) -> std::io::Result<()> {
    Ok(())
}

// ── rendering ──────────────────────────────────────────────────────────────

/// Human-readable report.
pub fn render_human(report: &Report) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "pmat init — target: {}\nroot: {}\n\n",
        report.target,
        report.root.display()
    ));

    for a in &report.applied {
        out.push_str(&format!(
            "  {}  {:<44} {}\n",
            a.outcome.label(),
            a.path,
            a.note
        ));
    }

    if report.kept() > 0 {
        out.push_str(
            "\nkept: the file on disk differs from the template and was NOT modified.\n\
             Re-run with --force to replace it (your version is not backed up).\n",
        );
    }

    for r in &report.refusals {
        out.push_str(&format!(
            "\nrefused — nothing written: {}\n  {}\n",
            r.artifact,
            wrap(r.reason, 76, "  ")
        ));
    }

    out.push_str(&format!(
        "\n{} written, {} already current, {} kept, {} refused\n",
        report.written(),
        report.already_current(),
        report.kept(),
        report.refusals.len()
    ));
    out.push_str(
        "\nnext: install pmat on PATH (`cargo install pmat`), then verify the MCP\n\
         registration with:  pmat --mode mcp <<< '{\"jsonrpc\":\"2.0\",\"id\":1,\
         \"method\":\"initialize\",\"params\":{}}'\n",
    );
    out
}

/// Machine-readable report.
pub fn render_json(report: &Report) -> serde_json::Value {
    serde_json::json!({
        "target": report.target.as_str(),
        "root": report.root.display().to_string(),
        "artifacts": report.applied.iter().map(|a| serde_json::json!({
            "path": a.path,
            "outcome": a.outcome.as_str(),
            "note": a.note,
        })).collect::<Vec<_>>(),
        "refused": report.refusals.iter().map(|r| serde_json::json!({
            "artifact": r.artifact,
            "reason": r.reason,
        })).collect::<Vec<_>>(),
        "summary": {
            "written": report.written(),
            "already_current": report.already_current(),
            "kept": report.kept(),
            "refused": report.refusals.len(),
        }
    })
}

/// Greedy word wrap with a continuation indent, so a refusal reads as prose in
/// a terminal instead of one 900-column line.
fn wrap(text: &str, width: usize, indent: &str) -> String {
    let mut lines = Vec::new();
    let mut current = String::new();
    for word in text.split_whitespace() {
        if !current.is_empty() && current.len() + 1 + word.len() > width {
            lines.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        lines.push(current);
    }
    lines.join(&format!("\n{indent}"))
}
