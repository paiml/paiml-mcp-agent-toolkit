// CB-2120: declared committed derived artifacts are fresh (GH-1641).
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// The class: a committed file that a command derives from the tree (a manifest,
// a pin list, a registry) is edited around, or two clean merges leave it wrong,
// and the first thing to notice is a red PR or a red default branch.
//
// A repository declares each such file in `.pmat-derived.toml`:
//
//     [[artifact]]
//     path = "docs/registry.txt"
//     generate = ["scripts/registry.sh", "--print"]   # argv; prints the file
//     timeout_secs = 600                               # optional
//
// `generate` runs repository code, so it runs ONLY when CB-2120 is named in
// `--checks`. A plain `comply check` validates the declaration and skips.

/// The declaration file, at the project root.
const DERIVED_FILE: &str = ".pmat-derived.toml";
const DERIVED_DEFAULT_TIMEOUT_SECS: u64 = 600;
const DERIVED_NAME: &str = "CB-2120: Derived Artifacts Fresh";

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedDecl {
    artifact: Vec<DerivedArtifact>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DerivedArtifact {
    path: String,
    generate: Vec<String>,
    timeout_secs: Option<u64>,
}

fn derived_row(status: CheckStatus, message: String) -> ComplianceCheck {
    let severity = match status {
        CheckStatus::Fail | CheckStatus::Warn => Severity::Warning,
        CheckStatus::Pass | CheckStatus::Skip => Severity::Info,
    };
    ComplianceCheck {
        name: DERIVED_NAME.into(),
        status,
        message,
        severity,
    }
}

/// Parse and validate the declaration. Anything unmeasurable is an error.
fn parse_derived_decl(raw: &str) -> Result<Vec<DerivedArtifact>, String> {
    let decl: DerivedDecl = toml::from_str(raw).map_err(|e| e.to_string())?;
    if decl.artifact.is_empty() {
        return Err("no [[artifact]] is declared".to_string());
    }
    for a in &decl.artifact {
        let relative = !a.path.is_empty()
            && std::path::Path::new(&a.path)
                .components()
                .all(|c| matches!(c, std::path::Component::Normal(_)));
        if !relative {
            return Err(format!(
                "path {:?} must be relative to the root, with no `.` or `..`",
                a.path
            ));
        }
        if a.generate.is_empty() {
            return Err(format!("{}: `generate` is empty", a.path));
        }
    }
    Ok(decl.artifact)
}

/// CB-2120: does every declared derived file equal what its generator prints?
pub(crate) fn check_derived_fresh(project_path: &Path, selected: &[String]) -> ComplianceCheck {
    let file = project_path.join(DERIVED_FILE);
    let raw = match std::fs::read_to_string(&file) {
        Ok(raw) => raw,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return derived_row(
                CheckStatus::Skip,
                format!("no {DERIVED_FILE} — no derived artifact is declared"),
            )
        }
        Err(e) => return derived_row(CheckStatus::Fail, format!("{DERIVED_FILE}: {e}")),
    };
    let artifacts = match parse_derived_decl(&raw) {
        Ok(artifacts) => artifacts,
        Err(e) => return derived_row(CheckStatus::Fail, format!("{DERIVED_FILE}: {e}")),
    };
    if !selected.iter().any(|s| s.eq_ignore_ascii_case("cb-2120")) {
        return derived_row(
            CheckStatus::Skip,
            format!(
                "{} derived artifact(s) declared; generators run repository code and were \
                 not run. Verify with: pmat comply check --checks CB-2120",
                artifacts.len()
            ),
        );
    }
    let findings: Vec<String> = artifacts
        .iter()
        .filter_map(|a| derived_finding(project_path, a).err())
        .collect();
    if findings.is_empty() {
        derived_row(
            CheckStatus::Pass,
            format!("{} derived artifact(s) match their generators", artifacts.len()),
        )
    } else {
        derived_row(CheckStatus::Fail, findings.join("; "))
    }
}

/// `Ok` when the committed file is what the generator prints.
fn derived_finding(root: &Path, a: &DerivedArtifact) -> Result<(), String> {
    let path = &a.path;
    let committed = committed_bytes(root, path)?;
    let before = std::fs::read(root.join(path)).ok();
    let generated = run_generator(root, a).map_err(|e| format!("{path}: generator {e}"))?;
    if std::fs::read(root.join(path)).ok() != before {
        return Err(format!(
            "{path}: the generator rewrote the artifact; it must print to stdout"
        ));
    }
    if generated == committed {
        return Ok(());
    }
    if before.as_deref() == Some(generated.as_slice()) {
        return Err(format!("{path}: regenerated but not committed"));
    }
    Err(format!(
        "{path}: stale, first difference at line {}; regenerate with `{} > {path}` and commit",
        first_differing_line(&committed, &generated),
        a.generate.join(" ")
    ))
}

/// The bytes HEAD holds for `path`: what a push would send.
fn committed_bytes(root: &Path, path: &str) -> Result<Vec<u8>, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(root)
        .args(["cat-file", "blob", &format!("HEAD:./{path}")])
        .output()
        .map_err(|e| format!("{path}: could not run git: {e}"))?;
    if !out.status.success() {
        return Err(format!("{path}: not in HEAD; commit the derived file"));
    }
    Ok(out.stdout)
}

/// Run the generator at the root and return its stdout. Output goes to files, so
/// a large output cannot block the child while the timeout loop waits on it.
fn run_generator(root: &Path, a: &DerivedArtifact) -> Result<Vec<u8>, String> {
    use std::io::{Read, Seek};
    let io = |e: std::io::Error| format!("output could not be captured: {e}");
    let mut stdout = tempfile::tempfile().map_err(io)?;
    let mut stderr = tempfile::tempfile().map_err(io)?;
    let (program, args) = a.generate.split_first().ok_or("is empty")?;
    let mut child = std::process::Command::new(program)
        .args(args)
        .current_dir(root)
        .stdin(std::process::Stdio::null())
        .stdout(stdout.try_clone().map_err(io)?)
        .stderr(stderr.try_clone().map_err(io)?)
        .spawn()
        .map_err(|e| format!("could not start `{program}`: {e}"))?;
    let limit = std::time::Duration::from_secs(
        a.timeout_secs.unwrap_or(DERIVED_DEFAULT_TIMEOUT_SECS),
    );
    let started = std::time::Instant::now();
    let status = loop {
        if let Some(status) = child.try_wait().map_err(io)? {
            break status;
        }
        if started.elapsed() > limit {
            let _ = child.kill();
            let _ = child.wait();
            return Err(format!("timed out after {}s", limit.as_secs()));
        }
        std::thread::sleep(std::time::Duration::from_millis(50));
    };
    if !status.success() {
        let mut text = String::new();
        stderr.rewind().map_err(io)?;
        let _ = stderr.read_to_string(&mut text);
        let last = text.lines().rev().find(|l| !l.trim().is_empty());
        return Err(format!("exited with {status}: {}", last.unwrap_or("no stderr")));
    }
    let mut out = Vec::new();
    stdout.rewind().map_err(io)?;
    stdout.read_to_end(&mut out).map_err(io)?;
    Ok(out)
}

/// 1-based number of the first line that differs.
fn first_differing_line(a: &[u8], b: &[u8]) -> usize {
    let mut left = a.split_inclusive(|&c| c == b'\n');
    let mut right = b.split_inclusive(|&c| c == b'\n');
    let mut line = 1;
    loop {
        match (left.next(), right.next()) {
            (Some(l), Some(r)) if l == r => line += 1,
            _ => return line,
        }
    }
}
