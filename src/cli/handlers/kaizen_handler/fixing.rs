//! Kaizen fixing: automated fix application and AI agent delegation.

use super::git_ops::commit_changes;
use super::{FindingSource, KaizenFinding};
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Apply safe deterministic fixes (clippy --fix + cargo fmt), verify with cargo check.
/// Returns the number of fixes applied.
pub(crate) fn apply_safe_fixes(path: &Path, findings: &mut [KaizenFinding]) -> Result<usize> {
    let has_clippy = findings
        .iter()
        .any(|f| f.source == FindingSource::Clippy && f.auto_fixable);
    let has_fmt = findings
        .iter()
        .any(|f| f.source == FindingSource::Rustfmt && f.auto_fixable);

    if !has_clippy && !has_fmt {
        return Ok(0);
    }

    // Run clippy --fix
    if has_clippy {
        let status = Command::new("cargo")
            .args([
                "clippy",
                "--fix",
                "--allow-dirty",
                "--allow-staged",
                "--quiet",
            ])
            .current_dir(path)
            .status()
            .context("Failed to run cargo clippy --fix")?;

        if !status.success() {
            eprintln!("Kaizen: clippy --fix returned non-zero, reverting");
            let _ = Command::new("git")
                .args(["checkout", "--", "."])
                .current_dir(path)
                .status();
            return Ok(0);
        }
    }

    // Run cargo fmt
    if has_fmt {
        let status = Command::new("cargo")
            .args(["fmt"])
            .current_dir(path)
            .status()
            .context("Failed to run cargo fmt")?;

        if !status.success() {
            eprintln!("Kaizen: cargo fmt returned non-zero");
        }
    }

    // Verify with cargo check
    let check = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(path)
        .status()
        .context("Failed to run cargo check")?;

    if !check.success() {
        eprintln!("Kaizen: cargo check failed after fixes, reverting");
        let _ = Command::new("git")
            .args(["checkout", "--", "."])
            .current_dir(path)
            .status();
        return Ok(0);
    }

    // Mark applied
    let mut count = 0usize;
    for f in findings.iter_mut() {
        if f.auto_fixable
            && (f.source == FindingSource::Clippy || f.source == FindingSource::Rustfmt)
        {
            f.fix_applied = true;
            count += 1;
        }
    }

    Ok(count)
}

/// Apply safe fixes for a specific crate's findings within a cross-stack run.
/// Runs clippy --fix and cargo fmt in the crate directory, then marks matching findings.
pub(crate) fn apply_safe_fixes_for_crate(
    crate_path: &Path,
    crate_name: &str,
    findings: &mut [KaizenFinding],
) -> Result<usize> {
    let has_clippy = findings.iter().any(|f| {
        f.crate_name.as_deref() == Some(crate_name)
            && f.source == FindingSource::Clippy
            && f.auto_fixable
    });
    let has_fmt = findings.iter().any(|f| {
        f.crate_name.as_deref() == Some(crate_name)
            && f.source == FindingSource::Rustfmt
            && f.auto_fixable
    });

    if !has_clippy && !has_fmt {
        return Ok(0);
    }

    if has_clippy {
        let status = Command::new("cargo")
            .args([
                "clippy",
                "--fix",
                "--allow-dirty",
                "--allow-staged",
                "--quiet",
            ])
            .current_dir(crate_path)
            .status()
            .context("Failed to run cargo clippy --fix")?;

        if !status.success() {
            eprintln!(
                "Kaizen: clippy --fix returned non-zero for {}, reverting",
                crate_name
            );
            let _ = Command::new("git")
                .args(["checkout", "--", "."])
                .current_dir(crate_path)
                .status();
            return Ok(0);
        }
    }

    if has_fmt {
        let _ = Command::new("cargo")
            .args(["fmt"])
            .current_dir(crate_path)
            .status();
    }

    let check = Command::new("cargo")
        .args(["check", "--quiet"])
        .current_dir(crate_path)
        .status()
        .context("Failed to run cargo check")?;

    if !check.success() {
        eprintln!(
            "Kaizen: cargo check failed after fixes for {}, reverting",
            crate_name
        );
        let _ = Command::new("git")
            .args(["checkout", "--", "."])
            .current_dir(crate_path)
            .status();
        return Ok(0);
    }

    let mut count = 0usize;
    for f in findings.iter_mut() {
        if f.crate_name.as_deref() == Some(crate_name)
            && f.auto_fixable
            && (f.source == FindingSource::Clippy || f.source == FindingSource::Rustfmt)
        {
            f.fix_applied = true;
            count += 1;
        }
    }

    Ok(count)
}

/// Spawn AI sub-agents for complex findings. Runs sequentially in v1.
pub(crate) fn spawn_agents(
    path: &Path,
    findings: &mut [KaizenFinding],
    max_agents: usize,
    commit: bool,
) -> Result<usize> {
    let mut fixed = 0usize;
    let mut attempted = 0usize;

    for finding in findings.iter_mut() {
        if attempted >= max_agents {
            break;
        }
        if finding.fix_applied || !finding.agent_fixable {
            continue;
        }
        let Some(prompt) = &finding.agent_prompt else {
            continue;
        };

        attempted += 1;
        eprintln!(
            "Kaizen: agent [{}/{}] {}",
            attempted, max_agents, finding.category
        );

        let status = Command::new("claude")
            .args([
                "-p",
                prompt,
                "--allowedTools",
                "Bash,Read,Edit,Write,Grep,Glob",
            ])
            .current_dir(path)
            .status();

        match status {
            Ok(s) if s.success() => {
                // Verify with cargo check
                let check = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(path)
                    .status();

                if check.map(|s| s.success()).unwrap_or(false) {
                    finding.fix_applied = true;
                    fixed += 1;

                    if commit {
                        let msg = format!("kaizen(agent): fix {}", finding.category);
                        let _ = commit_changes(path, &msg);
                    }
                } else {
                    eprintln!("Kaizen: agent fix broke cargo check, reverting");
                    let _ = Command::new("git")
                        .args(["checkout", "--", "."])
                        .current_dir(path)
                        .status();
                }
            }
            Ok(_) => {
                eprintln!("Kaizen: agent returned non-zero for {}", finding.category);
            }
            Err(e) => {
                eprintln!("Kaizen: failed to spawn claude: {e}");
                break; // claude CLI not available, stop trying
            }
        }
    }

    Ok(fixed)
}

/// Spawn agents for a specific crate's findings in cross-stack mode.
pub(crate) fn spawn_agents_for_crate(
    crate_path: &Path,
    crate_name: &str,
    findings: &mut [KaizenFinding],
    max_agents: usize,
    commit: bool,
) -> Result<usize> {
    let mut fixed = 0usize;
    let mut attempted = 0usize;

    for finding in findings.iter_mut() {
        if attempted >= max_agents {
            break;
        }
        if finding.crate_name.as_deref() != Some(crate_name) {
            continue;
        }
        if finding.fix_applied || !finding.agent_fixable {
            continue;
        }
        let Some(prompt) = &finding.agent_prompt else {
            continue;
        };

        attempted += 1;
        eprintln!(
            "Kaizen: agent [{}/{}] {} ({})",
            attempted, max_agents, finding.category, crate_name
        );

        let status = Command::new("claude")
            .args([
                "-p",
                prompt,
                "--allowedTools",
                "Bash,Read,Edit,Write,Grep,Glob",
            ])
            .current_dir(crate_path)
            .status();

        match status {
            Ok(s) if s.success() => {
                let check = Command::new("cargo")
                    .args(["check", "--quiet"])
                    .current_dir(crate_path)
                    .status();

                if check.map(|s| s.success()).unwrap_or(false) {
                    finding.fix_applied = true;
                    fixed += 1;
                    if commit {
                        let msg =
                            format!("kaizen(agent): fix {} in {}", finding.category, crate_name);
                        let _ = commit_changes(crate_path, &msg);
                    }
                } else {
                    eprintln!(
                        "Kaizen: agent fix broke cargo check for {}, reverting",
                        crate_name
                    );
                    let _ = Command::new("git")
                        .args(["checkout", "--", "."])
                        .current_dir(crate_path)
                        .status();
                }
            }
            Ok(_) => {
                eprintln!("Kaizen: agent returned non-zero for {}", finding.category);
            }
            Err(e) => {
                eprintln!("Kaizen: failed to spawn claude: {e}");
                break;
            }
        }
    }

    Ok(fixed)
}
