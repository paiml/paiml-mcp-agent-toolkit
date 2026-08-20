// Included from check_macs.rs — do NOT add `use` imports or `#!` attributes here.

// CB-1663..CB-1666: AGY (Antigravity 2.0) client-config compliance —
// PMAT-INIT-002 claims 4 and 5, GitHub #1031.
//
// These four checks are the read side of `pmat init --target agy`: whatever
// writes `.agents/`, this is what judges it.
//
// One rule governs all four, and it is the reason they were written the way
// they are: **a check that returns Pass when it looked at nothing is the
// defect, not the baseline.** `pmat comply check` already reports ~49 Skips
// out of 155, and a Skip whose reason is "No X directory" is indistinguishable
// from a Pass in every summary line a human actually reads. So:
//
//   - `.agents/` absent            -> Skip, with a reason that says nothing was judged.
//   - `.agents/` present, artifact
//     absent                       -> Skip, naming the artifact that was missing.
//   - artifact present             -> judged, and the count of judged items appears
//                                     in the message for Pass, Warn and Fail alike,
//                                     so "0 judged" can never read as "0 problems".
//   - a container present but empty -> Fail. An empty `.agents/skills/` is six dead
//                                     skills as far as the client is concerned; it
//                                     is not a clean result. This is a deliberate
//                                     departure from CB-1650, which skips on
//                                     `No skill files under .claude/skills`.

/// Directory every AGY (Antigravity) client artifact lives under.
pub(crate) const AGY_DIR: &str = ".agents";

/// Frontmatter keys a `.agents/skills/*/SKILL.md` must declare.
///
/// Derived from the six skills committed in this repo, not invented: all six
/// carry `effort` and `description`; only one carries `allowed-tools`, so
/// `allowed-tools` is optional and is not required here.
pub(crate) const AGY_REQUIRED_SKILL_KEYS: [&str; 2] = ["effort", "description"];

/// Hook event keys an AGY `hooks.json` entry may declare. `PreToolUse` is the
/// one PMAT-INIT-002 claim 1 names; `PostToolUse` is accepted as its mirror.
pub(crate) const AGY_HOOK_EVENTS: [&str; 2] = ["PreToolUse", "PostToolUse"];

/// Build the final verdict from a judged-count summary plus hard/soft findings.
///
/// The summary is prepended to EVERY outcome, including Pass. That is the whole
/// point: a reader can always tell a clean run over 7 artifacts from a run that
/// found nothing to open.
fn agy_verdict(
    name: &str,
    summary: &str,
    hard: &[String],
    soft: &[String],
) -> ComplianceCheck {
    if !hard.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: format!(
                "{summary}; {} violation(s):\n{}{}",
                hard.len(),
                format_violation_list(hard),
                agy_soft_suffix(soft)
            ),
            severity: Severity::Error,
        };
    }
    if !soft.is_empty() {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Warn,
            message: format!(
                "{summary}; no hard violation, {} warning(s):\n{}",
                soft.len(),
                format_violation_list(soft)
            ),
            severity: Severity::Warning,
        };
    }
    ComplianceCheck {
        name: name.to_string(),
        status: CheckStatus::Pass,
        message: summary.to_string(),
        severity: Severity::Info,
    }
}

fn agy_soft_suffix(soft: &[String]) -> String {
    if soft.is_empty() {
        String::new()
    } else {
        format!(
            "\n  plus {} warning(s):\n{}",
            soft.len(),
            format_violation_list(soft)
        )
    }
}

/// Path relative to the project root, with `/` separators, for messages.
fn agy_rel(project_path: &Path, path: &Path) -> String {
    path.strip_prefix(project_path)
        .unwrap_or(path)
        .to_string_lossy()
        .replace('\\', "/")
}

/// Every `*.<ext>` under `dir`, recursively, sorted.
fn agy_files_with_ext(dir: &Path, ext: &str) -> Vec<std::path::PathBuf> {
    let mut out = Vec::new();
    agy_collect_ext(dir, ext, &mut out);
    out.sort();
    out
}

fn agy_collect_ext(dir: &Path, ext: &str, out: &mut Vec<std::path::PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            agy_collect_ext(&path, ext, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            out.push(path);
        }
    }
}

/// Immediate subdirectories of `.agents/skills/`, sorted. One per skill.
fn agy_skill_dirs(agents_dir: &Path) -> Vec<std::path::PathBuf> {
    let mut dirs = Vec::new();
    let Ok(entries) = std::fs::read_dir(agents_dir.join("skills")) else {
        return dirs;
    };
    for entry in entries.flatten() {
        if entry.path().is_dir() {
            dirs.push(entry.path());
        }
    }
    dirs.sort();
    dirs
}

/// The `SKILL.md`/`skill.md` inside a skill directory, if it has one.
fn agy_skill_file(skill_dir: &Path) -> Option<std::path::PathBuf> {
    ["SKILL.md", "skill.md"]
        .iter()
        .map(|f| skill_dir.join(f))
        .find(|p| p.is_file())
}

/// Is `path` executable by anybody? A hook whose script is present but not
/// `+x` fails at exec time exactly like a missing one, and both clients fail
/// open, so it is a silent no-op rather than an error.
fn agy_is_executable(path: &Path) -> bool {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::metadata(path)
            .map(|m| m.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        path.is_file()
    }
}

/// CB-1663: `.agents/` structure and JSON syntax (PMAT-INIT-002 claim 4).
///
/// Judges three structural facts and nothing else — the schema of each file is
/// CB-1664/1665/1666's job:
///   1. every `*.json` under `.agents/` parses as JSON;
///   2. every `.agents/rules/*.md` is non-empty;
///   3. every `.agents/skills/<name>/` contains a `SKILL.md`.
///
/// Absent files are NOT a violation here (a project may legitimately ship only
/// skills); the per-artifact checks report their own absence as an explicit
/// Skip. What IS a violation is a `.agents/` with nothing judgeable in it.
pub(crate) fn check_agy_structure(project_path: &Path) -> ComplianceCheck {
    let name = "CB-1663: AGY Directory Structure";
    let agents_dir = project_path.join(AGY_DIR);
    if !agents_dir.is_dir() {
        // `.agents` present but not a directory is a layout error, not an
        // absence — skipping it would be the silent hole this family exists to
        // close, so it is reported rather than waved through.
        if agents_dir.exists() {
            return ComplianceCheck {
                name: name.to_string(),
                status: CheckStatus::Fail,
                message: ".agents exists but is not a directory — no client can read \
                          a hook, skill or MCP registration out of it"
                    .to_string(),
                severity: Severity::Error,
            };
        }
        return skip_check(
            name,
            "no .agents/ directory in this project — 0 AGY artifacts judged",
        );
    }

    let mut hard: Vec<String> = Vec::new();
    let soft: Vec<String> = Vec::new();

    let json_files = agy_files_with_ext(&agents_dir, "json");
    for path in &json_files {
        let rel = agy_rel(project_path, path);
        match std::fs::read_to_string(path) {
            Err(e) => hard.push(format!("{rel}: unreadable ({e})")),
            Ok(text) => {
                if let Err(e) = serde_json::from_str::<serde_json::Value>(&text) {
                    hard.push(format!(
                        "{rel}: invalid JSON at line {} column {} — {e}",
                        e.line(),
                        e.column()
                    ));
                }
            }
        }
    }

    let rules_dir = agents_dir.join("rules");
    let rule_files = if rules_dir.is_dir() {
        agy_files_with_ext(&rules_dir, "md")
    } else {
        Vec::new()
    };
    for path in &rule_files {
        let rel = agy_rel(project_path, path);
        match std::fs::read_to_string(path) {
            Err(e) => hard.push(format!("{rel}: unreadable ({e})")),
            Ok(text) if text.trim().is_empty() => {
                hard.push(format!("{rel}: empty rule file (0 bytes of content)"))
            }
            Ok(_) => {}
        }
    }

    let skill_dirs = agy_skill_dirs(&agents_dir);
    for dir in &skill_dirs {
        if agy_skill_file(dir).is_none() {
            hard.push(format!(
                "{}/: skill directory with no SKILL.md — the client loads nothing from it",
                agy_rel(project_path, dir)
            ));
        }
    }

    let judged = json_files.len() + rule_files.len() + skill_dirs.len();
    if judged == 0 {
        return ComplianceCheck {
            name: name.to_string(),
            status: CheckStatus::Fail,
            message: ".agents/ exists but holds no *.json, no rules/*.md and no skills/*/ \
                      — 0 artifacts judged. An empty AGY tree is inert, not compliant"
                .to_string(),
            severity: Severity::Error,
        };
    }

    let summary = format!(
        "judged {judged} AGY artifact(s) under .agents/: {} JSON config(s) parsed, \
         {} rule file(s), {} skill dir(s)",
        json_files.len(),
        rule_files.len(),
        skill_dirs.len()
    );
    agy_verdict(name, &summary, &hard, &soft)
}
