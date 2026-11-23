pub(crate) fn check_cargo_lock(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    if !project_path.join("Cargo.toml").exists() {
        return ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Skip,
            message: "Not a Rust project (no Cargo.toml)".into(),
            severity: Severity::Info,
        };
    }
    if project_path.join("Cargo.lock").exists() {
        ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Pass,
            message: "Cargo.lock present - reproducible builds enabled".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "Cargo.lock Present".into(),
            status: CheckStatus::Fail,
            message: "Missing Cargo.lock - run 'cargo build' to generate".into(),
            severity: Severity::Error,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_msrv(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let cargo_toml = project_path.join("Cargo.toml");
    if !cargo_toml.exists() {
        return ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Skip,
            message: "No Cargo.toml found".into(),
            severity: Severity::Info,
        };
    }
    let content = fs::read_to_string(&cargo_toml).unwrap_or_default();
    if content.contains("rust-version") {
        ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Pass,
            message: "rust-version field present in Cargo.toml".into(),
            severity: Severity::Info,
        }
    } else {
        ComplianceCheck {
            name: "MSRV Defined".into(),
            status: CheckStatus::Warn,
            message: "No rust-version field - add to Cargo.toml for compatibility".into(),
            severity: Severity::Warning,
        }
    }
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_ci_configured(project_path: &Path) -> ComplianceCheck {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let github_workflows = project_path.join(".github").join("workflows");
    if github_workflows.exists() && github_workflows.is_dir() {
        let wf_count = fs::read_dir(&github_workflows)
            .map(|e| e.filter_map(|e| e.ok()).count())
            .unwrap_or(0);
        if wf_count > 0 {
            return ComplianceCheck {
                name: "CI Configured".into(),
                status: CheckStatus::Pass,
                message: format!("{} GitHub Actions workflow(s) found", wf_count),
                severity: Severity::Info,
            };
        }
    }
    if project_path.join(".gitlab-ci.yml").exists() {
        return ComplianceCheck {
            name: "CI Configured".into(),
            status: CheckStatus::Pass,
            message: "GitLab CI configured".into(),
            severity: Severity::Info,
        };
    }
    if project_path.join("Jenkinsfile").exists() {
        return ComplianceCheck {
            name: "CI Configured".into(),
            status: CheckStatus::Pass,
            message: "Jenkins pipeline configured".into(),
            severity: Severity::Info,
        };
    }
    ComplianceCheck {
        name: "CI Configured".into(),
        status: CheckStatus::Warn,
        message: "No CI configuration found - add .github/workflows/".into(),
        severity: Severity::Warning,
    }
}

fn onegate_has_push_trigger(doc: &serde_yaml_ng::Value) -> bool {
    let Some(on) = doc.get("on").or_else(|| doc.get(serde_yaml_ng::Value::Bool(true))) else {
        return false;
    };
    match on {
        serde_yaml_ng::Value::String(s) => s == "push",
        serde_yaml_ng::Value::Sequence(seq) => seq.iter().any(|v| v.as_str() == Some("push")),
        serde_yaml_ng::Value::Mapping(_) => on.get("push").is_some(),
        _ => false,
    }
}

fn onegate_flagged_if(cond: &str) -> bool {
    cond.contains("github.event_name")
        || cond.contains("github.head_ref")
        || cond
            .split(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
            .any(|tok| tok == "labels")
}

fn onegate_scan_workflow(file: &str, text: &str) -> Vec<String> {
    let Ok(doc) = serde_yaml_ng::from_str::<serde_yaml_ng::Value>(text) else {
        return Vec::new();
    };
    let Some(jobs) = onegate_has_push_trigger(&doc)
        .then(|| doc.get("jobs").and_then(serde_yaml_ng::Value::as_mapping))
        .flatten()
    else {
        return Vec::new();
    };
    jobs.iter()
        .filter_map(|(id, body)| {
            let job = id.as_str()?;
            let cond = body.get("if").and_then(serde_yaml_ng::Value::as_str)?;
            onegate_flagged_if(cond).then(|| format!("{file}:{job}"))
        })
        .collect()
}

/// ONEGATE (#1870): flag push-triggered workflows whose jobs skip pull_request
/// via `if:` on `github.event_name`, `github.head_ref`, or `labels`.
pub(crate) fn check_onegate(project_path: &Path) -> ComplianceCheck {
    let verdict = |status, message: String, severity| ComplianceCheck {
        name: "ONEGATE".into(),
        status,
        message,
        severity,
    };
    let dir = project_path.join(".github").join("workflows");
    if !dir.is_dir() {
        return verdict(CheckStatus::Skip, "No .github/workflows/".into(), Severity::Info);
    }
    let mut files: Vec<_> = fs::read_dir(&dir)
        .ok()
        .into_iter()
        .flatten()
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter(|p| matches!(p.extension().and_then(|e| e.to_str()), Some("yml" | "yaml")))
        .collect();
    files.sort();
    if files.is_empty() {
        return verdict(CheckStatus::Skip, "No .github/workflows/*.y{,a}ml files".into(), Severity::Info);
    }
    let mut hits = Vec::new();
    for path in &files {
        if let Ok(text) = fs::read_to_string(path) {
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            hits.extend(onegate_scan_workflow(&format!(".github/workflows/{name}"), &text));
        }
    }
    if hits.is_empty() {
        verdict(
            CheckStatus::Pass,
            format!("{} workflow(s) checked: PR job set matches push job set", files.len()),
            Severity::Info,
        )
    } else {
        verdict(
            CheckStatus::Fail,
            format!(
                "{} push-workflow job(s) split PR vs master via event_name/head_ref/labels: {}",
                hits.len(),
                hits.join(", ")
            ),
            Severity::Error,
        )
    }
}

#[cfg(test)]
mod check_individual_ci_tests {
    //! Covers check_individual_ci.rs CI/MSRV/Cargo.lock checks
    //! (28 uncov on broad, 0% cov).
    use super::*;

    // ── check_cargo_lock ──

    #[test]
    fn test_check_cargo_lock_no_cargo_toml_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_cargo_lock(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
        assert!(check.message.contains("Not a Rust project"));
    }

    #[test]
    fn test_check_cargo_lock_present_passes() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        std::fs::write(tmp.path().join("Cargo.lock"), "# auto-generated").unwrap();
        let check = check_cargo_lock(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("reproducible"));
    }

    #[test]
    fn test_check_cargo_lock_missing_fails() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"").unwrap();
        let check = check_cargo_lock(tmp.path());
        assert!(matches!(check.status, CheckStatus::Fail));
        assert!(check.message.contains("Missing Cargo.lock"));
    }

    // ── check_msrv ──

    #[test]
    fn test_check_msrv_no_cargo_toml_skips() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_msrv(tmp.path());
        assert!(matches!(check.status, CheckStatus::Skip));
    }

    #[test]
    fn test_check_msrv_with_rust_version_passes() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Cargo.toml"),
            "[package]\nname = \"x\"\nrust-version = \"1.85\"\n",
        )
        .unwrap();
        let check = check_msrv(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("rust-version"));
    }

    #[test]
    fn test_check_msrv_without_rust_version_warns() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Cargo.toml"), "[package]\nname = \"x\"\n").unwrap();
        let check = check_msrv(tmp.path());
        assert!(matches!(check.status, CheckStatus::Warn));
        assert!(check.message.contains("No rust-version"));
    }

    // ── check_ci_configured: 4 arms ──

    #[test]
    fn test_check_ci_configured_github_actions_workflows_pass() {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github").join("workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(wf.join("ci.yml"), "name: CI\n").unwrap();
        let check = check_ci_configured(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("GitHub Actions"));
    }

    #[test]
    fn test_check_ci_configured_gitlab_ci_pass() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join(".gitlab-ci.yml"), "stages: []\n").unwrap();
        let check = check_ci_configured(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("GitLab"));
    }

    #[test]
    fn test_check_ci_configured_jenkinsfile_pass() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Jenkinsfile"), "pipeline {}\n").unwrap();
        let check = check_ci_configured(tmp.path());
        assert!(matches!(check.status, CheckStatus::Pass));
        assert!(check.message.contains("Jenkins"));
    }

    #[test]
    fn test_check_ci_configured_no_ci_warns() {
        let tmp = tempfile::tempdir().unwrap();
        let check = check_ci_configured(tmp.path());
        assert!(matches!(check.status, CheckStatus::Warn));
        assert!(check.message.contains("No CI"));
    }

    #[test]
    fn test_check_ci_configured_empty_workflows_dir_falls_through_to_warn() {
        // .github/workflows exists but contains no files → falls through.
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github").join("workflows");
        std::fs::create_dir_all(&wf).unwrap();
        let check = check_ci_configured(tmp.path());
        // Empty workflows dir + no GitLab/Jenkins → warn.
        assert!(matches!(check.status, CheckStatus::Warn));
    }

    // ── check_onegate (#1870) ──

    fn run_onegate_fixture(yaml: &str) -> ComplianceCheck {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github").join("workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(wf.join("ci.yml"), yaml).unwrap();
        check_onegate(tmp.path())
    }

    #[test]
    fn test_onegate_fails_on_push_event_name_if() {
        let wf = "on: [push, pull_request]\njobs:\n  heavy:\n    if: github.event_name != 'pull_request'\n    runs-on: ubuntu-latest\n";
        let check = run_onegate_fixture(wf);
        assert_eq!(check.status, CheckStatus::Fail, "{}", check.message);
        assert_eq!(check.severity, Severity::Error);
        assert!(check.message.contains("ci.yml:heavy"), "{}", check.message);
    }

    #[test]
    fn test_onegate_passes_without_if() {
        let wf = "on: [push, pull_request]\njobs:\n  heavy:\n    runs-on: ubuntu-latest\n";
        let check = run_onegate_fixture(wf);
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn test_onegate_passes_on_draft_not_true() {
        let wf = "on: [push, pull_request]\njobs:\n  heavy:\n    if: github.event.pull_request.draft != true\n    runs-on: ubuntu-latest\n";
        let check = run_onegate_fixture(wf);
        assert_eq!(check.status, CheckStatus::Pass, "{}", check.message);
    }

    #[test]
    fn test_onegate_flags_head_ref_and_labels_and_ignores_pr_only() {
        let head_ref = "on:\n  push:\n    branches: [master]\njobs:\n  a:\n    if: startsWith(github.head_ref, 'train/')\n    runs-on: ubuntu-latest\n";
        assert_eq!(run_onegate_fixture(head_ref).status, CheckStatus::Fail);
        let labels = "on: push\njobs:\n  b:\n    if: contains(github.event.pull_request.labels.*.name, 'full-ci')\n    runs-on: ubuntu-latest\n";
        assert_eq!(run_onegate_fixture(labels).status, CheckStatus::Fail);
        let pr_only = "on: pull_request\njobs:\n  c:\n    if: startsWith(github.head_ref, 'train/')\n    runs-on: ubuntu-latest\n";
        assert_eq!(run_onegate_fixture(pr_only).status, CheckStatus::Pass);
    }
}
