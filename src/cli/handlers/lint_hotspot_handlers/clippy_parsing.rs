// clippy_parsing.rs - Clippy output parsing and diagnostic processing
// Included by clippy.rs via include!()

fn parse_clippy_output(
    stdout: &[u8],
    abs_file_path: &Path,
    file_path: &Path,
) -> Result<(
    Vec<ViolationDetail>,
    Vec<ViolationDetail>,
    SeverityDistribution,
)> {
    let reader = BufReader::new(stdout);
    let mut file_violations = Vec::new();
    let mut all_violations = Vec::new();
    let mut severity_dist = SeverityDistribution::default();

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Some(violation) = parse_clippy_line(&line, abs_file_path, file_path)? {
            file_violations.push(violation.clone());
            update_severity_distribution(&mut severity_dist, &violation.severity);
            all_violations.push(violation);
        }
    }

    Ok((file_violations, all_violations, severity_dist))
}

fn parse_clippy_line(
    line: &str,
    abs_file_path: &Path,
    file_path: &Path,
) -> Result<Option<ViolationDetail>> {
    let msg = match serde_json::from_str::<ClippyMessage>(line) {
        Ok(msg) => msg,
        Err(_) => return Ok(None),
    };

    let (Some("compiler-message"), Some(diagnostic)) = (msg.reason.as_deref(), &msg.message) else {
        return Ok(None);
    };

    let Some(span) = find_primary_span(diagnostic) else {
        return Ok(None);
    };

    if !is_target_file(&span.file_name, abs_file_path, file_path) {
        return Ok(None);
    }

    Ok(Some(create_violation_detail(file_path, span, diagnostic)))
}

fn find_primary_span(diagnostic: &DiagnosticMessage) -> Option<&DiagnosticSpan> {
    diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary || diagnostic.spans.len() == 1)
}

fn is_target_file(diagnostic_file: &str, abs_file_path: &Path, file_path: &Path) -> bool {
    let diagnostic_path = PathBuf::from(diagnostic_file);
    diagnostic_path == *abs_file_path
        || diagnostic_path == *file_path
        || diagnostic_path.ends_with(file_path)
}

fn create_violation_detail(
    file_path: &Path,
    span: &DiagnosticSpan,
    diagnostic: &DiagnosticMessage,
) -> ViolationDetail {
    ViolationDetail {
        file: file_path.to_path_buf(),
        line: span.line_start,
        column: span.column_start,
        end_line: span.line_end,
        end_column: span.column_end,
        lint_name: extract_lint_name(diagnostic),
        message: diagnostic.message.clone(),
        severity: diagnostic.level.clone(),
        suggestion: span.suggested_replacement.clone(),
        machine_applicable: is_machine_applicable(span),
    }
}

fn extract_lint_name(diagnostic: &DiagnosticMessage) -> String {
    diagnostic
        .code
        .as_ref()
        .map(|c| c.code.clone())
        .unwrap_or_default()
}

fn is_machine_applicable(span: &DiagnosticSpan) -> bool {
    span.suggestion_applicability
        .as_ref()
        .is_some_and(|a| a == "machine-applicable" || a == "maybe-incorrect")
}

fn update_severity_distribution(severity_dist: &mut SeverityDistribution, level: &str) {
    match level {
        "error" => severity_dist.error += 1,
        "warning" => severity_dist.warning += 1,
        _ => severity_dist.note += 1,
    }
}

/// Process a diagnostic message
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn process_diagnostic(
    diagnostic: &DiagnosticMessage,
    file_metrics: &mut HashMap<PathBuf, FileMetrics>,
) {
    // Find the primary span
    let primary_span = diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary)
        .or_else(|| diagnostic.spans.first());

    if let Some(span) = primary_span {
        let mut file_path = PathBuf::from(&span.file_name);

        // Handle workspace paths - if path starts with "server/", strip it for consistent handling
        // But preserve the original path structure for examples
        if let Ok(stripped) = file_path.strip_prefix("server/") {
            file_path = PathBuf::from(stripped);
        } else if file_path.starts_with("examples/") {
            // Keep examples/ paths as-is since they are relative to server/
            file_path = PathBuf::from("server").join(&file_path);
        }

        // Skip non-Rust files (config files, etc.)
        if !file_path.extension().is_some_and(|ext| ext == "rs") {
            return;
        }

        let metrics = file_metrics.entry(file_path.clone()).or_default();

        // Count by severity
        match diagnostic.level.as_str() {
            "error" => metrics.severity_counts.error += 1,
            "warning" => metrics.severity_counts.warning += 1,
            "help" | "suggestion" => metrics.severity_counts.suggestion += 1,
            _ => metrics.severity_counts.note += 1,
        }

        // Count by lint code
        let lint_name = diagnostic
            .code
            .as_ref()
            .map_or_else(|| "unknown".to_string(), |c| c.code.clone());

        *metrics.violations.entry(lint_name.clone()).or_default() += 1;

        // Collect detailed violation information
        let violation = ViolationDetail {
            file: file_path,
            line: span.line_start,
            column: span.column_start,
            end_line: span.line_end,
            end_column: span.column_end,
            lint_name,
            message: diagnostic.message.clone(),
            severity: diagnostic.level.clone(),
            suggestion: span.suggested_replacement.clone(),
            machine_applicable: span
                .suggestion_applicability
                .as_ref()
                .is_some_and(|a| a == "MachineApplicable"),
        };

        metrics.detailed_violations.push(violation);
    }
}

/// Parse clippy JSON output into file metrics (cognitive complexity <=8)
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn parse_clippy_json_output(
    output: &std::process::Output,
) -> Result<HashMap<PathBuf, FileMetrics>> {
    let reader = BufReader::new(output.stdout.as_slice());
    let mut file_metrics: HashMap<PathBuf, FileMetrics> = HashMap::new();
    let mut message_count = 0;

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Ok(msg) = serde_json::from_str::<ClippyMessage>(&line) {
            if let Some(diagnostic) = msg.message {
                if msg.reason == Some("compiler-message".to_string()) {
                    message_count += 1;
                    process_diagnostic(&diagnostic, &mut file_metrics);
                }
            }
        }
    }

    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("📊 Processed {message_count} compiler messages");
        eprintln!("📁 Files with metrics: {}", file_metrics.len());
    }

    Ok(file_metrics)
}
