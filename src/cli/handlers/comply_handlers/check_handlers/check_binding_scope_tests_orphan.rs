// Tests for CB-1600 orphan detection —
// included from check_binding_scope.rs.
// Do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_orphan {
    use super::*;
    use std::path::PathBuf;
    use tempfile::tempdir;

    fn write_contract(project: &Path, ticket: &str, yaml_file: &Path, equation: &str, sha: &str) {
        let mut c = WorkContract::new(ticket.to_string(), "deadbeef".to_string());
        c.implements.push(ContractBinding {
            contract: "k".to_string(),
            equation: equation.to_string(),
            file: yaml_file.to_path_buf(),
            sha: sha.to_string(),
            bound_at: chrono::Utc::now(),
        });
        c.save(project).unwrap();
    }

    // ── CB-1600 orphan-detection tests ───────────────────────────────────

    fn write_binding_index(project: &Path, entries: &[(&str, &[&str])]) {
        std::fs::create_dir_all(project.join(".pmat")).unwrap();
        let mut obj = serde_json::Map::new();
        for (file, names) in entries {
            let arr = names
                .iter()
                .map(|n| serde_json::Value::String((*n).to_string()))
                .collect::<Vec<_>>();
            obj.insert((*file).to_string(), serde_json::Value::Array(arr));
        }
        let json = serde_json::Value::Object(obj).to_string();
        std::fs::write(project.join(".pmat/binding-index.json"), json).unwrap();
    }

    /// Init a git repo in `project` and stage `files` (each created empty).
    fn init_repo_with_staged(project: &Path, files: &[&str]) {
        use std::process::Command;
        let run = |args: &[&str]| {
            let s = Command::new("git")
                .arg("-C")
                .arg(project)
                .args(args)
                .output()
                .unwrap();
            assert!(s.status.success(), "git {:?}: {:?}", args, s);
        };
        run(&["init", "--quiet", "--initial-branch=main"]);
        run(&["config", "user.email", "t@t"]);
        run(&["config", "user.name", "t"]);
        for f in files {
            let path = project.join(f);
            if let Some(p) = path.parent() {
                std::fs::create_dir_all(p).unwrap();
            }
            std::fs::write(&path, "").unwrap();
            run(&["add", f]);
        }
    }

    #[test]
    fn orphan_skip_when_no_binding_index() {
        let tmp = tempdir().unwrap();
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("binding-index.json"));
    }

    #[test]
    fn orphan_skip_when_no_staged_files() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        // No git repo → staged_files returns empty → Skip
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("No staged files"));
    }

    #[test]
    fn orphan_pass_when_staged_files_not_bound() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["README.md"]);
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("none intersect"));
    }

    #[test]
    fn orphan_fail_when_bound_file_staged_without_implements() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["src/rope.rs"]);
        // No `.pmat-work/*/contract.json` → no `implements:` coverage
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail, "{}", r.message);
        assert!(r.message.contains("src/rope.rs"));
        assert!(r.message.contains("rope"));
    }

    #[test]
    fn orphan_pass_when_bound_file_covered_by_implements() {
        let tmp = tempdir().unwrap();
        write_binding_index(tmp.path(), &[("src/rope.rs", &["rope"])]);
        init_repo_with_staged(tmp.path(), &["src/rope.rs"]);
        // Active ticket declares implements for src/rope.rs
        write_contract(
            tmp.path(),
            "T-1",
            &PathBuf::from("src/rope.rs"),
            "rope",
            "deadbeef",
        );
        let r = check_binding_scope_orphan(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("covered"));
    }
}
