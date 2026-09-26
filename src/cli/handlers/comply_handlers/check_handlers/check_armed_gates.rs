// CB-2118: contracts-armed-gates-monotone (ONT-001 R-21 (c), #1422)
// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
//
// Arming only ratchets up: a gate or shape armed in `contracts/lint-baseline.json`
// at the merge-base must still be armed at HEAD. The rule never asks for anything
// to be armed — a repo that has never armed passes — so it cannot force arming.
// It reads two JSON string lists and nothing else: no contract type, no shape.

const ARMED_GATES_BASELINE: &str = "contracts/lint-baseline.json";

/// CB-2118: no entry of `armed_gates[]` or `armed_shapes[]` is dropped between
/// the merge-base and HEAD.
pub(crate) fn check_armed_gates_monotone(project_path: &Path) -> ComplianceCheck {
    let name = "CB-2118: contracts-armed-gates-monotone";
    let check = |status, message: String, severity| ComplianceCheck {
        name: name.into(),
        status,
        message,
        severity,
    };
    if !project_path.join("contracts").is_dir() {
        return check(
            CheckStatus::Skip,
            "decline: no contracts/".into(),
            Severity::Info,
        );
    }
    let head_text = std::fs::read_to_string(project_path.join(ARMED_GATES_BASELINE)).ok();
    let base_text = match armed_gates_merge_base(project_path) {
        Ok(base) => git_show_file(project_path, &base, ARMED_GATES_BASELINE),
        Err(why) => {
            return check(
                CheckStatus::Warn,
                format!("decline: no merge-base to compare {ARMED_GATES_BASELINE} against ({why})"),
                Severity::Warning,
            )
        }
    };
    let (status, message) = judge_armed_gates(base_text.as_deref(), head_text.as_deref());
    let severity = match status {
        CheckStatus::Fail => Severity::Error,
        _ => Severity::Info,
    };
    check(status, message, severity)
}

/// Judge the two versions of the baseline. `None` = the file is absent there.
fn judge_armed_gates(base: Option<&str>, head: Option<&str>) -> (CheckStatus, String) {
    let parse = |text: Option<&str>, at: &str| -> Result<serde_json::Value, String> {
        match text {
            None => Ok(serde_json::Value::Object(Default::default())),
            Some(t) => serde_json::from_str(t)
                .map_err(|e| format!("{ARMED_GATES_BASELINE} at {at} is not JSON: {e}")),
        }
    };
    let (base_v, head_v) = match (parse(base, "the merge-base"), parse(head, "HEAD")) {
        (Ok(b), Ok(h)) => (b, h),
        (Err(e), _) | (_, Err(e)) => return (CheckStatus::Fail, e),
    };
    let mut dropped = Vec::new();
    for key in ["armed_gates", "armed_shapes"] {
        // `armed_shapes` absent means every shape is armed (ONT-001 §3.9), so
        // removing the key widens arming; `armed_gates` has no such default.
        if key == "armed_shapes" && head_v.get(key).is_none() {
            continue;
        }
        let head_names = armed_names(&head_v, key);
        dropped.extend(
            armed_names(&base_v, key)
                .into_iter()
                .filter(|n| !head_names.contains(n))
                .map(|n| format!("{key}: {n}")),
        );
    }
    if !dropped.is_empty() {
        return (
            CheckStatus::Fail,
            format!(
                "arming shrank since the merge-base — dropped {}",
                dropped.join(", ")
            ),
        );
    }
    let armed = armed_names(&head_v, "armed_gates").len() + armed_names(&head_v, "armed_shapes").len();
    if armed == 0 && base.is_none() {
        (CheckStatus::Pass, "never armed (arming is opt-in)".into())
    } else {
        (
            CheckStatus::Pass,
            format!("no armed gate or shape dropped since the merge-base ({armed} armed at HEAD)"),
        )
    }
}

/// The entries of the list at `key`, each as a string (a non-string entry as
/// its compact JSON, so it still has a name to compare and report).
fn armed_names(v: &serde_json::Value, key: &str) -> Vec<String> {
    v.get(key)
        .and_then(|l| l.as_array())
        .map(|l| {
            l.iter()
                .map(|e| e.as_str().map_or_else(|| e.to_string(), str::to_string))
                .collect()
        })
        .unwrap_or_default()
}

/// The merge-base of HEAD with the base branch: `GITHUB_BASE_REF` when set,
/// else `origin/HEAD`, `origin/master`, `origin/main`, `master`, `main`.
fn armed_gates_merge_base(project_path: &Path) -> Result<String, String> {
    let env_base = std::env::var("GITHUB_BASE_REF")
        .ok()
        .filter(|s| !s.trim().is_empty());
    let mut candidates: Vec<String> = match env_base {
        Some(b) => vec![b.clone(), format!("origin/{b}")],
        None => {
            let mut c = Vec::new();
            if let Ok(r) = armed_gates_git(project_path, &["symbolic-ref", "-q", "refs/remotes/origin/HEAD"]) {
                if let Some(s) = r.strip_prefix("refs/remotes/") {
                    c.push(s.to_string());
                }
            }
            c.extend(["origin/master", "origin/main", "master", "main"].map(String::from));
            c
        }
    };
    candidates.dedup();
    for base in &candidates {
        if let Ok(mb) = armed_gates_git(project_path, &["merge-base", base, "HEAD"]) {
            return Ok(mb);
        }
    }
    Err(format!("none of {} resolves", candidates.join(", ")))
}

/// `git show <rev>:<path>`; `None` when the file is absent at that revision.
fn git_show_file(project_path: &Path, rev: &str, path: &str) -> Option<String> {
    armed_gates_git(project_path, &["show", &format!("{rev}:{path}")]).ok()
}

fn armed_gates_git(project_path: &Path, args: &[&str]) -> Result<String, String> {
    let out = std::process::Command::new("git")
        .arg("-C")
        .arg(project_path)
        .args(args)
        .env("LC_ALL", "C")
        .output()
        .map_err(|e| format!("git: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).trim().to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).trim_end().to_string())
}
