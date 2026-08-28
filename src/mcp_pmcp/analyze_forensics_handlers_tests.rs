// Behavioural tests for the three analyzers #1029 found CLI-only.
//
// Registering a tool is only half of exposing it: a handler that answers
// `{"orphan_count": 0}` over a tree it never walked would satisfy every parity
// check in `cli::analyze_mcp_exposure_tests` while telling an agent the
// opposite of the truth. So these tests assert the two properties that make the
// exposure worth having — the tool measures something, and it REFUSES rather
// than reports a clean result when it cannot.

#[cfg(test)]
mod forensics_tool_tests {
    use super::{HardcodedPathsTool, ReachabilityTool, VacuousTestsTool};
    use pmcp::{RequestHandlerExtra, ToolHandler};
    use serde_json::json;
    use std::path::Path;
    use tokio_util::sync::CancellationToken;

    fn extra() -> RequestHandlerExtra {
        RequestHandlerExtra::new("test-request".to_string(), CancellationToken::new())
    }

    /// A git repository whose index lists `files`, since all three analyzers
    /// enumerate their scope with `git ls-files` and would otherwise measure
    /// nothing.
    fn git_fixture(dir: &Path, files: &[(&str, &str)]) {
        let run = |args: &[&str]| {
            let _ = std::process::Command::new("git")
                .arg("-C")
                .arg(dir)
                .args(args)
                .output();
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "fixture@example.invalid"]);
        run(&["config", "user.name", "fixture"]);
        for (rel, body) in files {
            let path = dir.join(rel);
            if let Some(parent) = path.parent() {
                let _ = std::fs::create_dir_all(parent);
            }
            std::fs::write(&path, body).expect("write fixture file");
        }
        run(&["add", "-A"]);
    }

    // === analyze_reachability ===============================================

    /// A missing root must be refused, not reported as a clean tree.
    ///
    /// This is GH #639's failure mode: an MCP client has no exit code, so
    /// `{"orphan_count": 0}` over a path that does not exist is
    /// indistinguishable from a passing repository.
    #[tokio::test]
    async fn reachability_refuses_a_root_that_does_not_exist() {
        let err = ReachabilityTool::new()
            .handle(json!({"project_path": "/definitely/not/here/pmat-1029"}), extra())
            .await
            .err();
        let message = format!("{err:?}");
        assert!(err.is_some(), "a nonexistent root answered successfully");
        assert!(
            message.contains("not found"),
            "the refusal must name the problem, got: {message}"
        );
    }

    /// A file is refused too: `cargo metadata` from a file yields nothing, and
    /// nothing reads as clean.
    #[tokio::test]
    async fn reachability_refuses_a_file_where_a_root_is_required() {
        let dir = tempfile::tempdir().expect("tempdir");
        let file = dir.path().join("lib.rs");
        std::fs::write(&file, "pub fn f() {}\n").expect("write");
        let err = ReachabilityTool::new()
            .handle(json!({"project_path": file.to_string_lossy()}), extra())
            .await
            .err();
        assert!(err.is_some(), "a file answered as if it were a project root");
        assert!(
            format!("{err:?}").contains("directory"),
            "the refusal must say a directory is required"
        );
    }

    /// A tree with no cargo targets is refused rather than graded 0 orphans.
    #[tokio::test]
    async fn reachability_refuses_a_tree_with_no_cargo_targets() {
        let dir = tempfile::tempdir().expect("tempdir");
        git_fixture(dir.path(), &[("README.md", "no crate here\n")]);
        let err = ReachabilityTool::new()
            .handle(json!({"project_path": dir.path().to_string_lossy()}), extra())
            .await
            .err();
        assert!(
            err.is_some(),
            "a directory with no cargo targets was reported as measured"
        );
        assert!(
            format!("{err:?}").contains("not a clean result"),
            "the refusal must say so in as many words"
        );
    }

    // === analyze_vacuous_tests ==============================================

    /// The tool finds a test that cannot fail, and answers with the CLI's field
    /// names.
    ///
    /// The field names matter as much as the finding: two surfaces answering
    /// the same question with different keys is the CLI-vs-MCP contradiction
    /// the round-5 dogfood catalogued 24 of, and avoiding one more of those is
    /// the reason this tool exists.
    #[tokio::test]
    async fn vacuous_tests_tool_reports_a_test_that_cannot_fail() {
        let dir = tempfile::tempdir().expect("tempdir");
        git_fixture(
            dir.path(),
            &[(
                "src/lib.rs",
                "pub fn add(a: i32, b: i32) -> i32 { a + b }\n\
                 #[cfg(test)]\n\
                 mod t {\n\
                 #[test]\n\
                 fn nothing_is_asserted() {\n\
                 let _ = super::add(1, 2);\n\
                 }\n\
                 }\n",
            )],
        );

        let out = VacuousTestsTool::new()
            .handle(json!({"project_path": dir.path().to_string_lossy()}), extra())
            .await
            .expect("vacuous-test scan over a real fixture");

        for key in ["vacuous", "tests_examined", "files_parsed", "skipped"] {
            assert!(
                out.get(key).is_some(),
                "the MCP payload is missing `{key}`, which `pmat analyze vacuous-tests \
                 --format json` emits — the two surfaces must answer with the same keys"
            );
        }
        assert_eq!(
            out["tests_examined"].as_u64(),
            Some(1),
            "the fixture holds exactly one #[test]; payload: {out}"
        );
        assert_eq!(
            out["vacuous"].as_array().map(Vec::len),
            Some(1),
            "a test whose body asserts nothing must be reported; payload: {out}"
        );
    }

    /// A tree with no `#[test]` at all is refused: zero vacuous out of zero
    /// tests is not a pass.
    #[tokio::test]
    async fn vacuous_tests_tool_refuses_a_tree_with_no_tests() {
        let dir = tempfile::tempdir().expect("tempdir");
        git_fixture(dir.path(), &[("src/lib.rs", "pub fn f() -> i32 { 1 }\n")]);
        let err = VacuousTestsTool::new()
            .handle(json!({"project_path": dir.path().to_string_lossy()}), extra())
            .await
            .err();
        assert!(
            err.is_some(),
            "0 vacuous tests out of 0 tests was reported as a clean result"
        );
        assert!(
            format!("{err:?}").contains("not a clean result"),
            "the refusal must say so in as many words"
        );
    }

    // === analyze_hardcoded_paths ============================================

    /// The tool finds a machine-specific path, and answers with the CLI's field
    /// names.
    ///
    /// The offending literal is assembled at runtime rather than written here,
    /// so this file does not itself become a finding of the analyzer it tests.
    #[tokio::test]
    async fn hardcoded_paths_tool_reports_a_machine_specific_path() {
        let dir = tempfile::tempdir().expect("tempdir");
        let leaked = format!("/{}/{}/{}", "home", "fixtureuser", "prod");
        let source = format!("pub fn root() -> &'static str {{ \"{leaked}\" }}\n");
        git_fixture(dir.path(), &[("src/lib.rs", source.as_str())]);

        let out = HardcodedPathsTool::new()
            .handle(json!({"project_path": dir.path().to_string_lossy()}), extra())
            .await
            .expect("hardcoded-path scan over a real fixture");

        for key in [
            "summary",
            "files_scanned",
            "literals_scanned",
            "finding_count",
            "shipped_count",
            "by_kind",
            "skipped",
            "findings",
        ] {
            assert!(
                out.get(key).is_some(),
                "the MCP payload is missing `{key}`, which `pmat analyze hardcoded-paths \
                 --format json` emits — the two surfaces must answer with the same keys"
            );
        }
        assert_eq!(
            out["finding_count"].as_u64(),
            Some(1),
            "the planted path was not found; payload: {out}"
        );
        assert_eq!(
            out["shipped_count"].as_u64(),
            Some(1),
            "a literal in src/lib.rs reaches a shipped artifact; payload: {out}"
        );
    }

    /// A directory git does not track is refused, not scanned to zero.
    #[tokio::test]
    async fn hardcoded_paths_tool_refuses_an_untracked_tree() {
        let dir = tempfile::tempdir().expect("tempdir");
        // A git repo with an EMPTY index: `git ls-files` succeeds and returns
        // nothing, which is the case that reads as clean.
        let _ = std::process::Command::new("git")
            .arg("-C")
            .arg(dir.path())
            .args(["init", "-q"])
            .output();
        std::fs::write(dir.path().join("untracked.rs"), "pub fn f() {}\n").expect("write");

        let err = HardcodedPathsTool::new()
            .handle(json!({"project_path": dir.path().to_string_lossy()}), extra())
            .await
            .err();
        assert!(
            err.is_some(),
            "a tree with nothing tracked was reported as scanned"
        );
        assert!(
            format!("{err:?}").contains("not a clean result"),
            "the refusal must say so in as many words"
        );
    }

    // === schema =============================================================

    /// All three advertise a `project_path`, required, with a description.
    ///
    /// pmcp's default `metadata()` is `None`, which the builder silently turns
    /// into an empty description and an empty schema — a tool an agent can see
    /// but cannot call. Registering without checking this is how three tools
    /// would have been "exposed" and still unusable.
    #[test]
    fn all_three_advertise_a_usable_schema() {
        let tools: Vec<(&str, Option<pmcp::types::ToolInfo>)> = vec![
            ("analyze_reachability", ReachabilityTool::new().metadata()),
            (
                "analyze_hardcoded_paths",
                HardcodedPathsTool::new().metadata(),
            ),
            ("analyze_vacuous_tests", VacuousTestsTool::new().metadata()),
        ];
        for (name, info) in tools {
            let Some(info) = info else {
                unreachable!("{name}: metadata() is None — tools/list would advertise nothing")
            };
            assert_eq!(info.name, name);
            assert!(
                info.description
                    .as_deref()
                    .is_some_and(|d| d.trim().len() > 20),
                "{name}: description must say what the analyzer does"
            );
            assert_eq!(
                info.input_schema["required"],
                json!(["project_path"]),
                "{name}: project_path must be required — defaulting it to `.` would let a \
                 caller measure the server's working directory by accident"
            );
            assert!(
                info.input_schema["properties"]["project_path"]["description"]
                    .as_str()
                    .is_some_and(|d| !d.is_empty()),
                "{name}: project_path must be described"
            );
        }
    }
}
