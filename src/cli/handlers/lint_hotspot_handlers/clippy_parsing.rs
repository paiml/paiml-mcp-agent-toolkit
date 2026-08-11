// clippy_parsing.rs - Clippy output parsing and diagnostic processing
// Included by clippy.rs via include!()

fn parse_clippy_output(
    stdout: &[u8],
    abs_file_path: &Path,
    file_path: &Path,
    base_dirs: &[PathBuf],
) -> Result<(
    Vec<ViolationDetail>,
    Vec<ViolationDetail>,
    SeverityDistribution,
)> {
    let reader = BufReader::new(stdout);
    let mut file_violations = Vec::new();
    let mut all_violations = Vec::new();
    let mut severity_dist = SeverityDistribution::default();
    // `--all-targets` compiles the same source file once per target (lib, test,
    // bench, …) so cargo emits an identical diagnostic once per target. Counting
    // them all reported 40 findings for a fixture that has 20.
    let mut seen: std::collections::HashSet<ViolationKey> = std::collections::HashSet::new();

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        if let Some(violation) = parse_clippy_line(&line, abs_file_path, file_path, base_dirs)? {
            if !seen.insert(violation_key(&violation)) {
                continue;
            }
            file_violations.push(violation.clone());
            update_severity_distribution(&mut severity_dist, &violation.severity);
            all_violations.push(violation);
        }
    }

    Ok((file_violations, all_violations, severity_dist))
}

/// Identity of a single clippy finding, used to collapse the per-target copies
/// `--all-targets` produces.
type ViolationKey = (PathBuf, u32, u32, u32, u32, String, String);

fn violation_key(v: &ViolationDetail) -> ViolationKey {
    (
        v.file.clone(),
        v.line,
        v.column,
        v.end_line,
        v.end_column,
        v.lint_name.clone(),
        v.message.clone(),
    )
}

/// Decode one line of cargo's JSON stream.
///
/// Returns `Ok(None)` for blank lines. Any other line that cannot be decoded is
/// an ERROR, not a silent skip: `ClippyMessage` accepts every JSON object cargo
/// emits (both fields are optional), so a decode failure means the schema moved
/// under us. Swallowing those failures is exactly how #679 hid — a field-name
/// typo in the span struct made every text-carrying diagnostic undecodable and
/// the parser dropped them without a word, reporting the project as clean.
fn decode_clippy_line(line: &str) -> Result<Option<ClippyMessage>> {
    if line.trim().is_empty() {
        return Ok(None);
    }
    match serde_json::from_str::<ClippyMessage>(line) {
        Ok(msg) => Ok(Some(msg)),
        Err(e) => Err(anyhow::anyhow!(
            "could not decode a `cargo clippy --message-format=json` record, so the \
             lint count would silently under-report: {e}\nrecord: {}",
            line.chars().take(400).collect::<String>()
        )),
    }
}

fn parse_clippy_line(
    line: &str,
    abs_file_path: &Path,
    file_path: &Path,
    base_dirs: &[PathBuf],
) -> Result<Option<ViolationDetail>> {
    let Some(msg) = decode_clippy_line(line)? else {
        return Ok(None);
    };

    let (Some("compiler-message"), Some(diagnostic)) = (msg.reason.as_deref(), &msg.message) else {
        return Ok(None);
    };

    let Some(span) = find_primary_span(diagnostic) else {
        return Ok(None);
    };

    // #679: cargo emits span paths RELATIVE, so a user-supplied absolute
    // `--file /abs/src/lib.rs` matched none of the comparisons below and the
    // command reported 0 violations for a file that had 20. #698: they are
    // relative to the WORKSPACE ROOT, not to `-p`, so `span_base_dirs` supplies
    // both candidates and each is compared for path identity.
    if !span_matches_target(&span.file_name, base_dirs, abs_file_path, file_path) {
        return Ok(None);
    }

    Ok(Some(create_violation_detail(file_path, span, diagnostic)))
}

/// Does a span's `file_name`, resolved against any plausible base, name the
/// file the user asked for?
fn span_matches_target(
    span_file: &str,
    base_dirs: &[PathBuf],
    abs_file_path: &Path,
    file_path: &Path,
) -> bool {
    base_dirs.iter().any(|base| {
        let resolved = resolve_diagnostic_path(span_file, base);
        is_target_file(&resolved, abs_file_path, file_path)
    })
}

/// Resolve a diagnostic's `file_name` to an absolute path using the directory
/// cargo was run in, canonicalizing when the file exists on disk.
fn resolve_diagnostic_path(diagnostic_file: &str, base_dir: &Path) -> String {
    let raw = Path::new(diagnostic_file);
    let joined = if raw.is_absolute() {
        raw.to_path_buf()
    } else {
        base_dir.join(raw)
    };
    std::fs::canonicalize(&joined)
        .unwrap_or(joined)
        .to_string_lossy()
        .into_owned()
}

pub(super) fn find_primary_span(diagnostic: &DiagnosticMessage) -> Option<&DiagnosticSpan> {
    diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary || diagnostic.spans.len() == 1)
}

/// Does this diagnostic belong to the file the user named with `--file`?
///
/// #698: the third arm used to be `diagnostic_path.ends_with(file_path)`.
/// `Path::ends_with` matches on trailing path COMPONENTS, so `--file main.rs`
/// claimed every file named `main.rs` anywhere in the project (and
/// `--file src/lib.rs` claimed every workspace member's `src/lib.rs`).
///
/// Observed on a two-binary fixture whose root `main.rs` is clean and whose
/// `src/main.rs` carries 8 `clippy::len_zero`:
/// `--file main.rs` reported `total_violations: 22`, `sloc: 3`,
/// `defect_density: 7.33`, every entry labelled `"file": "main.rs"` while
/// carrying line/column numbers from `src/main.rs` — and the quality gate
/// failed with exit 1 on a file that has nothing wrong with it. The same run
/// with `--file src/main.rs` reported density 0.58 and exit 0.
///
/// `diagnostic_file` has already been resolved to an absolute (canonical where
/// the file exists) path by `resolve_diagnostic_path`, so file identity is the
/// only correct test. The `file_path` arm keeps the caller-supplied spelling
/// working when the span is already exactly that string.
pub(super) fn is_target_file(diagnostic_file: &str, abs_file_path: &Path, file_path: &Path) -> bool {
    let diagnostic_path = PathBuf::from(diagnostic_file);
    diagnostic_path == *abs_file_path || diagnostic_path == *file_path
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

pub(super) fn extract_lint_name(diagnostic: &DiagnosticMessage) -> String {
    diagnostic
        .code
        .as_ref()
        .map(|c| c.code.clone())
        .unwrap_or_default()
}

pub(super) fn is_machine_applicable(span: &DiagnosticSpan) -> bool {
    span.suggestion_applicability
        .as_ref()
        .is_some_and(|a| a == "machine-applicable" || a == "maybe-incorrect")
}

pub(super) fn update_severity_distribution(severity_dist: &mut SeverityDistribution, level: &str) {
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
        // Use the path exactly as cargo stated it.
        //
        // This used to rewrite `examples/foo.rs` to `server/examples/foo.rs`
        // for a `server/` crate layout that no longer exists. On pmat's own
        // tree that invented a path for 85 example files, none of which could
        // then be found on disk, so their SLOC stayed 0 — they counted toward
        // `total_project_violations` but were silently excluded from the
        // per-file list and the hotspot ranking.
        let file_path = PathBuf::from(&span.file_name);

        // Skip non-Rust files (config files, etc.)
        if file_path.extension().is_none_or(|ext| ext != "rs") {
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
    // See `parse_clippy_output`: `--all-targets` re-emits every diagnostic once
    // per target, so without this the same finding is counted 2-4 times.
    let mut seen: std::collections::HashSet<ViolationKey> = std::collections::HashSet::new();

    for line in std::io::BufRead::lines(reader) {
        let line = line?;
        // Strict decode: a record we cannot read must not be silently dropped
        // (that is how #679's 0-violation "clean" verdict was manufactured).
        let Some(msg) = decode_clippy_line(&line)? else {
            continue;
        };
        if msg.reason.as_deref() != Some("compiler-message") {
            continue;
        }
        let Some(diagnostic) = msg.message else {
            continue;
        };
        if let Some(key) = diagnostic_dedup_key(&diagnostic) {
            if !seen.insert(key) {
                continue;
            }
        }
        message_count += 1;
        process_diagnostic(&diagnostic, &mut file_metrics);
    }

    if std::env::var("LINT_HOTSPOT_DEBUG").is_ok() {
        eprintln!("📊 Processed {message_count} compiler messages");
        eprintln!("📁 Files with metrics: {}", file_metrics.len());
    }

    Ok(file_metrics)
}

/// Dedup key for a raw diagnostic, taken from its primary span.
fn diagnostic_dedup_key(diagnostic: &DiagnosticMessage) -> Option<ViolationKey> {
    let span = diagnostic
        .spans
        .iter()
        .find(|s| s.is_primary)
        .or_else(|| diagnostic.spans.first())?;
    Some((
        PathBuf::from(&span.file_name),
        span.line_start,
        span.column_start,
        span.line_end,
        span.column_end,
        diagnostic
            .code
            .as_ref()
            .map_or_else(String::new, |c| c.code.clone()),
        diagnostic.message.clone(),
    ))
}
