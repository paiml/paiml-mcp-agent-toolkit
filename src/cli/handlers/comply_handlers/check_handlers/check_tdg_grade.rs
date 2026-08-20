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

struct TdgGateOverrides {
    min_grade: Option<String>,
    exclude: Vec<String>,
}

fn load_tdg_gate_overrides(project_path: &Path) -> TdgGateOverrides {
    let path = project_path.join(".pmat-gates.toml");
    let content = match std::fs::read_to_string(&path) {
        Ok(c) => c,
        Err(_) => {
            return TdgGateOverrides {
                min_grade: None,
                exclude: Vec::new(),
            }
        }
    };
    let table: toml::Table = match content.parse() {
        Ok(t) => t,
        Err(_) => {
            return TdgGateOverrides {
                min_grade: None,
                exclude: Vec::new(),
            }
        }
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
    TdgGateOverrides { min_grade, exclude }
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
    let count = filtered.len();
    if count == 0 {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".into(),
            status: CheckStatus::Pass,
            message: format!(
                "All non-test functions meet minimum grade {min_grade}{}{staleness}",
                if violations.is_empty() {
                    String::new()
                } else {
                    format!(" ({} test/excluded functions skipped)", violations.len())
                }
            ),
            severity: Severity::Info,
        };
    }
    let mut details: Vec<String> = filtered
        .iter()
        .take(10)
        .map(|v| {
            format!(
                "    {}:{} {} [{}] (complexity: {})",
                v.file_path, v.start_line, v.function_name, v.tdg_grade, v.complexity
            )
        })
        .collect();
    if count > 10 {
        details.push(format!("    ... and {} more", count - 10));
    }
    ComplianceCheck {
        name: "CB-200: TDG Grade Gate".into(),
        status: CheckStatus::Fail,
        message: format!(
            "{count} function(s) below minimum grade {min_grade}{staleness}\n{}",
            details.join("\n")
        ),
        severity: Severity::Error,
    }
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
        assert_eq!(passing_spellings("A+").unwrap(), vec!["A+"]);
        assert_eq!(passing_spellings("A").unwrap(), vec!["A+", "A"]);
        // The spelling that used to produce an EMPTY failing set and a Pass.
        assert_eq!(passing_spellings("A-").unwrap(), vec!["A+", "A", "A-"]);
        assert_eq!(
            passing_spellings("B").unwrap(),
            vec!["A+", "A", "A-", "B+", "B"]
        );
        // Only F admits everything, and that is the one honest vacuous floor.
        assert_eq!(passing_spellings("F").unwrap().len(), GRADE_VARIANTS.len());
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
}
