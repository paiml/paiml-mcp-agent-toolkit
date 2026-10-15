#[cfg(test)]
mod armed_gates_tests {
    //! ONT-11 (#1422, ONT-001 R-21 (c)): CB-2118 fails when an armed gate or
    //! shape is dropped between the merge-base and HEAD, and never asks for
    //! anything to be armed.
    use super::*;

    const BASE: &str = r#"{"armed_gates":["validate","audit"],"armed_shapes":["ont-shapes-v1"]}"#;

    #[test]
    fn ont11_dropping_an_armed_gate_fails_naming_it() {
        let head = r#"{"armed_gates":["validate"],"armed_shapes":["ont-shapes-v1"]}"#;
        let (status, msg) = judge_armed_gates(Some(BASE), Some(head));
        assert_eq!(status, CheckStatus::Fail, "{msg}");
        assert!(msg.contains("armed_gates: audit"), "{msg}");
        assert!(!msg.contains("validate"), "{msg}");
    }

    #[test]
    fn ont11_dropping_an_armed_shape_fails_naming_it() {
        let head = r#"{"armed_gates":["validate","audit"],"armed_shapes":[]}"#;
        let (status, msg) = judge_armed_gates(Some(BASE), Some(head));
        assert_eq!(status, CheckStatus::Fail, "{msg}");
        assert!(msg.contains("armed_shapes: ont-shapes-v1"), "{msg}");
    }

    #[test]
    fn ont11_deleting_the_baseline_file_drops_every_gate() {
        let (status, msg) = judge_armed_gates(Some(BASE), None);
        assert_eq!(status, CheckStatus::Fail, "{msg}");
        assert!(msg.contains("armed_gates: validate") && msg.contains("armed_gates: audit"), "{msg}");
        // The key-absent default is for a baseline that exists, not a deleted one.
        assert!(msg.contains("armed_shapes: ont-shapes-v1"), "{msg}");
        let shapes_only = r#"{"armed_shapes":["ont-shapes-v1"]}"#;
        let (status, msg) = judge_armed_gates(Some(shapes_only), None);
        assert_eq!(status, CheckStatus::Fail, "{msg}");
        assert!(msg.contains("armed_shapes: ont-shapes-v1"), "{msg}");
    }

    #[test]
    fn ont11_growing_or_keeping_arming_passes() {
        let head = r#"{"armed_gates":["audit","validate","score"],"armed_shapes":["ont-shapes-v1","ladder-green"]}"#;
        assert_eq!(judge_armed_gates(Some(BASE), Some(head)).0, CheckStatus::Pass);
        assert_eq!(judge_armed_gates(Some(BASE), Some(BASE)).0, CheckStatus::Pass);
    }

    #[test]
    fn ont11_removing_the_armed_shapes_key_arms_every_shape() {
        let head = r#"{"armed_gates":["validate","audit"]}"#;
        assert_eq!(judge_armed_gates(Some(BASE), Some(head)).0, CheckStatus::Pass);
    }

    #[test]
    fn ont11_a_repo_that_never_armed_passes() {
        let (status, msg) = judge_armed_gates(None, None);
        assert_eq!(status, CheckStatus::Pass);
        assert!(msg.contains("never armed"), "{msg}");
        assert_eq!(judge_armed_gates(Some("{}"), Some("{}")).0, CheckStatus::Pass);
    }

    #[test]
    fn ont11_an_unparseable_baseline_fails() {
        let (status, msg) = judge_armed_gates(Some(BASE), Some("{not json"));
        assert_eq!(status, CheckStatus::Fail);
        assert!(msg.contains("not JSON"), "{msg}");
    }

    #[test]
    fn ont11_no_contracts_dir_is_skip_with_a_decline() {
        let dir = tempfile::tempdir().expect("tempdir");
        let c = check_armed_gates_monotone(dir.path());
        assert_eq!(c.status, CheckStatus::Skip);
        assert_eq!(c.message, "decline: no contracts/");
        assert_eq!(c.name, "CB-2118: contracts-armed-gates-monotone");
    }

    fn git(dir: &Path, args: &[&str]) {
        let ok = std::process::Command::new("git")
            .arg("-C")
            .arg(dir)
            .args(["-c", "user.email=a@b", "-c", "user.name=a", "-c", "core.hooksPath=/dev/null"])
            .args(args)
            .env_remove("GITHUB_BASE_REF")
            .status()
            .expect("git")
            .success();
        assert!(ok, "git {args:?}");
    }

    #[test]
    fn ont11_a_branch_that_drops_a_gate_fails_against_its_merge_base() {
        let dir = tempfile::tempdir().expect("tempdir");
        let p = dir.path();
        std::fs::create_dir_all(p.join("contracts")).expect("mkdir");
        std::fs::write(p.join(ARMED_GATES_BASELINE), BASE).expect("write");
        git(p, &["init", "-q", "-b", "master"]);
        git(p, &["add", "."]);
        git(p, &["commit", "-qm", "base"]);
        git(p, &["checkout", "-qb", "topic"]);
        std::fs::write(p.join(ARMED_GATES_BASELINE), r#"{"armed_gates":["validate"],"armed_shapes":["ont-shapes-v1"]}"#)
            .expect("write");
        // The base ref is pinned, not read from this process's environment:
        // on a CI runner GITHUB_BASE_REF names the PR's base, not this fixture's.
        let c = armed_gates_monotone_against(p, None);
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("armed_gates: audit"), "{}", c.message);
        // A pull request's base, as GITHUB_BASE_REF names it, finds the same merge-base.
        let c = armed_gates_monotone_against(p, Some("master".into()));
        assert_eq!(c.status, CheckStatus::Fail, "{}", c.message);
        assert!(c.message.contains("armed_gates: audit"), "{}", c.message);
    }

    /// F-31 (ONT-001 R-21): pmat reads pv's verdict through its exit code and
    /// `contracts/lint-baseline.json`, never through pv's library. The needle is
    /// assembled so this file does not match itself.
    #[test]
    fn ont11_f31_no_source_file_uses_the_contracts_library() {
        let needle = concat!("aprender", "_contracts", "::");
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        let mut hits = vec![];
        let mut stack = vec![root.join("src")];
        let mut scanned = 0usize;
        while let Some(dir) = stack.pop() {
            for entry in std::fs::read_dir(&dir).expect("read_dir").flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension().is_some_and(|e| e == "rs") {
                    scanned += 1;
                    let text = std::fs::read_to_string(&path).unwrap_or_default();
                    if text.contains(needle) {
                        hits.push(path.display().to_string());
                    }
                }
            }
        }
        assert!(scanned > 100, "scanned only {scanned} files under src/ — the walk is broken");
        assert!(hits.is_empty(), "pmat must not link pv's library (F-31): {hits:?}");
    }
}
