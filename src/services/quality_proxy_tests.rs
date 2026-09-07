#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_proxy_high_quality_code() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"/// A simple greeting function
/// Greet.
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Accepted));
        assert!(response.quality_report.passed);
    }

    #[tokio::test]
    async fn test_proxy_reject_satd() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"fn process() {
    // TODO: This needs to be implemented properly
    // FIXME: Critical bug here
    unimplemented!()
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Rejected));
        assert!(!response.quality_report.passed);
        assert!(response.quality_report.metrics.satd_count > 0);
    }

    #[tokio::test]
    async fn test_proxy_advisory_mode() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some(
                r#"pub fn undocumented() {
    println!("No docs");
}"#
                .to_string(),
            ),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Advisory,
            quality_config: QualityConfig::default(),
        };

        let response = service.proxy_operation(request).await.unwrap();
        assert!(matches!(response.status, ProxyStatus::Accepted));
        assert!(!response.quality_report.violations.is_empty());
    }

    #[test]
    fn test_get_operation_content() {
        let service = QualityProxyService::new();

        let write_request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some("write content".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let content = service.get_operation_content(&write_request).unwrap();
        assert_eq!(content, "write content");

        let edit_request = ProxyRequest {
            operation: ProxyOperation::Edit,
            file_path: "test.rs".to_string(),
            content: Some("original content here".to_string()),
            old_content: Some("original".to_string()),
            new_content: Some("modified".to_string()),
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };

        let content = service.get_operation_content(&edit_request).unwrap();
        assert_eq!(content, "modified content here");
    }

    #[test]
    fn test_quality_proxy_service_new() {
        // Verify service is created successfully
        let _service = QualityProxyService::new();
    }

    #[test]
    fn test_quality_proxy_service_default() {
        // Verify default impl works
        let _service = QualityProxyService::default();
    }

    #[test]
    fn test_get_operation_content_missing_content() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: None,
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let result = service.get_operation_content(&request);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_operation_content_append() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Append,
            file_path: "test.rs".to_string(),
            content: Some("appended".to_string()),
            old_content: Some("existing".to_string()),
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let content = service.get_operation_content(&request).unwrap();
        assert!(content.contains("existing"));
        assert!(content.contains("appended"));
    }

    #[test]
    fn test_get_operation_content_append_no_existing() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Append,
            file_path: "test.rs".to_string(),
            content: Some("appended content".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let content = service.get_operation_content(&request).unwrap();
        assert_eq!(content, "appended content");
    }

    #[test]
    fn test_get_operation_content_edit_missing_parts() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Edit,
            file_path: "test.rs".to_string(),
            content: Some("content".to_string()),
            old_content: Some("old".to_string()),
            new_content: None, // Missing new_content
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let result = service.get_operation_content(&request);
        assert!(result.is_err());
    }

    // === Polyglot gates ===
    //
    // These replace `test_proxy_non_rust_file`, which asserted the defect
    // rather than catching it: its body was a `.py` file in Strict mode with
    // the comment "Non-Rust files should pass with no violations", pinning
    // `Accepted` and `passed == true`. Every file whose extension was not the
    // literal lowercase `rs` returned from `analyze_content` immediately with
    // all-zero metrics and an empty violation list, so that assertion held for
    // ANY Python content — debt markers, unparseable text, anything.
    //
    // The debt markers in the fixtures below live in `#`-anchored comments,
    // which is what those languages actually use; the repository's own SATD
    // ratchet greps `//`-anchored lines, so the fixtures cannot inflate it.

    const PYTHON_WITH_DEBT: &str =
        "def hello(name):\n    # TODO: implement this properly\n    return name\n";

    const SHELL_WITH_DEBT: &str = "#!/bin/sh\n# FIXME: deletes the wrong path\nrm -rf \"$1\"\n";

    /// Python carrying a debt marker must be rejected under the default
    /// `allow_satd: false`, in the default Strict mode.
    #[tokio::test]
    async fn test_proxy_python_with_debt_marker_is_rejected() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "payload.py".to_string(),
            content: Some(PYTHON_WITH_DEBT.to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");

        assert!(
            matches!(response.status, ProxyStatus::Rejected),
            "a debt marker pmat's own `analyze satd` finds must not be accepted: {:?}",
            response.quality_report
        );
        assert!(!response.quality_report.passed);
        assert!(
            response.quality_report.metrics.satd_count > 0,
            "the marker was found, so it must be counted: {:?}",
            response.quality_report.metrics
        );
    }

    /// A clean Python file is accepted — and says which gates it was accepted
    /// by. `lint` and `docs` are Rust-only, so their absence from `gates_run`
    /// is the disclosure that `lint_violations: 0` is not a measurement.
    #[tokio::test]
    async fn test_proxy_clean_python_names_the_gates_that_ran() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "clean.py".to_string(),
            content: Some("def hello(name):\n    return name\n".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");
        let report = &response.quality_report;

        assert!(matches!(response.status, ProxyStatus::Accepted));
        assert_eq!(report.language, "python");
        assert!(
            report.gates_run.iter().any(|g| g == "satd"),
            "{:?}",
            report.gates_run
        );
        assert!(
            report.gates_run.iter().any(|g| g == "complexity"),
            "{:?}",
            report.gates_run
        );
        assert!(
            !report.gates_run.iter().any(|g| g == "lint"),
            "clippy cannot judge Python and must not be claimed: {:?}",
            report.gates_run
        );
        assert!(
            !report.gates_run.iter().any(|g| g == "docs"),
            "the `pub fn` doc scan cannot judge Python: {:?}",
            report.gates_run
        );
    }

    /// Shell scripts are `#`-comment territory too, and were passed the same
    /// silent way.
    #[tokio::test]
    async fn test_proxy_shell_with_debt_marker_is_rejected() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "deploy.sh".to_string(),
            content: Some(SHELL_WITH_DEBT.to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");

        assert!(
            matches!(response.status, ProxyStatus::Rejected),
            "{:?}",
            response.quality_report
        );
        assert_eq!(response.quality_report.language, "bash");
    }

    /// An extensionless file is `unknown`, never Rust. The extension lookup
    /// ended in `.unwrap_or("rs")`, so a `Makefile` was written into a temp
    /// crate's `src/lib.rs` and handed to `cargo clippy`, which rejected it
    /// with parse errors about content that was never Rust.
    #[tokio::test]
    async fn test_proxy_extensionless_file_is_not_treated_as_rust() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "Makefile".to_string(),
            content: Some("all:\n\techo hello\n".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Strict,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");
        let report = &response.quality_report;

        assert_eq!(report.language, "unknown");
        assert!(
            !report.gates_run.iter().any(|g| g == "lint"),
            "a Makefile must never reach cargo clippy: {:?}",
            report.gates_run
        );
        assert_eq!(
            report.gates_run,
            vec!["satd".to_string()],
            "an unknown language is scanned for debt markers and nothing else, \
             and the report must claim exactly that"
        );
        assert!(matches!(response.status, ProxyStatus::Accepted));
    }

    /// AutoFix on a language with no auto-fix must say why it did nothing.
    /// The skip used to return an empty plan, which reaches the caller as a
    /// bare `refactoring_applied: false` — the same shape as "the content was
    /// already fine".
    #[tokio::test]
    async fn test_autofix_on_python_records_why_it_did_nothing() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "payload.py".to_string(),
            content: Some(PYTHON_WITH_DEBT.to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::AutoFix,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");

        assert!(!response.refactoring_applied);
        let plan = response
            .refactoring_plan
            .expect("a skipped auto-fix must carry its reason");
        assert!(
            plan.iter().any(|step| {
                step.get("action").and_then(|v| v.as_str()) == Some("skipped")
                    && step.get("language").and_then(|v| v.as_str()) == Some("python")
            }),
            "{plan:?}"
        );
    }

    /// A parse failure is not a measurement of zero. The complexity arm used
    /// to be `Err(e) => { warn!(...); 0 }`, publishing `max_complexity: 0` with
    /// no violation at all, so a consumer could not tell unparseable content
    /// from trivial content. Advisory mode is used so the assertion is about
    /// the report rather than about the verdict, which clippy also influences.
    #[tokio::test]
    async fn test_unparseable_rust_reports_that_complexity_was_not_measured() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "broken.rs".to_string(),
            content: Some("this is not rust at all !!!".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::Advisory,
            quality_config: QualityConfig::default(),
        };
        let response = service
            .proxy_operation(request)
            .await
            .expect("proxy_operation runs");
        let report = &response.quality_report;

        assert!(
            !report.gates_run.iter().any(|g| g == "complexity"),
            "a gate that could not run must not be listed as having run: {:?}",
            report.gates_run
        );
        assert!(
            report.violations.iter().any(|v| {
                matches!(v.violation_type, ViolationType::Complexity)
                    && matches!(v.severity, ViolationSeverity::Error)
                    && v.message.contains("not measured")
            }),
            "the parse failure must be recorded, not swallowed to 0: {:?}",
            report.violations
        );
        assert!(
            !report.passed,
            "an Error violation means the content did not pass, whatever the mode does with that"
        );
    }

    #[tokio::test]
    async fn test_proxy_autofix_mode_simple() {
        let service = QualityProxyService::new();
        let request = ProxyRequest {
            operation: ProxyOperation::Write,
            file_path: "test.rs".to_string(),
            content: Some("fn simple() {}".to_string()),
            old_content: None,
            new_content: None,
            mode: ProxyMode::AutoFix,
            quality_config: QualityConfig::default(),
        };
        let response = service.proxy_operation(request).await.unwrap();
        // Simple code should pass without needing fixes
        assert!(matches!(response.status, ProxyStatus::Accepted));
    }

    #[test]
    fn test_quality_config_default() {
        let config = QualityConfig::default();
        assert!(config.max_complexity > 0);
        // Just verify defaults are reasonable
        // Just verify default is constructed without panic
        let _ = config.allow_satd;
    }

    #[test]
    fn test_proxy_operation_enum_debug() {
        let write = ProxyOperation::Write;
        let edit = ProxyOperation::Edit;
        let append = ProxyOperation::Append;

        assert!(format!("{:?}", write).contains("Write"));
        assert!(format!("{:?}", edit).contains("Edit"));
        assert!(format!("{:?}", append).contains("Append"));
    }

    #[test]
    fn test_proxy_mode_debug() {
        let strict = ProxyMode::Strict;
        let advisory = ProxyMode::Advisory;
        let autofix = ProxyMode::AutoFix;

        assert!(format!("{:?}", strict).contains("Strict"));
        assert!(format!("{:?}", advisory).contains("Advisory"));
        assert!(format!("{:?}", autofix).contains("AutoFix"));
    }

    #[test]
    fn test_proxy_status_variants() {
        let accepted = ProxyStatus::Accepted;
        let rejected = ProxyStatus::Rejected;
        let modified = ProxyStatus::Modified;

        assert!(format!("{:?}", accepted).contains("Accepted"));
        assert!(format!("{:?}", rejected).contains("Rejected"));
        assert!(format!("{:?}", modified).contains("Modified"));
    }

    #[test]
    fn test_violation_severity_ordering() {
        // Error should be more severe than Warning
        let error = ViolationSeverity::Error;
        let warning = ViolationSeverity::Warning;

        assert!(format!("{:?}", error).contains("Error"));
        assert!(format!("{:?}", warning).contains("Warning"));
    }

    #[test]
    fn test_quality_violation_creation() {
        let violation = QualityViolation {
            violation_type: ViolationType::Complexity,
            severity: ViolationSeverity::Error,
            location: "test.rs:10".to_string(),
            message: "Complexity too high".to_string(),
            suggestion: Some("Refactor function".to_string()),
        };

        assert_eq!(violation.location, "test.rs:10");
        assert!(matches!(
            violation.violation_type,
            ViolationType::Complexity
        ));
    }

    #[test]
    fn test_quality_metrics_default() {
        let metrics = QualityMetrics {
            max_complexity: 0,
            satd_count: 0,
            lint_violations: 0,
            coverage_percentage: None,
        };

        assert_eq!(metrics.max_complexity, 0);
        assert_eq!(metrics.satd_count, 0);
    }
}


#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod child_process_isolation_tests {
    //! PMAT-694 (#1202, absorbing #1127): what the quality proxy's child
    //! compilers are allowed to touch.
    //!
    //! `ci / coverage` runs `cargo llvm-cov` over `cargo test --lib`, which
    //! exports `RUSTFLAGS`, `LLVM_PROFILE_FILE`, `CARGO_LLVM_COV*` and a shared
    //! `CARGO_TARGET_DIR` into every process the test binary spawns. The proxy
    //! spawns a real `cargo clippy`; inheriting that environment made the child
    //! build instrumented, into a target directory a dozen sibling tests were
    //! already holding the package lock on, and the 600s deadline expired. Run
    //! 34020631941 on PR #1181 failed exactly there, on
    //! `test_proxy_advisory_mode` and a sibling, and two reruns went green — a
    //! rerun to green is a flake, not a pass.
    //!
    //! These four tests are the guard: a fixture crate the child can lint in
    //! seconds, an environment the child cannot inherit instrumentation
    //! through, an address-space cap on the child, and a wall-clock bound on
    //! the two tests that failed.
    use super::*;
    use std::ffi::OsStr;
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    /// The fixture crate the lint stage's temp crate is built from.
    const FIXTURE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/quality_proxy");

    /// The environment a child compiler must not inherit from a `cargo
    /// llvm-cov` parent. Instrumenting pmat must not instrument the child.
    const INSTRUMENTATION_VARS: [&str; 6] = [
        "LLVM_PROFILE_FILE",
        "CARGO_LLVM_COV",
        "CARGO_LLVM_COV_TARGET_DIR",
        "RUSTFLAGS",
        "CARGO_ENCODED_RUSTFLAGS",
        "CARGO_INCREMENTAL",
    ];

    fn rust_files(dir: &Path) -> Vec<PathBuf> {
        let mut found = Vec::new();
        let mut stack = vec![dir.to_path_buf()];
        while let Some(next) = stack.pop() {
            let Ok(entries) = std::fs::read_dir(&next) else {
                continue;
            };
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    stack.push(path);
                } else if path.extension() == Some(OsStr::new("rs")) {
                    found.push(path);
                }
            }
        }
        found.sort();
        found
    }

    /// (a) The crate the child lints is a fixture, not this workspace.
    ///
    /// `[workspace]` is the load-bearing line: without it cargo walks up from
    /// the crate directory, finds pmat's workspace root and lints 3,000+ files.
    #[test]
    fn fixture_crate_is_small() {
        let dir = Path::new(FIXTURE_DIR);
        let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
        assert!(
            !manifest.is_empty(),
            "the quality proxy fixture crate must exist at {FIXTURE_DIR}/Cargo.toml"
        );
        assert!(
            manifest.contains("[workspace]"),
            "the fixture must declare its own [workspace] so cargo cannot climb \
             into pmat's; manifest was:\n{manifest}"
        );
        assert!(
            dir.join("Cargo.lock").is_file(),
            "the fixture's Cargo.lock must be committed so the child never resolves a registry"
        );

        let files = rust_files(dir);
        assert!(!files.is_empty(), "the fixture must contain Rust to lint");
        let lines: usize = files
            .iter()
            .map(|f| std::fs::read_to_string(f).unwrap_or_default().lines().count())
            .sum();
        assert!(
            lines <= 50,
            "the fixture is a seconds-long lint, not a workspace: {lines} lines over {files:?}"
        );
    }

    /// (b) The child cannot inherit the parent's coverage instrumentation.
    ///
    /// Asserted on the constructed `Command`: `env_remove` records the variable
    /// with no value, so the clear is visible whatever the parent process has
    /// set, and the test does not have to mutate a process-global environment
    /// that 21,000 sibling tests share.
    #[test]
    fn child_env_is_scrubbed() {
        let workdir = std::env::temp_dir();
        let cmd = child_command("cargo", &[OsStr::new("clippy")], &workdir);
        let envs: Vec<(String, Option<String>)> = cmd
            .get_envs()
            .map(|(k, v)| {
                (
                    k.to_string_lossy().into_owned(),
                    v.map(|v| v.to_string_lossy().into_owned()),
                )
            })
            .collect();

        for var in INSTRUMENTATION_VARS {
            assert!(
                envs.iter().any(|(k, v)| k == var && v.is_none()),
                "the child must clear {var}; the command carries {envs:?}"
            );
        }

        let target = envs
            .iter()
            .find_map(|(k, v)| (k == "CARGO_TARGET_DIR").then(|| v.clone()))
            .flatten()
            .unwrap_or_default();
        assert!(
            !target.is_empty() && Path::new(&target).starts_with(&workdir),
            "the child needs a private CARGO_TARGET_DIR under its own working \
             directory, not the shared one a dozen sibling tests hold the \
             package lock on; got {target:?}"
        );
    }

    /// (c) The child has an address-space cap — #1127's ask.
    ///
    /// The deadline and the process-group kill bound how long a child runs and
    /// guarantee it dies; neither bounds how much memory it takes with it.
    #[test]
    fn child_has_a_memory_cap() {
        let line = child_command_line(&child_command(
            "cargo",
            &[OsStr::new("clippy")],
            &std::env::temp_dir(),
        ));
        assert!(
            line.contains("prlimit --as=") || line.contains("ulimit -v"),
            "the child compiler must run under an address-space cap; command line was: {line}"
        );
    }

    /// (d) The two tests that `ci / coverage` killed still judge the same
    /// content, and finish two orders of magnitude inside the deadline.
    ///
    /// 60s is not the target — the walls this prints are seconds — it is the
    /// bound below which the failure mode cannot recur: the killed runs sat at
    /// the 600s test-mode budget.
    #[tokio::test]
    async fn proxy_modes_finish_well_inside_the_deadline() {
        let service = QualityProxyService::new();

        let advisory_started = Instant::now();
        let advisory = service
            .proxy_operation(ProxyRequest {
                operation: ProxyOperation::Write,
                file_path: "test.rs".to_string(),
                content: Some("pub fn undocumented() {\n    println!(\"No docs\");\n}".to_string()),
                old_content: None,
                new_content: None,
                mode: ProxyMode::Advisory,
                quality_config: QualityConfig::default(),
            })
            .await
            .expect("advisory mode must produce a report, not an unmeasured lint stage");
        let advisory_wall = advisory_started.elapsed();
        assert!(matches!(advisory.status, ProxyStatus::Accepted));

        let strict_started = Instant::now();
        let strict = service
            .proxy_operation(ProxyRequest {
                operation: ProxyOperation::Write,
                file_path: "test.rs".to_string(),
                content: Some(
                    "/// A simple greeting function\n/// Greet.\npub fn greet(name: &str) -> \
                     String {\n    format!(\"Hello, {}!\", name)\n}"
                        .to_string(),
                ),
                old_content: None,
                new_content: None,
                mode: ProxyMode::Strict,
                quality_config: QualityConfig::default(),
            })
            .await
            .expect("strict mode must produce a report, not an unmeasured lint stage");
        let strict_wall = strict_started.elapsed();
        assert!(matches!(strict.status, ProxyStatus::Accepted));

        eprintln!(
            "PMAT-694 wall: advisory {:.2}s, strict {:.2}s",
            advisory_wall.as_secs_f64(),
            strict_wall.as_secs_f64()
        );
        for (name, wall) in [("advisory", advisory_wall), ("strict", strict_wall)] {
            assert!(
                wall.as_secs() < 60,
                "{name} mode took {:.2}s; the lint stage is meant to be one rustc \
                 invocation over one file",
                wall.as_secs_f64()
            );
        }
    }
}
