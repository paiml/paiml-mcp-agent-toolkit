// CB-533: Stale Path References
// Detects paths in Makefiles and CI YAML that reference non-existent files/directories.
// Derived from empirical analysis: 20/241 fix commits (8%) were stale path fixes.

use super::types::{CbPatternViolation, Severity};
use std::path::Path;

/// CB-533: Detect stale path references in Makefiles and CI workflows.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb533_stale_path_references(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // 1. Check Makefile
    check_makefile(project_path, &mut violations);

    // 2. Check CI workflows
    check_ci_workflows(project_path, &mut violations);

    violations
}

fn check_makefile(project_path: &Path, violations: &mut Vec<CbPatternViolation>) {
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&makefile) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Check `cd <dir>` patterns
        if let Some(pos) = trimmed.find("cd ") {
            let rest = &trimmed[pos + 3..];
            let dir = rest.split([';', '&', ' ', '/']).next().unwrap_or("");
            if !dir.is_empty()
                && !dir.starts_with('$')
                && !dir.starts_with('-')
                && dir != "."
                && dir != ".."
            {
                let target = project_path.join(dir);
                if !target.exists() {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-533".to_string(),
                        file: "Makefile".to_string(),
                        line: i + 1,
                        description: format!(
                            "Stale path: `cd {dir}` references non-existent directory"
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}

fn check_ci_workflows(project_path: &Path, violations: &mut Vec<CbPatternViolation>) {
    let workflows_dir = project_path.join(".github/workflows");
    if !workflows_dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&workflows_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_none_or(|e| e != "yml" && e != "yaml") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check working-directory references
            if let Some(pos) = trimmed.find("working-directory:") {
                let dir = trimmed[pos + 18..]
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                if !dir.starts_with('$') && !dir.starts_with('.') {
                    let target = project_path.join(dir);
                    if !target.exists() {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-533".to_string(),
                            file: format!(".github/workflows/{filename}"),
                            line: i + 1,
                            description: format!(
                                "Stale path: `working-directory: {dir}` references non-existent directory"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod stale_paths_tests {
    //! Covers detect_cb533_stale_path_references + check_makefile + check_ci_workflows
    //! (81 uncov on broad, 0% cov).
    use super::*;

    // ── project without Makefile or .github: no violations ──

    #[test]
    fn test_detect_empty_project_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        let v = detect_cb533_stale_path_references(tmp.path());
        assert!(v.is_empty());
    }

    // ── check_makefile: comment/empty/cd . / cd .. / cd $VAR / cd -p all skipped ──

    #[test]
    fn test_check_makefile_no_cd_no_violations() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Makefile"), "all:\n\techo hello\n").unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_makefile_cd_to_missing_dir_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "build:\n\tcd nonexistent && cargo build\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-533");
        assert_eq!(v[0].file, "Makefile");
        assert!(v[0].description.contains("nonexistent"));
    }

    #[test]
    fn test_check_makefile_cd_to_existing_dir_not_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::create_dir(tmp.path().join("src")).unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "build:\n\tcd src && cargo build\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_makefile_cd_dot_and_dotdot_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "t:\n\tcd . && echo ok\n\tcd .. && echo ok\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty(), "'.' and '..' must be skipped");
    }

    #[test]
    fn test_check_makefile_cd_variable_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "t:\n\tcd $(PROJECT) && echo ok\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty(), "`$` prefix must be skipped");
    }

    #[test]
    fn test_check_makefile_cd_flag_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(tmp.path().join("Makefile"), "t:\n\tcd -p some/dir\n").unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        // `-p` starts with '-' → skipped.
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_makefile_comment_lines_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "# cd nonexistent\n\ntarget:\n\techo ok\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_makefile_missing_makefile_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let mut v = Vec::new();
        check_makefile(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    // ── check_ci_workflows: missing dir / non-yaml / working-directory ──

    #[test]
    fn test_check_ci_workflows_missing_dir_is_noop() {
        let tmp = tempfile::tempdir().unwrap();
        let mut v = Vec::new();
        check_ci_workflows(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_ci_workflows_working_directory_to_missing_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(
            wf.join("ci.yml"),
            "jobs:\n  build:\n    steps:\n      - run: cargo test\n        working-directory: nonexistent_dir\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_ci_workflows(tmp.path(), &mut v);
        assert_eq!(v.len(), 1);
        assert_eq!(v[0].pattern_id, "CB-533");
        assert!(v[0].file.contains("ci.yml"));
    }

    #[test]
    fn test_check_ci_workflows_working_directory_to_existing_not_flagged() {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::create_dir(tmp.path().join("sub")).unwrap();
        std::fs::write(
            wf.join("ci.yml"),
            "jobs:\n  build:\n    working-directory: sub\n",
        )
        .unwrap();
        let mut v = Vec::new();
        check_ci_workflows(tmp.path(), &mut v);
        assert!(v.is_empty());
    }

    #[test]
    fn test_check_ci_workflows_non_yaml_files_ignored() {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(wf.join("README.md"), "working-directory: missing_dir\n").unwrap();
        let mut v = Vec::new();
        check_ci_workflows(tmp.path(), &mut v);
        assert!(v.is_empty(), "non-yaml workflow files must be skipped");
    }

    #[test]
    fn test_check_ci_workflows_working_directory_dot_prefix_skipped() {
        let tmp = tempfile::tempdir().unwrap();
        let wf = tmp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(wf.join("ci.yaml"), "working-directory: ./relative\n").unwrap();
        let mut v = Vec::new();
        check_ci_workflows(tmp.path(), &mut v);
        assert!(v.is_empty(), "'./' prefix must be skipped");
    }

    // ── Integration: detect_cb533 combines both checks ──

    #[test]
    fn test_detect_cb533_combines_makefile_and_ci_violations() {
        let tmp = tempfile::tempdir().unwrap();
        std::fs::write(
            tmp.path().join("Makefile"),
            "t:\n\tcd missing_makefile_dir\n",
        )
        .unwrap();
        let wf = tmp.path().join(".github/workflows");
        std::fs::create_dir_all(&wf).unwrap();
        std::fs::write(wf.join("ci.yml"), "working-directory: missing_ci_dir\n").unwrap();
        let violations = detect_cb533_stale_path_references(tmp.path());
        assert_eq!(violations.len(), 2);
        assert!(violations
            .iter()
            .any(|v| v.description.contains("missing_makefile_dir")));
        assert!(violations
            .iter()
            .any(|v| v.description.contains("missing_ci_dir")));
    }
}
