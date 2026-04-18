// Tests for CB-1633 — manifest SHA drift detection.
// Included from check_codegen.rs — do NOT add `use` imports or `#!` attributes here.

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_manifest {
    use super::*;
    use tempfile::tempdir;

    // ── CB-1633 manifest SHA drift tests ─────────────────────────────────

    fn write_file(project: &Path, rel: &str, body: &[u8]) -> String {
        let p = project.join(rel);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, body).unwrap();
        sha256_hex(body)
    }

    fn write_manifest(project: &Path, name: &str, body: &str) {
        let dir = project.join("contracts/work");
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join(format!("{}.manifest.json", name)), body).unwrap();
    }

    #[test]
    fn manifest_sha_skip_when_no_dir() {
        let tmp = tempdir().unwrap();
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("contracts/work/"));
    }

    #[test]
    fn manifest_sha_skip_when_no_manifests() {
        let tmp = tempdir().unwrap();
        std::fs::create_dir_all(tmp.path().join("contracts/work")).unwrap();
        // A plain .rs module, not a manifest
        std::fs::write(tmp.path().join("contracts/work/PMAT-1.rs"), "// generated").unwrap();
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Skip);
        assert!(r.message.contains("manifest.json"));
    }

    #[test]
    fn manifest_sha_pass_when_all_match() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        let sha_b = write_file(tmp.path(), "src/b.rs", b"fn b(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(
                r#"{{"ticket":"PMAT-1","entries":[{{"path":"src/a.rs","sha":"{sha_a}"}},{{"path":"src/b.rs","sha":"{sha_b}"}}]}}"#
            ),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
        assert!(r.message.contains("1 manifest"));
        assert!(r.message.contains("2 entry"));
    }

    #[test]
    fn manifest_sha_accepts_files_alias() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        // Alternate naming — `files` instead of `entries`
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"files":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_accepts_sources_alias() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"sources":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_fails_on_drift() {
        let tmp = tempdir().unwrap();
        write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        // Recorded sha is stale (file content differs from recorded hash)
        write_manifest(
            tmp.path(),
            "PMAT-1",
            r#"{"entries":[{"path":"src/a.rs","sha":"deadbeef"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("drifted"));
        assert!(r.message.contains("src/a.rs"));
        assert!(r.message.contains("deadbeef"));
    }

    #[test]
    fn manifest_sha_fails_on_missing_file() {
        let tmp = tempdir().unwrap();
        write_manifest(
            tmp.path(),
            "PMAT-1",
            r#"{"entries":[{"path":"src/ghost.rs","sha":"deadbeef"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing"));
        assert!(r.message.contains("src/ghost.rs"));
    }

    #[test]
    fn manifest_sha_fails_on_malformed_json() {
        let tmp = tempdir().unwrap();
        write_manifest(tmp.path(), "PMAT-1", "not-json{");
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("not valid JSON"));
    }

    #[test]
    fn manifest_sha_fails_on_missing_entries_key() {
        let tmp = tempdir().unwrap();
        write_manifest(tmp.path(), "PMAT-1", r#"{"ticket":"PMAT-1"}"#);
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("missing `entries`"));
    }

    #[test]
    fn manifest_sha_case_insensitive_hex_match() {
        let tmp = tempdir().unwrap();
        let sha_upper = write_file(tmp.path(), "src/a.rs", b"fn a(){}").to_uppercase();
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"entries":[{{"path":"src/a.rs","sha":"{sha_upper}"}}]}}"#),
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Pass, "{}", r.message);
    }

    #[test]
    fn manifest_sha_aggregates_across_multiple_manifests() {
        let tmp = tempdir().unwrap();
        let sha_a = write_file(tmp.path(), "src/a.rs", b"fn a(){}");
        write_file(tmp.path(), "src/b.rs", b"fn b(){}");
        write_manifest(
            tmp.path(),
            "PMAT-1",
            &format!(r#"{{"entries":[{{"path":"src/a.rs","sha":"{sha_a}"}}]}}"#),
        );
        // Second manifest has stale hash → one drift
        write_manifest(
            tmp.path(),
            "PMAT-2",
            r#"{"entries":[{"path":"src/b.rs","sha":"cafebabe"}]}"#,
        );
        let r = check_manifest_sha_drift(tmp.path());
        assert_eq!(r.status, CheckStatus::Fail);
        assert!(r.message.contains("src/b.rs"));
        assert!(!r.message.contains("src/a.rs (drift)"));
    }
}
