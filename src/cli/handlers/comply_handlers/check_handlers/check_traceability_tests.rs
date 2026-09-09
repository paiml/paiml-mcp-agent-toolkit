// Included from check.rs — do NOT add `use` imports or `#!` attributes here.
#[cfg(all(test, not(coverage_nightly)))]
mod tests_traceability {
    use super::*;
    use std::process::Command;
    use crate::models::comply_config::CheckSeverity;

    fn git(dir: &std::path::Path, args: &[&str]) {
        let out = Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.name=t", "-c", "user.email=t@example.invalid", "-c", "commit.gpgsign=false"])
            .args(args)
            .env("LC_ALL", "C")
            .output()
            .expect("git runs");
        assert!(out.status.success(), "git failed: {}", String::from_utf8_lossy(&out.stderr));
    }

    #[test]
    fn test_plain_tempdir() {
        let dir = tempfile::tempdir().expect("tempdir");
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].status, CheckStatus::Skip);
        assert!(checks[0].message.contains("not a git repository"));
    }

    #[test]
    fn test_no_roadmap() {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q", "-b", "master"]);
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks[0].status, CheckStatus::Skip);
        assert!(checks[0].message.contains("roadmap.yaml"));
    }

    fn repo_with_roadmap() -> tempfile::TempDir {
        let dir = tempfile::tempdir().expect("tempdir");
        git(dir.path(), &["init", "-q", "-b", "master"]);
        let rm = dir.path().join("docs/roadmaps/roadmap.yaml");
        std::fs::create_dir_all(rm.parent().expect("parent")).expect("mkdir");
        let content = "roadmap_version: \"1.0\"\ngithub_enabled: false\ngithub_repo: null\nroadmap:\n  - id: PMAT-001\n    title: planned work\n    status: planned\n";
        std::fs::write(&rm, content).expect("write roadmap");
        git(dir.path(), &["add", "."]);
        git(dir.path(), &["commit", "-q", "-m", "roadmap"]);
        git(dir.path(), &["tag", "v0.1.0"]);
        dir
    }

    #[test]
    fn test_fixture_fail() {
        let dir = repo_with_roadmap();
        git(dir.path(), &["switch", "-q", "-c", "feature"]);
        git(dir.path(), &["commit", "-q", "--allow-empty", "-m", "no trailer here"]);
        let hash = std::str::from_utf8(&Command::new("git").arg("-C").arg(dir.path()).args(["rev-parse", "HEAD"]).output().expect("git runs").stdout).expect("utf-8").trim().to_string();
        
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks[0].status, CheckStatus::Fail);
        assert_eq!(checks[0].severity, CheckSeverity::Error.into());
        assert!(checks[0].message.contains(&hash[..7]));
        assert!(checks[0].message.contains("no Pmat-Ticket"));
    }

    #[test]
    fn test_fixture_pass() {
        let dir = repo_with_roadmap();
        git(dir.path(), &["switch", "-q", "-c", "feature"]);
        git(dir.path(), &["commit", "-q", "--allow-empty", "-m", "feat: x", "-m", "Pmat-Ticket: PMAT-001"]);
        
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert!(checks[0].message.contains("carry a Pmat-Ticket"));
    }

    #[test]
    fn test_fixture_master() {
        let dir = repo_with_roadmap();
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks[0].status, CheckStatus::Pass);
        assert!(checks[0].message.starts_with("not_applicable"));
    }

    #[test]
    fn test_build_traceability_checks_name() {
        let dir = repo_with_roadmap();
        let checks = build_traceability_checks(dir.path(), &crate::models::comply_config::ComplyConfig::default());
        assert_eq!(checks.len(), 1);
        assert_eq!(checks[0].name, "CB-2113: Commit Traceability");
    }
}
