// CB-200: TDG Grade Gate (#214)
//
// Reads the SQLite index (.pmat/context.db) and fails if functions fall below
// a configurable minimum TDG grade (default A).

use crate::models::comply_config::ComplyConfig;
use crate::tdg::grade::GRADE_VARIANTS;
use crate::tdg::Grade;
use std::path::Path;

use super::types::*;

/// Convert a TDG grade letter to a numeric ordinal for comparison.
/// The spellings a floor ADMITS, computed from the proved order.
///
/// Returns `None` when the threshold is not a grade this codebase produces, so
/// an unreadable threshold fails the rule instead of yielding an empty set that
/// reads as "nothing violates".
///
/// This replaces a private `grade_ordinal` over `["A","B","C","D","F"]` with a
/// `_ => 5` catch-all. Both halves were wrong. The five-letter list could never
/// match a MODIFIED grade against `WHERE tdg_grade IN (...)`, so at a floor of
/// "A" the gate saw 247 violations and could not see 1,719. And the catch-all
/// ranked every modified grade WORSE THAN F, so `grades_below("A-")` was empty
/// and the caller returned Pass — `.pmat-metrics.toml:44` declares exactly that
/// spelling.
///
/// The order is `Grade`'s, anchored to the score bands in
/// `contracts/lean/Theorems/Tdg/Grade.lean::Grade_Rank_Anchored_To_Score`.
fn passing_spellings(min_grade: &str) -> Option<Vec<&'static str>> {
    let floor = Grade::from_variant_name(min_grade.trim())?;
    Some(
        GRADE_VARIANTS
            .iter()
            .copied()
            .filter(|g| Grade::from_variant_name(g).is_some_and(|x| x.meets_threshold(floor)))
            .collect(),
    )
}

struct TdgViolation {
    file_path: String,
    function_name: String,
    tdg_grade: String,
    complexity: u32,
    start_line: usize,
}

/// How many below-floor definitions this project has already agreed to carry,
/// read from `.pmat-gates.toml` `[tdg] baseline`.
///
/// CB-200 is a RATCHET, not a threshold, and the distinction is the whole
/// design. `.pmat-ratchet.toml`'s own header states it: "A number in a config
/// file that nobody re-runs is not a gate; it is a wish with a colon after it."
/// `.pmat-metrics.toml:45` is this repository's worked example —
/// `max_unwrap_calls = 100`, annotated `Current: 570`, in a tree measuring
/// 20,390: three numbers, no two agreeing, nothing reading the key, green
/// throughout.
///
/// This number is different in exactly one way, and it is the only way that
/// matters: it is RE-DERIVED on every run by the same query that produced it,
/// and compared. It can never quietly become a transcription. It is a record of
/// debt, not a permission to add more — a definition below the floor that
/// pushes the count past it fails the gate, closed, and the absolute count is
/// printed on every outcome so "passing" can never be read as "clean".
///
/// It does NOT touch `min_tdg_grade` and adds no exclude glob. Both of those
/// are threshold-lowering in disguise: they make debt invisible, where this
/// keeps every unit of it counted and reported.
#[derive(Debug, PartialEq, Eq)]
enum TdgBaseline {
    /// No `baseline` key. Zero tolerance — any violation fails, exactly as
    /// before. This is the default precisely so that ratcheting THIS repo
    /// cannot silently relax any other repo that runs pmat.
    Absent,
    /// The count this project last agreed to hold flat.
    Held(usize),
    /// The key is present and is not a count. Fails, for the same reason an
    /// unparseable `min_grade` fails: a bound nobody can read must not be read
    /// as a bound nothing exceeds.
    Unreadable(String),
}

struct TdgGateOverrides {
    min_grade: Option<String>,
    exclude: Vec<String>,
    baseline: TdgBaseline,
}

impl Default for TdgGateOverrides {
    /// No overrides: the floor comes from `.pmat.yaml`, nothing extra is
    /// excluded, and the gate has zero tolerance. An unreadable or unparsable
    /// `.pmat-gates.toml` lands here, which fails CLOSED — the excludes vanish
    /// and the baseline vanishes with them, so a typo cannot buy headroom.
    fn default() -> Self {
        Self {
            min_grade: None,
            exclude: Vec::new(),
            baseline: TdgBaseline::Absent,
        }
    }
}

/// A `baseline` value is a count or it is nothing. A string, a float, a
/// negative — anything that is not a non-negative integer — is reported rather
/// than rounded down to "no baseline", because "the key you wrote does nothing"
/// and "you configured zero tolerance" are opposite claims and look identical
/// from the outside.
fn parse_tdg_baseline(value: Option<&toml::Value>) -> TdgBaseline {
    let Some(value) = value else {
        return TdgBaseline::Absent;
    };
    match value.as_integer() {
        Some(n) if n >= 0 => TdgBaseline::Held(n as usize),
        _ => TdgBaseline::Unreadable(value.to_string()),
    }
}

fn load_tdg_gate_overrides(project_path: &Path) -> TdgGateOverrides {
    let path = project_path.join(".pmat-gates.toml");
    let Ok(content) = std::fs::read_to_string(&path) else {
        return TdgGateOverrides::default();
    };
    let Ok(table) = content.parse::<toml::Table>() else {
        return TdgGateOverrides::default();
    };
    let tdg = table.get("tdg");
    let min_grade = tdg
        .and_then(|t| t.get("min_grade"))
        .and_then(|v| v.as_str())
        .map(String::from);
    let exclude = tdg
        .and_then(|t| t.get("exclude"))
        .and_then(|v| v.as_array())
        .map(|a| {
            a.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .unwrap_or_default();
    let baseline = parse_tdg_baseline(tdg.and_then(|t| t.get("baseline")));
    TdgGateOverrides {
        min_grade,
        exclude,
        baseline,
    }
}

fn is_index_stale(project_path: &Path, db_path: &Path) -> bool {
    let db_mtime = match std::fs::metadata(db_path).and_then(|m| m.modified()) {
        Ok(t) => t,
        Err(_) => return true,
    };
    for dir_name in ["src", "lib"] {
        let dir = project_path.join(dir_name);
        if !dir.exists() {
            continue;
        }
        if has_newer_source_file(&dir, db_mtime) {
            return true;
        }
    }
    false
}

fn has_newer_source_file(dir: &Path, threshold: std::time::SystemTime) -> bool {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return false;
    };
    entries.flatten().any(|entry| {
        let path = entry.path();
        if path.is_dir() {
            has_newer_source_file(&path, threshold)
        } else {
            is_source_file(&path) && entry_is_newer(&entry, threshold)
        }
    })
}

/// Is a directory entry's mtime newer than `threshold`? (false on any I/O error)
fn entry_is_newer(entry: &std::fs::DirEntry, threshold: std::time::SystemTime) -> bool {
    entry
        .metadata()
        .and_then(|m| m.modified())
        .map(|mtime| mtime > threshold)
        .unwrap_or(false)
}

fn is_source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .is_some_and(|ext| {
            matches!(
                ext,
                "rs" | "ts"
                    | "tsx"
                    | "js"
                    | "jsx"
                    | "py"
                    | "go"
                    | "java"
                    | "kt"
                    | "swift"
                    | "c"
                    | "cpp"
                    | "cs"
            )
        })
}

// `rebuild_index` used to live here: CB-200 called
// `AgentContextIndex::build[_incremental]` and SAVED the result whenever
// `.pmat/context.db` was missing or stale, so merely ASKING for a compliance
// verdict wrote 160KB+ of index into the project under audit and, on a large
// repo, spent minutes doing it. Building the index is `pmat query`'s job; the
// gate now reads what exists and refuses honestly when nothing does (#939).

/// Check TDG grade gate against the SQLite index.
/// Query violations from the context database for grades below threshold
fn query_tdg_violations(
    db_path: &Path,
    passing_grades: &[&str],
) -> Result<Vec<TdgViolation>, ComplianceCheck> {
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Failed to open context.db: {e}"),
        severity: Severity::Info,
    })?;
    let placeholders: Vec<String> = passing_grades
        .iter()
        .enumerate()
        .map(|(i, _)| format!("?{}", i + 1))
        .collect();
    // NOT IN the PASSING set, never IN a failing list. `IN` answers "no row"
    // for a value it does not list and never an error, so an eleven-letter
    // writer and a five-letter reader coexisted silently for a release.
    // Enumerating what passes makes any future alphabet drift return a row that
    // must be classified, instead of a smaller number nobody can see.
    let sql = format!("SELECT file_path, function_name, tdg_grade, complexity, start_line FROM functions WHERE tdg_grade NOT IN ({})", placeholders.join(", "));
    let mut stmt = conn.prepare(&sql).map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Failed to query context.db: {e}"),
        severity: Severity::Info,
    })?;
    let params: Vec<&dyn rusqlite::types::ToSql> = passing_grades
        .iter()
        .map(|g| g as &dyn rusqlite::types::ToSql)
        .collect();
    stmt.query_map(params.as_slice(), |row| {
        Ok(TdgViolation {
            file_path: row.get(0)?,
            function_name: row.get(1)?,
            tdg_grade: row.get(2)?,
            complexity: row.get::<_, i64>(3)? as u32,
            start_line: row.get::<_, i64>(4)? as usize,
        })
    })
    .map(|iter| iter.filter_map(|r| r.ok()).collect())
    .map_err(|e| ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Skip,
        message: format!("Query failed: {e}"),
        severity: Severity::Info,
    })
}

/// Check if a violation should be excluded (test files or glob patterns)
fn is_tdg_violation_excluded(v: &TdgViolation, exclude_patterns: &[glob::Pattern]) -> bool {
    if v.file_path.contains("/tests/")
        || v.file_path.contains("/test/")
        || v.file_path.ends_with("_test.rs")
        || v.file_path.ends_with("_tests.rs")
    {
        return true;
    }
    let opts = glob::MatchOptions {
        case_sensitive: true,
        require_literal_separator: false,
        require_literal_leading_dot: false,
    };
    exclude_patterns
        .iter()
        .any(|pat| pat.matches_with(&v.file_path, opts))
}

#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_tdg_grade_gate(
    project_path: &Path,
    comply_config: &ComplyConfig,
) -> ComplianceCheck {
    let db_path = project_path.join(".pmat").join("context.db");
    if !db_path.exists() {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".into(),
            status: CheckStatus::Skip,
            message: "Not measured: no .pmat/context.db. `comply check` will not build one - \
                      building writes .pmat/context.db and .pmat/context.idx into the project \
                      being audited. Run `pmat query \"x\"` to create the index, then re-run."
                .into(),
            severity: Severity::Info,
        };
    }
    // A stale index still measures something REAL; it is a rebuild that would
    // not. `comply check` used to rebuild and SAVE the index here, writing
    // .pmat/context.db + .pmat/context.idx (160KB+) into the audited tree on
    // every run — the same class of defect as the Cargo.lock it created while
    // checking whether a Cargo.lock existed (#939). The staleness is reported
    // instead of silently repaired, so the reader knows what the verdict rests
    // on.
    let staleness = if is_index_stale(project_path, &db_path) {
        " (index is stale: source files are newer than .pmat/context.db - \
         run `pmat query \"x\"` to refresh)"
    } else {
        ""
    };
    let overrides = load_tdg_gate_overrides(project_path);
    let min_grade = overrides
        .min_grade
        .as_deref()
        .unwrap_or(&comply_config.thresholds.min_tdg_grade);
    let Some(passing_grades) = passing_spellings(min_grade) else {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".into(),
            status: CheckStatus::Fail,
            message: format!(
                "minimum grade {min_grade:?} is not a grade this codebase produces (known: {}). \
                 A threshold that cannot be parsed must not be read as a threshold nothing violates.",
                GRADE_VARIANTS.join(", ")
            ),
            severity: Severity::Error,
        };
    };
    if passing_grades.len() == GRADE_VARIANTS.len() {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".into(),
            status: CheckStatus::Pass,
            message: format!(
                "Minimum grade {min_grade} \u{2014} no grades below threshold{staleness}"
            ),
            severity: Severity::Info,
        };
    }
    let violations = match query_tdg_violations(&db_path, &passing_grades) {
        Ok(v) => v,
        Err(check) => return check,
    };
    let all_excludes: Vec<&str> = comply_config
        .thresholds
        .tdg_exclude_paths
        .iter()
        .map(|s| s.as_str())
        .chain(overrides.exclude.iter().map(|s| s.as_str()))
        .collect();
    let exclude_patterns: Vec<glob::Pattern> = all_excludes
        .iter()
        .filter_map(|p| glob::Pattern::new(p).ok())
        .collect();
    let filtered: Vec<&TdgViolation> = violations
        .iter()
        .filter(|v| !is_tdg_violation_excluded(v, &exclude_patterns))
        .collect();
    tdg_grade_verdict(
        &filtered,
        violations.len(),
        min_grade,
        &overrides.baseline,
        staleness,
    )
}

/// The most anyone reads before scrolling. Both listings stop here.
const TDG_MAX_LISTED: usize = 10;

/// Worst first, and deterministically so.
///
/// The listing used to be `filtered.iter().take(10)` over an unordered SQLite
/// scan with no `ORDER BY`. Two runs of the same gate over the same database
/// printing a different ten is indistinguishable, to the reader, from the tree
/// having changed — and on a flat distribution of 1,905 across 1,052 files
/// there is nothing else in the message to tell them apart.
///
/// `Grade`'s derived `Ord` runs best-to-worst (`APlus` first), so the worst
/// grade is the LARGEST; a spelling the scale does not know sorts above even
/// `F`, because a violation nobody can rank is the one most worth looking at.
fn rank_tdg_violations<'a>(filtered: &[&'a TdgViolation]) -> Vec<&'a TdgViolation> {
    fn severity_rank(v: &TdgViolation) -> usize {
        Grade::from_variant_name(&v.tdg_grade).map_or(usize::MAX, |g| g as usize)
    }
    let mut ranked: Vec<&TdgViolation> = filtered.to_vec();
    ranked.sort_by(|a, b| {
        severity_rank(b)
            .cmp(&severity_rank(a))
            .then_with(|| b.complexity.cmp(&a.complexity))
            .then_with(|| a.file_path.cmp(&b.file_path))
            .then_with(|| a.start_line.cmp(&b.start_line))
    });
    ranked
}

/// Up to `limit` offenders, worst first, with a truncation line that names how
/// many were not shown. `limit` of 0 still lists one: a Fail that names nothing
/// is a number the reader cannot act on.
fn tdg_offender_listing(filtered: &[&TdgViolation], limit: usize) -> String {
    let limit = limit.clamp(1, TDG_MAX_LISTED);
    let ranked = rank_tdg_violations(filtered);
    let mut details: Vec<String> = ranked
        .iter()
        .take(limit)
        .map(|v| {
            format!(
                "    {}:{} {} [{}] (complexity: {})",
                v.file_path, v.start_line, v.function_name, v.tdg_grade, v.complexity
            )
        })
        .collect();
    if ranked.len() > limit {
        details.push(format!("    ... and {} more", ranked.len() - limit));
    }
    details.join("\n")
}

/// How many distinct files the surviving violations span — the shape of the
/// debt, not just its size. 1,905 in one file is a refactor; 1,905 across 1,052
/// files is a policy, and the reader cannot tell which from a count alone.
fn tdg_violation_file_count(filtered: &[&TdgViolation]) -> usize {
    filtered
        .iter()
        .map(|v| v.file_path.as_str())
        .collect::<std::collections::HashSet<_>>()
        .len()
}

fn tdg_check(status: CheckStatus, severity: Severity, message: String) -> ComplianceCheck {
    ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status,
        message,
        severity,
    }
}

/// CB-200's verdict over a completed measurement.
///
/// Four outcomes, and which one you get depends only on the baseline the
/// project recorded:
///
/// ```text
///   baseline unreadable      -> Fail   (a bound nobody can read is not a bound)
///   no baseline              -> Fail on any violation      (unchanged)
///   count <= baseline        -> Pass, carrying count AND baseline
///   count >  baseline        -> Fail, naming the excess and the offenders
/// ```
fn tdg_grade_verdict(
    filtered: &[&TdgViolation],
    queried_rows: usize,
    min_grade: &str,
    baseline: &TdgBaseline,
    staleness: &str,
) -> ComplianceCheck {
    match baseline {
        TdgBaseline::Unreadable(raw) => unreadable_baseline_verdict(raw),
        TdgBaseline::Absent => zero_tolerance_verdict(filtered, queried_rows, min_grade, staleness),
        TdgBaseline::Held(b) if filtered.len() > *b => {
            over_baseline_verdict(filtered, *b, min_grade, staleness)
        }
        TdgBaseline::Held(b) => within_baseline_verdict(filtered, *b, min_grade, staleness),
    }
}

fn unreadable_baseline_verdict(raw: &str) -> ComplianceCheck {
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "`[tdg] baseline` in .pmat-gates.toml is {raw}, which is not a count. A ratchet \
             baseline that cannot be read must not be read as a baseline nothing exceeds \
             \u{2014} write a non-negative integer, or delete the key to restore zero tolerance."
        ),
    )
}

/// A project that records no baseline gets exactly what it got before: any
/// surviving violation fails. The message is unchanged, byte for byte, so
/// adding a ratchet here cannot move another repository's output.
fn zero_tolerance_verdict(
    filtered: &[&TdgViolation],
    queried_rows: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    if count == 0 {
        return tdg_check(
            CheckStatus::Pass,
            Severity::Info,
            format!(
                "All non-test functions meet minimum grade {min_grade}{}{staleness}",
                if queried_rows == 0 {
                    String::new()
                } else {
                    format!(" ({queried_rows} test/excluded functions skipped)")
                }
            ),
        );
    }
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "{count} function(s) below minimum grade {min_grade}{staleness}\n{}",
            tdg_offender_listing(filtered, TDG_MAX_LISTED)
        ),
    )
}

/// At or under the recorded baseline: the gate passes, and says why it is not
/// clean while doing so.
///
/// `Warn`, not `Pass`, and the reason is mechanical rather than aesthetic.
///
/// `Pass` was the first choice — held-flat debt must not block a release, which
/// is the whole point of a ratchet — with `Severity::Warning` carrying the "not
/// clean" signal. That does not work: `retain_blocking_checks` (check.rs:270)
/// switches on `CheckStatus` ALONE and drops `Pass` unconditionally, ignoring
/// `Severity` entirely. `quality-gate.yml` runs
/// `pmat comply check --failures-only`, so the one line saying "1,904
/// definitions are below the floor" was discarded by the exact invocation CI
/// uses. A gate that hides its own debt from the only place anyone reads it is
/// how 1,904 accumulated unseen in the first place.
///
/// `Warn` does not block either — `exit_policy` (check.rs:241) only turns
/// warnings into a non-zero code under `--strict` — but it IS counted in
/// `report.summary.warn`, and the summary is deliberately tallied before the
/// list is narrowed, so the count survives `--failures-only`. The debt is
/// therefore always reachable, which was the requirement.
///
/// A clean tree at baseline 0 still reports `Pass`/`Info`: nothing is being
/// held, so there is nothing to warn about.
fn within_baseline_verdict(
    filtered: &[&TdgViolation],
    baseline: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    let slack = baseline - count;
    let slack_note = if slack == 0 {
        String::new()
    } else {
        format!(
            " The tree is {slack} under the recorded baseline: lower `[tdg] baseline` to \
             {count} in .pmat-gates.toml to bank it \u{2014} a baseline the tree has already \
             beaten is headroom for new debt."
        )
    };
    if count == 0 {
        return tdg_check(
            CheckStatus::Pass,
            if slack == 0 {
                Severity::Info
            } else {
                Severity::Warning
            },
            format!(
                "0 definitions below minimum grade {min_grade}, against a recorded baseline of \
                 {baseline}.{slack_note}{staleness}"
            ),
        );
    }
    tdg_check(
        CheckStatus::Warn,
        Severity::Warning,
        format!(
            "{count} definition(s) below minimum grade {min_grade} across {} file(s), at the \
             recorded baseline of {baseline} \u{2014} this is debt held flat, not a clean tree. \
             Any new definition below {min_grade} fails this gate.{slack_note}{staleness}",
            tdg_violation_file_count(filtered)
        ),
    )
}

/// Over the recorded baseline: closed, naming how many and by how much.
///
/// The caveat about the listing is not hedging. A baseline is a COUNT, so it
/// cannot identify WHICH definitions are new — only that there are more than
/// there were. Presenting the worst-graded survivors as "the ones you just
/// added" would be a claim the measurement does not support, and the reader
/// would chase the wrong functions.
fn over_baseline_verdict(
    filtered: &[&TdgViolation],
    baseline: usize,
    min_grade: &str,
    staleness: &str,
) -> ComplianceCheck {
    let count = filtered.len();
    let over = count - baseline;
    tdg_check(
        CheckStatus::Fail,
        Severity::Error,
        format!(
            "{count} definition(s) below minimum grade {min_grade} \u{2014} {over} OVER the \
             recorded baseline of {baseline}. A ratchet holds only if new debt is refused: fix \
             {over}, or revert what added them. Raising `[tdg] baseline` is not the fix \u{2014} \
             a baseline may only go down.{staleness}\n    (the baseline is a count, not a \
             roster, so these are the worst-graded survivors, not necessarily the ones just \
             added)\n{}",
            tdg_offender_listing(filtered, over)
        ),
    )
}

/// Evaluate a single custom score definition and return the compliance check
fn evaluate_custom_score(
    project_path: &Path,
    score_def: &crate::models::comply_config::CustomScoreDefinition,
) -> ComplianceCheck {
    let check_name = format!("CB-1100: Custom Score [{}]", score_def.id);
    let output = match std::process::Command::new("sh")
        .args(["-c", &score_def.command])
        .current_dir(project_path)
        .output()
    {
        Ok(o) => o,
        Err(e) => {
            return ComplianceCheck {
                name: check_name,
                status: CheckStatus::Skip,
                message: format!("Failed to run command: {e}"),
                severity: Severity::Info,
            }
        }
    };
    if !output.status.success() {
        return ComplianceCheck {
            name: check_name,
            status: CheckStatus::Fail,
            message: format!(
                "{}: command failed (exit {})",
                score_def.name,
                output.status.code().unwrap_or(-1)
            ),
            severity: Severity::from(score_def.severity),
        };
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let actual_score = match extract_score_from_output(&stdout) {
        Some(s) => s,
        None => {
            return ComplianceCheck {
                name: check_name,
                status: CheckStatus::Skip,
                message: format!(
                    "{}: could not parse score from command output",
                    score_def.name
                ),
                severity: Severity::Info,
            }
        }
    };
    match score_def.min_score {
        Some(min) if actual_score < min => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Fail,
            message: format!(
                "{}: score {:.1} below minimum {:.1}",
                score_def.name, actual_score, min
            ),
            severity: Severity::from(score_def.severity),
        },
        Some(min) => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Pass,
            message: format!(
                "{}: score {:.1} (min: {:.1})",
                score_def.name, actual_score, min
            ),
            severity: Severity::Info,
        },
        None => ComplianceCheck {
            name: check_name,
            status: CheckStatus::Pass,
            message: format!("{}: score {:.1}", score_def.name, actual_score),
            severity: Severity::Info,
        },
    }
}

/// CB-1100: Custom Project Scores
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub(crate) fn check_custom_scores(project_path: &Path) -> Vec<ComplianceCheck> {
    let config = match crate::models::comply_config::PmatYamlConfig::load(project_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };
    if config.scoring.custom_scores.is_empty() {
        return vec![];
    }
    config
        .scoring
        .custom_scores
        .iter()
        .map(|s| evaluate_custom_score(project_path, s))
        .collect()
}

fn extract_score_from_output(output: &str) -> Option<f64> {
    for line in output.lines() {
        let line = line.trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(score) = json.get("score").and_then(|s| s.as_f64()) {
                return Some(score);
            }
        }
    }
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("SCORE:") {
            if let Ok(score) = rest.trim().parse::<f64>() {
                return Some(score);
            }
        }
    }
    None
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[cfg(test)]
mod tests_tdg_grade {
    use super::*;
    use crate::models::comply_config::ComplyConfig;
    use std::path::PathBuf;

    /// The floor admits exactly the grades at least as good as it.
    ///
    /// Replaces `test_grade_ordinal` and `test_grades_below`, which pinned the
    /// defect rather than the rule: they asserted `grade_ordinal("X") == 5`
    /// (the catch-all that ranked every unknown, and every MODIFIED, grade
    /// worse than F) and `grades_below("A") == ["B","C","D","F"]` (the
    /// five-letter blindness). Both were true of the old code and both were
    /// the bug.
    #[test]
    fn passing_set_is_the_up_set_of_the_floor() {
        assert_eq!(
            passing_spellings("A+").expect("A+ is a canonical grade spelling"),
            vec!["A+"]
        );
        assert_eq!(
            passing_spellings("A").expect("A is a canonical grade spelling"),
            vec!["A+", "A"]
        );
        // The spelling that used to produce an EMPTY failing set and a Pass.
        assert_eq!(
            passing_spellings("A-").expect("A- is a canonical grade spelling"),
            vec!["A+", "A", "A-"]
        );
        assert_eq!(
            passing_spellings("B").expect("B is a canonical grade spelling"),
            vec!["A+", "A", "A-", "B+", "B"]
        );
        // Only F admits everything, and that is the one honest vacuous floor.
        assert_eq!(
            passing_spellings("F")
                .expect("F is a canonical grade spelling")
                .len(),
            GRADE_VARIANTS.len()
        );
    }

    /// Passing and failing partition the scale at every floor — the property
    /// the five-letter table did not have, proved for all eleven floors in
    /// `contracts/lean/Theorems/Tdg/Grade.lean::Grade_Partition`.
    #[test]
    fn passing_and_failing_partition_the_scale() {
        for floor in GRADE_VARIANTS {
            let passing = passing_spellings(floor).expect("canonical spelling parses");
            let failing: Vec<_> = GRADE_VARIANTS
                .iter()
                .filter(|g| !passing.contains(g))
                .collect();
            assert_eq!(
                passing.len() + failing.len(),
                GRADE_VARIANTS.len(),
                "floor {floor} does not partition the scale"
            );
        }
    }

    /// An unreadable threshold yields `None`, never an empty set. An empty set
    /// is what the caller reads as "nothing violates".
    #[test]
    fn an_unparseable_floor_is_none_not_empty() {
        for bad in ["X", "", " ", "A--", "Q", "E"] {
            assert!(
                passing_spellings(bad).is_none(),
                "{bad:?} must not parse as a grade floor"
            );
        }
        // Counter-test: every canonical spelling still parses, so the guard did
        // not become a spelling police.
        for good in GRADE_VARIANTS {
            assert!(passing_spellings(good).is_some(), "{good} must parse");
        }
    }

    #[test]
    fn test_missing_db_returns_skip() {
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-missing-db");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(&tmp, &config);
        assert_eq!(result.status, CheckStatus::Skip);
        assert!(
            result.message.contains("Not measured"),
            "{}",
            result.message
        );
    }

    /// #939: CB-200 used to BUILD and SAVE the agent-context index whenever
    /// `.pmat/context.db` was missing or stale — so asking for a compliance
    /// verdict wrote `.pmat/context.db` and `.pmat/context.idx` into the
    /// project being audited (and spent minutes doing it on a large repo).
    /// An audit reads; `pmat query` builds.
    #[test]
    fn cb200_never_builds_an_index_inside_the_audited_project() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        std::fs::write(tmp.path().join("src/lib.rs"), "pub fn f() {}\n").expect("write source");

        let result = check_tdg_grade_gate(tmp.path(), &ComplyConfig::default());

        assert_eq!(result.status, CheckStatus::Skip);
        assert!(
            !tmp.path().join(".pmat").exists(),
            "comply check must not write .pmat/ into the project it audits"
        );
        assert!(
            result.message.contains("Not measured"),
            "an unbuilt index is unmeasured, not compliant: {}",
            result.message
        );
    }

    /// A stale index is still a real measurement — it is reported, with the
    /// staleness named, rather than silently repaired by a write.
    #[test]
    fn cb200_reports_staleness_instead_of_rebuilding() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, \
             function_name TEXT NOT NULL, tdg_grade TEXT NOT NULL DEFAULT 'A', \
             complexity INTEGER NOT NULL DEFAULT 1, start_line INTEGER NOT NULL DEFAULT 0)",
        )
        .expect("schema");
        drop(conn);

        // A source file newer than the db makes the index stale.
        //
        // The mtime is SET, not raced for. Writing the source after the db and
        // trusting wall-clock ordering failed on CI with "fixture must be
        // stale": both files landed in the same filesystem timestamp tick, so
        // `is_index_stale` correctly saw nothing newer and the fixture never
        // reached the behaviour under test. Filesystem timestamp granularity is
        // coarser than the microseconds between two writes, and varies by
        // filesystem — which is exactly the kind of thing that passes on the
        // author's machine and fails in CI.
        std::fs::create_dir_all(tmp.path().join("src")).expect("create src");
        let src = tmp.path().join("src/lib.rs");
        std::fs::write(&src, "pub fn f() {}\n").expect("write source");
        let db_mtime = std::fs::metadata(&db_path)
            .and_then(|m| m.modified())
            .expect("db mtime");
        std::fs::File::options()
            .write(true)
            .open(&src)
            .and_then(|f| f.set_modified(db_mtime + std::time::Duration::from_secs(60)))
            .expect("set source mtime a clear minute past the db");
        assert!(
            is_index_stale(tmp.path(), &db_path),
            "fixture must be stale"
        );

        let before = std::fs::metadata(&db_path).expect("meta").len();
        let result = check_tdg_grade_gate(tmp.path(), &ComplyConfig::default());
        let after = std::fs::metadata(&db_path).expect("meta").len();

        assert_eq!(before, after, "the audit rewrote the index it was reading");
        assert!(
            !pmat_dir.join("context.idx").exists(),
            "comply check must not build .pmat/context.idx"
        );
        assert!(
            result.message.contains("index is stale"),
            "staleness must be reported, not silently repaired: {}",
            result.message
        );
    }

    #[test]
    fn test_grade_f_no_violations() {
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-grade-f");
        let mut config = ComplyConfig::default();
        config.thresholds.min_tdg_grade = "F".to_string();
        let result = check_tdg_grade_gate(&tmp, &config);
        assert!(result.status == CheckStatus::Skip || result.status == CheckStatus::Pass);
    }

    #[test]
    fn test_in_memory_db_with_violations() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/core.rs', 'good_fn', 'A', 5, 10)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/legacy.rs', 'bad_fn', 'D', 42, 20)", []).expect("insert D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/awful.rs', 'terrible_fn', 'F', 60, 30)", []).expect("insert F");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/tests/helpers.rs', 'test_helper', 'D', 35, 40)", []).expect("insert test D");
        drop(conn);
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result
            .message
            .contains("2 function(s) below minimum grade A"));
        assert!(result.message.contains("src/legacy.rs:20 bad_fn [D]"));
        assert!(result.message.contains("src/awful.rs:30 terrible_fn [F]"));
        assert!(!result.message.contains("test_helper"));
    }

    #[test]
    fn test_tdg_exclude_paths() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('vendor/lib.rs', 'vendor_fn', 'D', 40, 10)", []).expect("insert vendor D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/main.rs', 'main_fn', 'D', 30, 5)", []).expect("insert src D");
        drop(conn);
        let mut config = ComplyConfig::default();
        config.thresholds.tdg_exclude_paths = vec!["vendor/*".to_string()];
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1 function(s)"));
        assert!(result.message.contains("main_fn"));
        assert!(!result.message.contains("vendor_fn"));
    }

    #[test]
    fn test_all_pass_with_good_grades() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)", []).expect("insert B");
        drop(conn);
        let mut config = ComplyConfig::default();
        config.thresholds.min_tdg_grade = "B".to_string();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Pass);
        assert!(result.message.contains("meet minimum grade B"));
    }

    #[test]
    fn test_pmat_gates_toml_min_grade_override() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)", []).expect("insert A");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)", []).expect("insert B");
        drop(conn);
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nmin_grade = \"B\"\n",
        )
        .expect("write gates toml");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_pmat_gates_toml_exclude_override() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");
        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch("CREATE TABLE IF NOT EXISTS functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, function_name TEXT NOT NULL, signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function', doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '', start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0, language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '', tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A', complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1, big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0, loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0, churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0, pattern_diversity REAL NOT NULL DEFAULT 0.0, fault_annotations TEXT NOT NULL DEFAULT '[]')").expect("create schema");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/core_generated.rs', 'gen_fn', 'D', 40, 10)", []).expect("insert generated D");
        conn.execute("INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line) VALUES ('src/real.rs', 'real_fn', 'D', 30, 5)", []).expect("insert real D");
        drop(conn);
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nexclude = [\"**/*_generated.rs\"]\n",
        )
        .expect("write gates toml");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1 function(s)"));
        assert!(result.message.contains("real_fn"));
        assert!(!result.message.contains("gen_fn"));
    }

    #[test]
    fn test_is_index_stale_no_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        assert!(is_index_stale(
            tmp.path(),
            &tmp.path().join("nonexistent.db")
        ));
    }

    #[test]
    fn test_is_index_stale_fresh_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        std::fs::write(src_dir.join("lib.rs"), "fn main() {}").expect("write src");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let db_path = tmp.path().join("context.db");
        std::fs::write(&db_path, "").expect("write db");
        assert!(!is_index_stale(tmp.path(), &db_path));
    }

    #[test]
    fn test_is_index_stale_outdated_db() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let db_path = tmp.path().join("context.db");
        std::fs::write(&db_path, "").expect("write db");
        std::thread::sleep(std::time::Duration::from_millis(50));
        let src_dir = tmp.path().join("src");
        std::fs::create_dir_all(&src_dir).expect("create src");
        std::fs::write(src_dir.join("lib.rs"), "fn main() {}").expect("write src");
        assert!(is_index_stale(tmp.path(), &db_path));
    }

    #[test]
    fn test_is_source_file() {
        assert!(is_source_file(Path::new("foo.rs")));
        assert!(is_source_file(Path::new("foo.py")));
        assert!(is_source_file(Path::new("foo.ts")));
        assert!(!is_source_file(Path::new("foo.txt")));
        assert!(!is_source_file(Path::new("foo.toml")));
        assert!(!is_source_file(Path::new("Makefile")));
    }

    // ── CB-200 as a RATCHET ──────────────────────────────────────────────
    //
    // The gate was BLIND until 2026-08-20: a five-letter reader against an
    // eleven-letter writer, so it saw 247 violations and could not see 1,719.
    // Every historical "CB-200 passed" came from that version. Once it could
    // see, it measured 1,905 below-A definitions across 1,052 files, max 12 per
    // file — a flat distribution with no hotspot and no bounded refactor.
    //
    // The two ways to make that green are both threshold-lowering in disguise:
    // drop `min_tdg_grade`, or add an exclude glob. One of them (187f506885) is
    // how a past "pass" was manufactured. Neither is done here. The gate holds
    // the count flat instead, refuses any increase closed, and prints the
    // absolute number on every outcome so that passing can never be mistaken
    // for clean.

    /// A project with `.pmat/context.db` holding exactly these rows.
    fn tdg_fixture(rows: &[(&str, &str, &str, u32, usize)]) -> tempfile::TempDir {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let conn = rusqlite::Connection::open(pmat_dir.join("context.db")).expect("open db");
        conn.execute_batch(
            "CREATE TABLE functions (id INTEGER PRIMARY KEY, file_path TEXT NOT NULL, \
             function_name TEXT NOT NULL, tdg_grade TEXT NOT NULL DEFAULT 'A', \
             complexity INTEGER NOT NULL DEFAULT 1, start_line INTEGER NOT NULL DEFAULT 0)",
        )
        .expect("schema");
        for (file, name, grade, complexity, line) in rows {
            conn.execute(
                "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, \
                 start_line) VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![file, name, grade, complexity, line],
            )
            .expect("insert row");
        }
        tmp
    }

    fn write_gates(tmp: &tempfile::TempDir, body: &str) {
        std::fs::write(tmp.path().join(".pmat-gates.toml"), body).expect("write gates toml");
    }

    fn judge(tmp: &tempfile::TempDir) -> ComplianceCheck {
        check_tdg_grade_gate(tmp.path(), &ComplyConfig::default())
    }

    /// The key is OPTIONAL, and its absence is not a softer gate.
    ///
    /// This is the counter-test that bounds the whole change: pmat runs against
    /// other repositories, and none of them has agreed to anything. A project
    /// that records no baseline must get zero tolerance, byte for byte, whether
    /// it has a `.pmat-gates.toml` with no `baseline` key or no file at all.
    #[test]
    fn without_a_baseline_key_any_violation_still_fails() {
        let no_file = tdg_fixture(&[("src/a.rs", "bad", "D", 30, 5)]);
        let verdict = judge(&no_file);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 function(s) below minimum grade A"),
            "the no-baseline message must be unchanged: {}",
            verdict.message
        );

        let other_keys = tdg_fixture(&[("src/a.rs", "bad", "D", 30, 5)]);
        write_gates(&other_keys, "[tdg]\nexclude = [\"nothing/**\"]\n");
        let verdict = judge(&other_keys);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 function(s) below minimum grade A"),
            "a [tdg] table without a baseline is still zero tolerance: {}",
            verdict.message
        );
    }

    /// At the baseline the gate PASSES — and must not read as clean.
    ///
    /// "1905 below A" printed at `Info` next to a genuinely empty tree is how
    /// 1,905 accumulated unseen. The count, the baseline and the word "debt"
    /// are all load-bearing, and so is `Severity::Warning` on a `Pass`.
    /// Held debt must survive `--failures-only`, which is the ONLY invocation
    /// CI runs.
    ///
    /// This is the counter-test for the status choice, and it exists because the
    /// obvious simplification — "it passed, so return `Pass`" — is wrong in a way
    /// nothing else here would catch. `retain_blocking_checks` (check.rs) matches
    /// on `CheckStatus` ALONE and drops `Pass` unconditionally; `Severity` is not
    /// consulted. `quality-gate.yml` runs `pmat comply check --failures-only`, so
    /// a `Pass` verdict deletes the one line reporting that 1,904 definitions sit
    /// below the floor, from the only report anyone reads.
    ///
    /// `Warn` does not block — `exit_policy` only escalates warnings under
    /// `--strict` — but it is counted into `summary.warn`, and the summary is
    /// tallied BEFORE the list is narrowed. That is what keeps the number
    /// reachable.
    ///
    /// RED: change `within_baseline_verdict`'s non-empty branch back to
    /// `CheckStatus::Pass` and this fails with `left: Pass, right: Warn`.
    #[test]
    fn held_debt_is_warn_so_failures_only_cannot_hide_it() {
        let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(
            verdict.status,
            CheckStatus::Warn,
            "held debt reported as Pass is dropped by `--failures-only`; the \
             count must stay reachable. Message was: {}",
            verdict.message
        );

        // ...and the counter-test: a genuinely clean tree is still a Pass, so
        // "always Warn" cannot satisfy the assertion above.
        let clean = tdg_fixture(&[]);
        write_gates(&clean, "[tdg]\nbaseline = 5\n");
        let clean_verdict = judge(&clean);
        assert_eq!(
            clean_verdict.status,
            CheckStatus::Pass,
            "nothing is being held, so there is nothing to warn about: {}",
            clean_verdict.message
        );
    }

    #[test]
    fn at_the_baseline_it_passes_carrying_the_count_and_the_baseline() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 2\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert_eq!(
            verdict.severity,
            Severity::Warning,
            "held debt at Info reads as clean: {}",
            verdict.message
        );
        assert!(
            verdict
                .message
                .contains("2 definition(s) below minimum grade A"),
            "the absolute count must lead: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("recorded baseline of 2"),
            "the baseline must be named: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("not a clean tree"),
            "passing at baseline must not read as passing clean: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("2 file(s)"),
            "the shape of the debt is part of the report: {}",
            verdict.message
        );
    }

    /// One over is a failure, named as one over.
    #[test]
    fn one_definition_over_the_baseline_fails_and_says_by_how_much() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
            ("src/c.rs", "three", "F", 44, 2),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 2\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert_eq!(verdict.severity, Severity::Error);
        assert!(
            verdict
                .message
                .contains("3 definition(s) below minimum grade A"),
            "the absolute count must lead even on a failure: {}",
            verdict.message
        );
        assert!(
            verdict
                .message
                .contains("1 OVER the recorded baseline of 2"),
            "the excess must be named: {}",
            verdict.message
        );
        // Exactly `over` offenders are listed, worst first, and the rest are
        // counted rather than dropped.
        assert!(
            verdict.message.contains("src/c.rs:2 three [F]"),
            "the worst survivor must be named: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("... and 2 more"),
            "the unlisted remainder must be counted: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("may only go down"),
            "the fix must not read as 'raise the baseline': {}",
            verdict.message
        );
    }

    /// Under the baseline is a pass that asks to be banked.
    ///
    /// A baseline the tree has already beaten is headroom for new debt, which
    /// is the failure mode `.pmat-ratchet.toml`'s `--lower` job exists to
    /// prevent. CB-200 has no such job, so it asks in the message instead.
    #[test]
    fn under_the_baseline_passes_and_asks_for_the_baseline_to_be_lowered() {
        let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert!(
            verdict.message.contains("4 under the recorded baseline"),
            "slack must be reported: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("lower `[tdg] baseline` to 1"),
            "the message must name the number to bank: {}",
            verdict.message
        );
    }

    /// A clean tree under a stale baseline reports the slack, not silence.
    #[test]
    fn a_clean_tree_still_reports_a_baseline_it_has_outgrown() {
        let tmp = tdg_fixture(&[("src/a.rs", "fine", "A", 3, 1)]);
        write_gates(&tmp, "[tdg]\nbaseline = 5\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Pass, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("0 definitions below minimum grade A"),
            "{}",
            verdict.message
        );
        assert!(
            verdict.message.contains("lower `[tdg] baseline` to 0"),
            "a baseline of 5 over an empty tree is pure headroom: {}",
            verdict.message
        );
    }

    /// A baseline nobody can read is not a baseline nothing exceeds.
    ///
    /// The same rule `passing_spellings` already enforces for `min_grade`: the
    /// unreadable value is REPORTED, never quietly rounded to "no baseline",
    /// because "your key does nothing" and "you chose zero tolerance" are
    /// opposite claims that look identical from outside.
    #[test]
    fn an_unreadable_baseline_fails_closed_and_names_itself() {
        for bad in ["\"many\"", "-1", "1.5", "true", "[1905]"] {
            let tmp = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
            write_gates(&tmp, &format!("[tdg]\nbaseline = {bad}\n"));
            let verdict = judge(&tmp);
            assert_eq!(
                verdict.status,
                CheckStatus::Fail,
                "baseline = {bad} must fail: {}",
                verdict.message
            );
            assert!(
                verdict.message.contains("is not a count"),
                "baseline = {bad} must name itself: {}",
                verdict.message
            );
        }
        // Counter-test: the guard did not become a "no baseline may be small"
        // rule. Zero is a real baseline — it is zero tolerance, said out loud.
        let clean = tdg_fixture(&[("src/a.rs", "fine", "A", 3, 1)]);
        write_gates(&clean, "[tdg]\nbaseline = 0\n");
        assert_eq!(judge(&clean).status, CheckStatus::Pass);
        let dirty = tdg_fixture(&[("src/a.rs", "one", "D", 30, 5)]);
        write_gates(&dirty, "[tdg]\nbaseline = 0\n");
        let verdict = judge(&dirty);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);
        assert!(
            verdict
                .message
                .contains("1 OVER the recorded baseline of 0"),
            "{}",
            verdict.message
        );
    }

    /// The over-correction this must NOT become: a baseline that hides debt.
    ///
    /// A generous baseline buys silence in exactly one place — the pass/fail
    /// verdict. It must not raise the floor, must not exclude a path, and must
    /// not remove a single definition from the count. Nothing here may read as
    /// the sentence a genuinely clean tree gets.
    #[test]
    fn a_baseline_holds_debt_flat_without_hiding_any_of_it() {
        let tmp = tdg_fixture(&[
            ("src/a.rs", "one", "D", 30, 5),
            ("src/b.rs", "two", "C-", 20, 9),
            ("src/c.rs", "three", "B+", 12, 3),
        ]);
        write_gates(&tmp, "[tdg]\nbaseline = 100\n");
        let verdict = judge(&tmp);

        assert_eq!(verdict.status, CheckStatus::Warn, "{}", verdict.message);
        assert!(
            verdict.message.contains("3 definition(s)"),
            "every unit of debt stays counted: {}",
            verdict.message
        );
        assert!(
            verdict.message.contains("minimum grade A"),
            "the floor is still A — a baseline is not a lowered threshold: {}",
            verdict.message
        );
        // B+ is below A and is still counted: the baseline did not quietly
        // narrow the alphabet the way the five-letter reader did.
        assert!(
            !verdict
                .message
                .contains("All non-test functions meet minimum grade"),
            "held debt must never borrow the clean tree's sentence: {}",
            verdict.message
        );
    }

    /// The listing is deterministic, worst first.
    ///
    /// It used to be `take(10)` off a `SELECT` with no `ORDER BY`. On a flat
    /// 1,905-across-1,052-files distribution there is nothing else in the
    /// message to distinguish "the tree changed" from "SQLite scanned in a
    /// different order".
    #[test]
    fn the_offender_listing_is_worst_first_and_stable() {
        let tmp = tdg_fixture(&[
            ("src/mild.rs", "mild", "B+", 11, 1),
            ("src/worst.rs", "worst", "F", 9, 1),
            ("src/bad_simple.rs", "bad_simple", "D", 4, 1),
            ("src/bad_complex.rs", "bad_complex", "D", 90, 1),
        ]);
        let verdict = judge(&tmp);
        assert_eq!(verdict.status, CheckStatus::Fail, "{}", verdict.message);

        for name in ["worst", "bad_complex", "bad_simple", "mild"] {
            assert!(
                verdict.message.contains(name),
                "{name} must be listed: {}",
                verdict.message
            );
        }
        let order: Vec<usize> = ["worst", "bad_complex", "bad_simple", "mild"]
            .iter()
            .map(|name| verdict.message.find(name).unwrap_or(usize::MAX))
            .collect();
        let mut sorted = order.clone();
        sorted.sort_unstable();
        assert_eq!(
            order, sorted,
            "worst grade first, then highest complexity: {}",
            verdict.message
        );
    }
}
