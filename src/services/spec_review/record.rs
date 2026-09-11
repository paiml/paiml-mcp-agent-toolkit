//! `pmat spec review --record <json>` (goal-mode.md §6.3): validate, then
//! stage. It does not produce. Production is a quorum's job or a human's —
//! the artifact is JSON, and a hand-written review with real findings is a
//! legitimate one.

use super::{artifact_path, judge, ReviewArtifact, ReviewFinding};
use crate::services::spec_epic::{parse_front_matter, SPECS_DIR};
use std::path::{Path, PathBuf};

/// What `--record` staged.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Recorded {
    /// Project-relative: `docs/audits/spec-<slug>-review.json`.
    pub artifact: String,
    pub spec: String,
    pub spec_sha256: String,
}

/// Why `--record` refused. [`RecordRefusal::NotStaged`] reports a file
/// written and not staged; every refusal checked before the write writes nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecordRefusal {
    Unreadable {
        file: PathBuf,
        reason: String,
    },
    BadReview {
        file: PathBuf,
        reason: String,
    },
    NotASpec {
        named: String,
    },
    SpecUnreadable {
        spec: String,
        reason: String,
    },
    Unjudgeable {
        spec: String,
        why: String,
    },
    Findings(Vec<ReviewFinding>),
    Unwritable {
        artifact: String,
        reason: String,
    },
    NotStaged {
        artifact: String,
        reason: String,
    },
    /// The artifact path cannot be staged (git ignores it, or this is not a
    /// git work tree), so nothing is written.
    NotStageable {
        artifact: String,
        reason: String,
    },
}

impl RecordRefusal {
    /// One line a reader can act on.
    pub fn render(&self) -> String {
        match self {
            RecordRefusal::Unreadable { file, reason } => format!("{} cannot be read ({reason}); nothing recorded", file.display()),
            RecordRefusal::BadReview { file, reason } => format!("BAD-REVIEW {}: does not parse as a review (goal-mode.md §6.1): {reason}; nothing recorded", file.display()),
            RecordRefusal::NotASpec { named } => format!("the review names {named}, which is not a spec under docs/specifications/ (a relative path ending .md, no . or .. segment); nothing recorded"),
            RecordRefusal::SpecUnreadable { spec, reason } => format!("the review names {spec}, which cannot be read ({reason}); nothing recorded"),
            RecordRefusal::Unjudgeable { spec, why } => format!("UNJUDGEABLE {spec}: its front-matter does not parse, so the roles its review needs cannot be read ({why}); nothing recorded"),
            RecordRefusal::Findings(findings) => format!("{} finding(s) — the review would fail CB-2111 as recorded; nothing recorded", findings.len()),
            RecordRefusal::Unwritable { artifact, reason } => format!("{artifact} cannot be written ({reason}); nothing staged"),
            RecordRefusal::NotStageable { artifact, reason } => format!("{artifact} cannot be staged ({reason}); nothing recorded"),
            RecordRefusal::NotStaged { artifact, reason } => format!("{artifact} is written but NOT staged: git add failed ({reason})"),
        }
    }
}

/// Validate the review at `file` against the spec it names under `project`,
/// exactly as CB-2111 will judge it; on success write it, byte for byte, to
/// its artifact path and `git add` it.
pub fn record(project: &Path, file: &Path) -> Result<Recorded, RecordRefusal> {
    let text = std::fs::read_to_string(file).map_err(|e| RecordRefusal::Unreadable {
        file: file.to_path_buf(),
        reason: e.to_string(),
    })?;
    let review: ReviewArtifact =
        serde_json::from_str(&text).map_err(|e| RecordRefusal::BadReview {
            file: file.to_path_buf(),
            reason: e.to_string(),
        })?;
    let spec = review.spec.clone();
    if !is_spec_path(&spec) {
        return Err(RecordRefusal::NotASpec { named: spec });
    }
    let spec_text = std::fs::read_to_string(project.join(&spec)).map_err(|e| {
        RecordRefusal::SpecUnreadable {
            spec: spec.clone(),
            reason: e.to_string(),
        }
    })?;
    let front = parse_front_matter(&spec_text).map_err(|e| RecordRefusal::Unjudgeable {
        spec: spec.clone(),
        why: e.render(),
    })?;
    let findings = judge(&spec, &spec_text, &front.vendors, Some(&text));
    if !findings.is_empty() {
        return Err(RecordRefusal::Findings(findings));
    }
    let artifact = artifact_path(&spec);
    let dest = project.join(&artifact);
    stageable(project, &artifact).map_err(|reason| RecordRefusal::NotStageable {
        artifact: artifact.clone(),
        reason,
    })?;
    if let Some(link) = symlink_on_the_way(project, &artifact) {
        return Err(RecordRefusal::Unwritable {
            artifact: artifact.clone(),
            reason: format!("{link} is a symlink; record never writes through one"),
        });
    }
    let in_place = matches!(
        (std::fs::canonicalize(file), std::fs::canonicalize(&dest)),
        (Ok(a), Ok(b)) if a == b
    );
    if !in_place {
        if let Some(parent) = dest.parent() {
            std::fs::create_dir_all(parent).map_err(|e| RecordRefusal::Unwritable {
                artifact: artifact.clone(),
                reason: e.to_string(),
            })?;
        }
        write_by_rename(&dest, text.as_bytes()).map_err(|e| RecordRefusal::Unwritable {
            artifact: artifact.clone(),
            reason: e.to_string(),
        })?;
    }
    stage(project, &artifact).map_err(|reason| RecordRefusal::NotStaged {
        artifact: artifact.clone(),
        reason,
    })?;
    Ok(Recorded {
        artifact,
        spec,
        spec_sha256: review.spec_sha256,
    })
}

/// `docs/specifications/<rel>.md` with no empty, `.` or `..` segment: the
/// review's hash must be of a spec, and its artifact path must stay under
/// docs/audits.
fn is_spec_path(spec: &str) -> bool {
    spec.strip_prefix(SPECS_DIR)
        .and_then(|rest| rest.strip_prefix('/'))
        .is_some_and(|rel| {
            rel.ends_with(".md")
                && !rel.chars().any(char::is_control)
                && !rel
                    .split('/')
                    .any(|seg| seg.is_empty() || seg == "." || seg == "..")
        })
}

/// `git add -- <artifact>` in `project`; the error is git's own words.
fn stage(project: &Path, artifact: &str) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["add", "--", artifact])
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

/// `git check-ignore`: exit 1 is "not ignored"; exit 0 means git would never
/// commit the artifact; anything else (not a git work tree) is refused too.
fn stageable(project: &Path, artifact: &str) -> Result<(), String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["check-ignore", "-q", "--", artifact])
        .output()
        .map_err(|e| e.to_string())?;
    match out.status.code() {
        Some(1) => Ok(()),
        Some(0) => Err("git ignores it, so it could never be committed".to_string()),
        _ => Err(format!(
            "git check-ignore failed: {}",
            String::from_utf8_lossy(&out.stderr).trim()
        )),
    }
}

/// The first component on the way from the project root to the artifact
/// (`docs`, `docs/audits`, the file itself) that is a symlink, if any.
fn symlink_on_the_way(project: &Path, artifact: &str) -> Option<String> {
    let mut at = project.to_path_buf();
    let mut rel = PathBuf::new();
    for part in Path::new(artifact).components() {
        at.push(part);
        rel.push(part);
        if std::fs::symlink_metadata(&at).is_ok_and(|m| m.file_type().is_symlink()) {
            return Some(rel.display().to_string());
        }
    }
    None
}

/// Write `bytes` to a new file beside `dest`, then rename it over `dest`: a
/// hard link or a file already at `dest` is replaced, never written through,
/// and no reader sees half a review.
fn write_by_rename(dest: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let tmp = dest.with_extension(format!("json.tmp-{}", std::process::id()));
    let mut file = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&tmp)?;
    let written = std::io::Write::write_all(&mut file, bytes).and_then(|()| file.sync_all());
    drop(file);
    match written.and_then(|()| std::fs::rename(&tmp, dest)) {
        Ok(()) => Ok(()),
        Err(e) => {
            let _ = std::fs::remove_file(&tmp);
            Err(e)
        }
    }
}
