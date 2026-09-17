// Estimate ledger (PMAT-1366) — one gated, append-only writer for
// `docs/audits/impl-estimates.jsonl`.
//
// Measured problem: the ledger was written by hand. No function in the tree
// produced it, so nothing checked a row before it landed. 16 rows keyed
// `paiml-mcp-agent-toolkit` were written without a `unit` (and 2 more keyed
// `pmat`, the key 14 of this repository's rows had drifted to), and the reader
// that pools them refuses a unit-less row: of those 16, 0 were poolable, and
// the estimator exited 2 for the repository they describe. Nothing any single row
// said was false; the missing field was the defect, and the producer was never
// gated.
//
// So: `record` is the ONE writer. It refuses a row with no unit, a unit it
// cannot pool, a range phase, an estimate with no basis, or a repo key that
// would split the ledger — and a refused row writes nothing at all. It only
// ever appends (O_APPEND, one `write_all`), never rewrites. `check` applies the same rule to the committed file, so a
// row written around the writer is caught instead of silently skipped.

/// Repo-relative path of the estimate ledger.
const ESTIMATE_LEDGER_REL: &str = "docs/audits/impl-estimates.jsonl";

/// Units a row written now may carry: the ones a pool can be built from.
const ESTIMATE_MEASURED_UNITS: [&str; 2] = ["turn", "session"];

/// Retirement marker for legacy rows whose unit could not be recovered.
const ESTIMATE_UNIT_UNKNOWN: &str = "unknown";

/// One line of `docs/audits/impl-estimates.jsonl`, in ledger field order.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstimateRow {
    /// Repository key: the origin remote's basename
    pub repo: String,
    /// Ticket id, e.g. "PMAT-1366"
    pub ticket: String,
    /// `all` or a decimal phase number
    pub phase: String,
    /// How the work was executed
    pub mode: String,
    /// Estimate (serialised as `null` when absent)
    pub est: Option<u64>,
    /// Measured actual (serialised as `null` when absent)
    pub actual: Option<u64>,
    /// Unit of `est` and `actual`
    pub unit: Option<String>,
    /// Where the estimate came from
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<String>,
    /// Free-form note
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

fn is_blank(s: &str) -> bool {
    s.trim().is_empty()
}

fn is_decimal(s: &str) -> bool {
    !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit())
}

fn gate_required_fields(row: &EstimateRow) -> Vec<String> {
    [
        ("repo", &row.repo),
        ("ticket", &row.ticket),
        ("mode", &row.mode),
    ]
    .into_iter()
    .filter(|(_, value)| is_blank(value))
    .map(|(name, _)| format!("{name} is empty"))
    .collect()
}

fn gate_unit(unit: Option<&str>) -> Option<String> {
    let unit = unit.map(str::trim).filter(|u| !u.is_empty());
    match unit {
        None => Some(
            "unit is required: a measured row without its unit is [U] and enters no pool".into(),
        ),
        Some(ESTIMATE_UNIT_UNKNOWN) => Some(
            "unit unknown is refused: unknown marks a legacy row whose unit could not be \
             recovered; a row written now is measured now"
                .into(),
        ),
        Some(u) if ESTIMATE_MEASURED_UNITS.contains(&u) => None,
        Some(u) => Some(format!("unit {u:?} is not a measured unit (turn|session)")),
    }
}

fn gate_phase(phase: &str) -> Option<String> {
    if phase == "all" || is_decimal(phase) {
        return None;
    }
    Some(format!(
        "phase {phase:?} is neither `all` nor an integer: a range is never pooled — record \
         per-phase integer rows or `all`"
    ))
}

fn gate_basis(row: &EstimateRow) -> Option<String> {
    let has_basis = row.basis.as_deref().is_some_and(|b| !is_blank(b));
    (row.est.is_some() && !has_basis)
        .then(|| "an estimate without basis= is an invented number".to_string())
}

/// Every reason `row` may not be written. Empty means admissible.
pub fn gate_new_row(row: &EstimateRow) -> Vec<String> {
    let mut defects = gate_required_fields(row);
    defects.extend(gate_unit(row.unit.as_deref()));
    defects.extend(gate_phase(&row.phase));
    defects.extend(gate_basis(row));
    defects
}

/// The ledger's text, or empty when it does not exist yet.
fn read_ledger_text(path: &Path) -> Result<String> {
    match std::fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(e) => Err(e).context("Failed to read estimate ledger"),
    }
}

/// Distinct repo keys of the ledger's JSON rows, sorted.
fn ledger_repo_keys(text: &str) -> Vec<String> {
    let keys: std::collections::BTreeSet<String> = text
        .lines()
        .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
        .filter_map(|v| v.get("repo").and_then(|r| r.as_str()).map(str::to_string))
        .collect();
    keys.into_iter().collect()
}

/// A row keyed differently from the ledger it joins splits one repository's
/// pool in two: the reader pools per key, so the other key's rows vanish.
fn gate_repo_split(existing: &str, repo: &str) -> Option<String> {
    let keys = ledger_repo_keys(existing);
    keys.iter().any(|k| k != repo).then(|| {
        format!(
            "repo {repo:?} would split the ledger, which is keyed {}: one repository's ledger has \
             one key (rows keyed `pmat` hid 14 rows from `paiml-mcp-agent-toolkit` until PMAT-1366)",
            keys.join(",")
        )
    })
}

/// 1-based line number of the record whose newline ends at byte offset `end`.
fn line_ending_at(ledger: &Path, end: u64) -> Result<usize> {
    let bytes = std::fs::read(ledger).context("Failed to re-read estimate ledger")?;
    let end = usize::try_from(end).map_or(bytes.len(), |e| e.min(bytes.len()));
    Ok(bytes.iter().take(end).filter(|&&b| b == b'\n').count())
}

/// Append one gated row and return its 1-based line number. A refused row
/// writes nothing: no file, no directory.
///
/// The line number comes from the file offset `write` left behind, so a
/// concurrent appender cannot make it name somebody else's row.
pub fn append_row(ledger: &Path, row: &EstimateRow) -> Result<usize> {
    use std::io::{Seek, Write};
    let existing = read_ledger_text(ledger)?;
    let mut defects = gate_new_row(row);
    defects.extend(gate_repo_split(&existing, &row.repo));
    if !defects.is_empty() {
        anyhow::bail!(
            "estimate record refused ({} problem(s)): {}",
            defects.len(),
            defects.join("; ")
        );
    }
    let mut buffer = String::new();
    if !existing.is_empty() && !existing.ends_with('\n') {
        buffer.push('\n');
    }
    buffer.push_str(&serde_json::to_string(row).context("Failed to serialize estimate row")?);
    buffer.push('\n');
    if let Some(parent) = ledger.parent().filter(|p| !p.as_os_str().is_empty()) {
        std::fs::create_dir_all(parent).context("Failed to create estimate ledger directory")?;
    }
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(ledger)
        .context("Failed to open estimate ledger")?;
    file.write_all(buffer.as_bytes())
        .context("Failed to append estimate row")?;
    let end = file
        .stream_position()
        .context("Failed to read the estimate ledger offset")?;
    line_ending_at(ledger, end)
}

/// Ledger repo key from a git remote URL: the basename with `.git` stripped.
pub fn repo_key_from_remote_url(url: &str) -> Option<String> {
    let trimmed = url.trim().trim_end_matches('/');
    let tail = trimmed.rsplit(['/', ':']).next()?;
    let key = tail.strip_suffix(".git").unwrap_or(tail);
    (!key.is_empty()).then(|| key.to_string())
}

/// Repo key of `project`'s `origin` remote, if it has one.
pub fn origin_repo_key(project: &Path) -> Option<String> {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(project)
        .args(["remote", "get-url", "origin"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    repo_key_from_remote_url(&String::from_utf8_lossy(&output.stdout))
}

/// Decide the repo key: `--repo` when given, else the origin basename.
///
/// Never a Cargo package name or a directory name — `pmat` is this crate's
/// package name and was exactly the key that split this repository's ledger.
/// A key that disagrees with the ledger's existing rows is refused at append
/// time by `gate_repo_split`, which needs no remote at all.
pub fn resolve_repo_key(explicit: Option<&str>, origin: Option<&str>) -> Result<String> {
    match (explicit.filter(|e| !is_blank(e)), origin) {
        (Some(explicit), _) => Ok(explicit.to_string()),
        (None, Some(origin)) => Ok(origin.to_string()),
        (None, None) => anyhow::bail!("no origin remote: pass --repo"),
    }
}

/// Result of `pmat work estimate check`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct EstimateLedgerReport {
    /// JSON-object rows for the selected repo key (all keys when unfiltered)
    pub rows: usize,
    /// Schema-valid rows the reader pools
    pub poolable: usize,
    /// Schema-valid rows with `actual: null`
    pub unmeasured: usize,
    /// Schema-valid rows the reader skips: (1-based line, ticket, reason)
    pub excluded: Vec<(usize, String, String)>,
    /// Schema violations across EVERY row, regardless of the repo filter
    pub violations: Vec<String>,
    /// Distinct repo keys in the ledger; more than one is a violation
    pub keys: Vec<String>,
}

fn is_count(value: Option<&serde_json::Value>) -> bool {
    value.is_none_or(|v| v.is_null() || v.is_u64())
}

fn unit_violation(line: usize, ticket: &str, unit: Option<&serde_json::Value>) -> Option<String> {
    let unit = unit.filter(|u| !u.is_null());
    match unit.map(|u| u.as_str()) {
        None => Some(format!("L{line} ticket={ticket}: no unit")),
        Some(Some(u)) if ESTIMATE_MEASURED_UNITS.contains(&u) || u == ESTIMATE_UNIT_UNKNOWN => None,
        Some(_) => Some(format!(
            "L{line} ticket={ticket}: unit {} is not one of turn|session|unknown",
            unit.map(|u| u.to_string()).unwrap_or_default()
        )),
    }
}

fn row_violations(line: usize, obj: &serde_json::Map<String, serde_json::Value>) -> Vec<String> {
    let ticket = obj.get("ticket").and_then(|t| t.as_str()).unwrap_or("?");
    let mut violations: Vec<String> = unit_violation(line, ticket, obj.get("unit"))
        .into_iter()
        .collect();
    for field in ["est", "actual"] {
        if !is_count(obj.get(field)) {
            violations.push(format!(
                "L{line} ticket={ticket}: {field} is neither null nor a non-negative integer"
            ));
        }
    }
    for field in ["repo", "ticket"] {
        if !obj.get(field).is_some_and(|v| v.is_string()) {
            violations.push(format!("L{line} ticket={ticket}: {field} is not a string"));
        }
    }
    violations
}

fn phase_is_poolable(phase: Option<&serde_json::Value>) -> bool {
    match phase {
        Some(serde_json::Value::Number(n)) => n.is_u64(),
        Some(serde_json::Value::String(s)) => s == "all" || is_decimal(s),
        _ => false,
    }
}

/// Pool classification of a schema-valid row: `Ok(true)` poolable,
/// `Ok(false)` unmeasured, `Err(reason)` excluded. The reason strings are
/// estimate.sh's own, so `unknown` is `unit-not-turn` there and here.
fn classify_row(obj: &serde_json::Map<String, serde_json::Value>) -> Result<bool, &'static str> {
    if obj.get("actual").is_none_or(|a| a.is_null()) {
        return Ok(false);
    }
    match obj.get("unit").and_then(|u| u.as_str()) {
        Some("turn") if phase_is_poolable(obj.get("phase")) => Ok(true),
        Some("turn") => Err("range-phase"),
        _ => Err("unit-not-turn"),
    }
}

fn tally_row(
    report: &mut EstimateLedgerReport,
    line: usize,
    obj: &serde_json::Map<String, serde_json::Value>,
) {
    report.rows += 1;
    match classify_row(obj) {
        Ok(true) => report.poolable += 1,
        Ok(false) => report.unmeasured += 1,
        Err(reason) => {
            let ticket = obj.get("ticket").and_then(|t| t.as_str()).unwrap_or("?");
            report
                .excluded
                .push((line, ticket.to_string(), reason.to_string()));
        }
    }
}

fn check_line(report: &mut EstimateLedgerReport, line: usize, text: &str, repo: Option<&str>) {
    let value = serde_json::from_str::<serde_json::Value>(text).ok();
    let Some(obj) = value.as_ref().and_then(|v| v.as_object()) else {
        report
            .violations
            .push(format!("L{line} is not a JSON object"));
        return;
    };
    let violations = row_violations(line, obj);
    let selected = repo.is_none_or(|r| obj.get("repo").and_then(|v| v.as_str()) == Some(r));
    if !selected {
        report.violations.extend(violations);
        return;
    }
    if violations.is_empty() {
        tally_row(report, line, obj);
    } else {
        report.rows += 1;
        report.violations.extend(violations);
    }
}

/// Check the ledger text: schema violations over every row, pool
/// classification over the rows for `repo` (all rows when `None`).
pub fn check_ledger_text(text: &str, repo: Option<&str>) -> EstimateLedgerReport {
    let mut report = EstimateLedgerReport::default();
    for (idx, line) in text.lines().enumerate() {
        if !is_blank(line) {
            check_line(&mut report, idx + 1, line.trim(), repo);
        }
    }
    report.keys = ledger_repo_keys(text);
    if report.keys.len() > 1 {
        report.violations.push(format!(
            "ledger carries {} repo keys ({}): one repository's ledger has one key",
            report.keys.len(),
            report.keys.join(",")
        ));
    }
    report
}

// ---- CLI handlers -----------------------------------------------------------

fn estimate_project_path(path: Option<PathBuf>) -> Result<PathBuf> {
    let project_path = path.unwrap_or_else(|| PathBuf::from("."));
    if !project_path.exists() {
        anyhow::bail!(
            "work estimate: path does not exist: {}",
            project_path.display()
        );
    }
    Ok(project_path)
}

/// Where `pmat work estimate record` writes, and how it reports.
pub struct EstimateRecordTarget {
    /// `--repo`; the origin basename when absent
    pub repo: Option<String>,
    /// `--ledger`; `<path>/docs/audits/impl-estimates.jsonl` when absent
    pub ledger: Option<PathBuf>,
    /// Output format
    pub format: QaOutputFormat,
    /// Project path (default: current directory)
    pub path: Option<PathBuf>,
}

/// `pmat work estimate record` — append one gated row to the estimate ledger.
///
/// Refuses a row the reader could not pool. The refusal is the feature: a
/// unit-less row is not a smaller measurement, it is no measurement at all.
/// `row.repo` is ignored on entry: the key comes from `target.repo` or origin.
pub async fn handle_work_estimate_record(
    mut row: EstimateRow,
    target: EstimateRecordTarget,
) -> Result<()> {
    let project_path = estimate_project_path(target.path)?;
    let origin = origin_repo_key(&project_path);
    row.repo = resolve_repo_key(target.repo.as_deref(), origin.as_deref())?;
    let ledger = target
        .ledger
        .unwrap_or_else(|| project_path.join(ESTIMATE_LEDGER_REL));
    let line = append_row(&ledger, &row)?;
    render_estimate_record(&row, line, &ledger, target.format);
    Ok(())
}

/// `pmat work estimate check` — judge the committed ledger against the writer's rule.
///
/// A missing ledger is an error: an absent file has proven nothing.
pub async fn handle_work_estimate_check(
    repo: Option<String>,
    ledger: Option<PathBuf>,
    format: QaOutputFormat,
    path: Option<PathBuf>,
) -> Result<()> {
    let project_path = estimate_project_path(path)?;
    let ledger = ledger.unwrap_or_else(|| project_path.join(ESTIMATE_LEDGER_REL));
    if !ledger.is_file() {
        anyhow::bail!(
            "work estimate check: no estimate ledger at {}",
            ledger.display()
        );
    }
    let text = std::fs::read_to_string(&ledger).context("Failed to read estimate ledger")?;
    let report = check_ledger_text(&text, repo.as_deref());
    render_estimate_check(&report, repo.as_deref(), &ledger, format);
    if !report.violations.is_empty() {
        anyhow::bail!(
            "work estimate check: {} violation(s) in {}",
            report.violations.len(),
            ledger.display()
        );
    }
    Ok(())
}

fn render_estimate_record(row: &EstimateRow, line: usize, ledger: &Path, format: QaOutputFormat) {
    let unit = row.unit.as_deref().unwrap_or_default();
    let summary = format!(
        "recorded L{line} {} ticket={} unit={unit}",
        ledger.display(),
        row.ticket
    );
    if matches!(format, QaOutputFormat::Json) {
        print_json(&serde_json::json!({
            "recorded": true,
            "line": line,
            "ledger": ledger.display().to_string(),
            "row": row,
            "summary": summary,
        }));
        return;
    }
    println!("{summary}");
}

fn render_estimate_check(
    report: &EstimateLedgerReport,
    repo: Option<&str>,
    ledger: &Path,
    format: QaOutputFormat,
) {
    if matches!(format, QaOutputFormat::Json) {
        print_json(&serde_json::json!({
            "ledger": ledger.display().to_string(),
            "repo": repo,
            "report": report,
        }));
        return;
    }
    println!(
        "{}: {} rows, {} poolable, {} unmeasured (repo={})",
        ledger.display(),
        report.rows,
        report.poolable,
        report.unmeasured,
        repo.unwrap_or("all")
    );
    for (line, ticket, reason) in &report.excluded {
        println!("excluded L{line} ticket={ticket} reason={reason}");
    }
    for violation in &report.violations {
        println!("{violation}");
    }
}
