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

