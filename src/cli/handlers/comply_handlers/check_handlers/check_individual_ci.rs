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
}
