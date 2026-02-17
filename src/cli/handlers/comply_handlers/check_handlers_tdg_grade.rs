// CB-200: TDG Grade Gate (#214)
//
// Reads the SQLite index (.pmat/context.db) and fails if functions fall below
// a configurable minimum TDG grade (default A). Uses direct SQLite query
// for performance (<10ms) instead of full index load (~150ms).
//
// Configuration in .pmat.yaml:
// ```yaml
// comply:
//   thresholds:
//     min_tdg_grade: "A"        # A, B, C, D, F
//     tdg_exclude_paths:
//       - "vendor/"
//       - "generated/"
// ```

/// Convert a TDG grade letter to a numeric ordinal for comparison.
/// A=0 (best), B=1, C=2, D=3, F=4 (worst).
fn grade_ordinal(grade: &str) -> u8 {
    match grade.trim() {
        "A" => 0,
        "B" => 1,
        "C" => 2,
        "D" => 3,
        "F" => 4,
        _ => 5, // Unknown grades treated as worst
    }
}

/// Return all grade letters that are strictly below (worse than) the given minimum.
fn grades_below(min_grade: &str) -> Vec<&'static str> {
    let threshold = grade_ordinal(min_grade);
    let all = ["A", "B", "C", "D", "F"];
    all.into_iter()
        .filter(|g| grade_ordinal(g) > threshold)
        .collect()
}

/// A single TDG grade violation from the index.
struct TdgViolation {
    file_path: String,
    function_name: String,
    tdg_grade: String,
    complexity: u32,
    start_line: usize,
}

/// Load TDG gate overrides from `.pmat-gates.toml` `[tdg]` section (#221).
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

/// Check TDG grade gate against the SQLite index.
pub(crate) fn check_tdg_grade_gate(
    project_path: &Path,
    comply_config: &ComplyConfig,
) -> ComplianceCheck {
    let db_path = project_path.join(".pmat").join("context.db");

    if !db_path.exists() {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".to_string(),
            status: CheckStatus::Skip,
            message: "No .pmat/context.db found — run `pmat index` first".to_string(),
            severity: Severity::Info,
        };
    }

    // Merge overrides from .pmat-gates.toml [tdg] section (#221)
    let overrides = load_tdg_gate_overrides(project_path);
    let min_grade = overrides
        .min_grade
        .as_deref()
        .unwrap_or(&comply_config.thresholds.min_tdg_grade);
    let failing_grades = grades_below(min_grade);

    if failing_grades.is_empty() {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".to_string(),
            status: CheckStatus::Pass,
            message: format!("Minimum grade {min_grade} — no grades below threshold"),
            severity: Severity::Info,
        };
    }

    // Open read-only to avoid locking issues with concurrent index builds
    let conn = match rusqlite::Connection::open_with_flags(
        &db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    ) {
        Ok(c) => c,
        Err(e) => {
            return ComplianceCheck {
                name: "CB-200: TDG Grade Gate".to_string(),
                status: CheckStatus::Skip,
                message: format!("Failed to open context.db: {e}"),
                severity: Severity::Info,
            };
        }
    };

    // Build WHERE clause for failing grades
    let placeholders: Vec<String> = failing_grades.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
    let sql = format!(
        "SELECT file_path, function_name, tdg_grade, complexity, start_line FROM functions WHERE tdg_grade IN ({})",
        placeholders.join(", ")
    );

    let violations = {
        let mut stmt = match conn.prepare(&sql) {
            Ok(s) => s,
            Err(e) => {
                return ComplianceCheck {
                    name: "CB-200: TDG Grade Gate".to_string(),
                    status: CheckStatus::Skip,
                    message: format!("Failed to query context.db: {e}"),
                    severity: Severity::Info,
                };
            }
        };

        let params: Vec<&dyn rusqlite::types::ToSql> = failing_grades
            .iter()
            .map(|g| g as &dyn rusqlite::types::ToSql)
            .collect();

        let rows = stmt.query_map(params.as_slice(), |row| {
            Ok(TdgViolation {
                file_path: row.get(0)?,
                function_name: row.get(1)?,
                tdg_grade: row.get(2)?,
                complexity: row.get::<_, i64>(3)? as u32,
                start_line: row.get::<_, i64>(4)? as usize,
            })
        });

        match rows {
            Ok(iter) => iter.filter_map(|r| r.ok()).collect::<Vec<_>>(),
            Err(e) => {
                return ComplianceCheck {
                    name: "CB-200: TDG Grade Gate".to_string(),
                    status: CheckStatus::Skip,
                    message: format!("Query failed: {e}"),
                    severity: Severity::Info,
                };
            }
        }
    };

    // Merge exclude patterns from .pmat.yaml and .pmat-gates.toml (#221)
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
        .filter(|v| {
            // Exclude test files
            if v.file_path.contains("/tests/")
                || v.file_path.contains("/test/")
                || v.file_path.ends_with("_test.rs")
                || v.file_path.ends_with("_tests.rs")
            {
                return false;
            }
            // Exclude configured paths
            let opts = glob::MatchOptions {
                case_sensitive: true,
                require_literal_separator: false,
                require_literal_leading_dot: false,
            };
            !exclude_patterns
                .iter()
                .any(|pat| pat.matches_with(&v.file_path, opts))
        })
        .collect();

    let count = filtered.len();
    if count == 0 {
        return ComplianceCheck {
            name: "CB-200: TDG Grade Gate".to_string(),
            status: CheckStatus::Pass,
            message: format!(
                "All non-test functions meet minimum grade {min_grade}{}",
                if violations.is_empty() {
                    String::new()
                } else {
                    format!(" ({} test/excluded functions skipped)", violations.len())
                }
            ),
            severity: Severity::Info,
        };
    }

    // Build detail lines (show up to 10)
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
        name: "CB-200: TDG Grade Gate".to_string(),
        status: CheckStatus::Fail,
        message: format!(
            "{count} function(s) below minimum grade {min_grade}\n{}",
            details.join("\n")
        ),
        severity: Severity::Error,
    }
}

/// CB-1100: Custom Project Scores
///
/// Runs project-specific score commands from .pmat.yaml `scoring.custom_scores`.
/// Each command must output JSON with a `score` field. Fails if below `min_score`.
pub(crate) fn check_custom_scores(
    project_path: &Path,
) -> Vec<ComplianceCheck> {
    let config = match crate::models::comply_config::PmatYamlConfig::load(project_path) {
        Ok(c) => c,
        Err(_) => return vec![],
    };

    if config.scoring.custom_scores.is_empty() {
        return vec![];
    }

    let mut checks = Vec::new();

    for score_def in &config.scoring.custom_scores {
        let check_name = format!("CB-1100: Custom Score [{}]", score_def.id);

        let output = std::process::Command::new("sh")
            .args(["-c", &score_def.command])
            .current_dir(project_path)
            .output();

        let output = match output {
            Ok(o) => o,
            Err(e) => {
                checks.push(ComplianceCheck {
                    name: check_name,
                    status: CheckStatus::Skip,
                    message: format!("Failed to run command: {e}"),
                    severity: Severity::Info,
                });
                continue;
            }
        };

        if !output.status.success() {
            checks.push(ComplianceCheck {
                name: check_name,
                status: CheckStatus::Fail,
                message: format!(
                    "{}: command failed (exit {})",
                    score_def.name,
                    output.status.code().unwrap_or(-1)
                ),
                severity: Severity::from(score_def.severity),
            });
            continue;
        }

        let stdout = String::from_utf8_lossy(&output.stdout);
        // Parse JSON output looking for {"score": N}
        let score: Option<f64> = extract_score_from_output(&stdout);

        match score {
            Some(actual_score) => {
                if let Some(min) = score_def.min_score {
                    if actual_score < min {
                        checks.push(ComplianceCheck {
                            name: check_name,
                            status: CheckStatus::Fail,
                            message: format!(
                                "{}: score {:.1} below minimum {:.1}",
                                score_def.name, actual_score, min
                            ),
                            severity: Severity::from(score_def.severity),
                        });
                    } else {
                        checks.push(ComplianceCheck {
                            name: check_name,
                            status: CheckStatus::Pass,
                            message: format!(
                                "{}: score {:.1} (min: {:.1})",
                                score_def.name, actual_score, min
                            ),
                            severity: Severity::Info,
                        });
                    }
                } else {
                    checks.push(ComplianceCheck {
                        name: check_name,
                        status: CheckStatus::Pass,
                        message: format!("{}: score {:.1}", score_def.name, actual_score),
                        severity: Severity::Info,
                    });
                }
            }
            None => {
                checks.push(ComplianceCheck {
                    name: check_name,
                    status: CheckStatus::Skip,
                    message: format!(
                        "{}: could not parse score from command output",
                        score_def.name
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    checks
}

/// Extract a numeric score from command output.
/// Looks for JSON `{"score": N}` pattern anywhere in the output.
fn extract_score_from_output(output: &str) -> Option<f64> {
    // Try to parse each line as JSON
    for line in output.lines() {
        let line = line.trim();
        if let Ok(json) = serde_json::from_str::<serde_json::Value>(line) {
            if let Some(score) = json.get("score").and_then(|s| s.as_f64()) {
                return Some(score);
            }
        }
    }
    // Fallback: try to find "SCORE: N" pattern
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

    #[test]
    fn test_grade_ordinal() {
        assert_eq!(grade_ordinal("A"), 0);
        assert_eq!(grade_ordinal("B"), 1);
        assert_eq!(grade_ordinal("C"), 2);
        assert_eq!(grade_ordinal("D"), 3);
        assert_eq!(grade_ordinal("F"), 4);
        assert_eq!(grade_ordinal("X"), 5);
    }

    #[test]
    fn test_grades_below() {
        assert_eq!(grades_below("A"), vec!["B", "C", "D", "F"]);
        assert_eq!(grades_below("B"), vec!["C", "D", "F"]);
        assert_eq!(grades_below("C"), vec!["D", "F"]);
        assert_eq!(grades_below("D"), vec!["F"]);
        assert!(grades_below("F").is_empty());
    }

    #[test]
    fn test_missing_db_returns_skip() {
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-missing-db");
        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(&tmp, &config);
        assert_eq!(result.status, CheckStatus::Skip);
        assert!(result.message.contains("run `pmat index`"));
    }

    #[test]
    fn test_grade_f_no_violations() {
        // min_grade F means nothing is below it
        let tmp = PathBuf::from("/tmp/pmat-test-tdg-grade-f");
        let mut config = ComplyConfig::default();
        config.thresholds.min_tdg_grade = "F".to_string();
        let result = check_tdg_grade_gate(&tmp, &config);
        // Either Skip (no db) or Pass (no grades below F)
        assert!(result.status == CheckStatus::Skip || result.status == CheckStatus::Pass);
    }

    #[test]
    fn test_in_memory_db_with_violations() {
        // Create a temp dir with a real SQLite DB
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");

        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS functions (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                function_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '',
                definition_type TEXT NOT NULL DEFAULT 'function',
                doc_comment TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                start_line INTEGER NOT NULL DEFAULT 0,
                end_line INTEGER NOT NULL DEFAULT 0,
                language TEXT NOT NULL DEFAULT 'Rust',
                checksum TEXT NOT NULL DEFAULT '',
                tdg_score REAL NOT NULL DEFAULT 0.0,
                tdg_grade TEXT NOT NULL DEFAULT 'A',
                complexity INTEGER NOT NULL DEFAULT 1,
                cognitive_complexity INTEGER NOT NULL DEFAULT 1,
                big_o TEXT NOT NULL DEFAULT 'O(1)',
                satd_count INTEGER NOT NULL DEFAULT 0,
                loc INTEGER NOT NULL DEFAULT 0,
                commit_count INTEGER NOT NULL DEFAULT 0,
                churn_score REAL NOT NULL DEFAULT 0.0,
                clone_count INTEGER NOT NULL DEFAULT 0,
                pattern_diversity REAL NOT NULL DEFAULT 0.0,
                fault_annotations TEXT NOT NULL DEFAULT '[]'
            )"
        ).expect("create schema");

        // Insert test data: one A, one D, one F, one D in test path
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/core.rs', 'good_fn', 'A', 5, 10)",
            [],
        ).expect("insert A");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/legacy.rs', 'bad_fn', 'D', 42, 20)",
            [],
        ).expect("insert D");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/awful.rs', 'terrible_fn', 'F', 60, 30)",
            [],
        ).expect("insert F");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/tests/helpers.rs', 'test_helper', 'D', 35, 40)",
            [],
        ).expect("insert test D");
        drop(conn);

        let config = ComplyConfig::default(); // min_tdg_grade = "A"
        let result = check_tdg_grade_gate(tmp.path(), &config);

        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("2 function(s) below minimum grade A"));
        assert!(result.message.contains("src/legacy.rs:20 bad_fn [D]"));
        assert!(result.message.contains("src/awful.rs:30 terrible_fn [F]"));
        // Test helper should be excluded
        assert!(!result.message.contains("test_helper"));
    }

    #[test]
    fn test_tdg_exclude_paths() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");

        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS functions (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                function_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '',
                definition_type TEXT NOT NULL DEFAULT 'function',
                doc_comment TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                start_line INTEGER NOT NULL DEFAULT 0,
                end_line INTEGER NOT NULL DEFAULT 0,
                language TEXT NOT NULL DEFAULT 'Rust',
                checksum TEXT NOT NULL DEFAULT '',
                tdg_score REAL NOT NULL DEFAULT 0.0,
                tdg_grade TEXT NOT NULL DEFAULT 'A',
                complexity INTEGER NOT NULL DEFAULT 1,
                cognitive_complexity INTEGER NOT NULL DEFAULT 1,
                big_o TEXT NOT NULL DEFAULT 'O(1)',
                satd_count INTEGER NOT NULL DEFAULT 0,
                loc INTEGER NOT NULL DEFAULT 0,
                commit_count INTEGER NOT NULL DEFAULT 0,
                churn_score REAL NOT NULL DEFAULT 0.0,
                clone_count INTEGER NOT NULL DEFAULT 0,
                pattern_diversity REAL NOT NULL DEFAULT 0.0,
                fault_annotations TEXT NOT NULL DEFAULT '[]'
            )"
        ).expect("create schema");

        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('vendor/lib.rs', 'vendor_fn', 'D', 40, 10)",
            [],
        ).expect("insert vendor D");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/main.rs', 'main_fn', 'D', 30, 5)",
            [],
        ).expect("insert src D");
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
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS functions (
                id INTEGER PRIMARY KEY,
                file_path TEXT NOT NULL,
                function_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '',
                definition_type TEXT NOT NULL DEFAULT 'function',
                doc_comment TEXT NOT NULL DEFAULT '',
                source TEXT NOT NULL DEFAULT '',
                start_line INTEGER NOT NULL DEFAULT 0,
                end_line INTEGER NOT NULL DEFAULT 0,
                language TEXT NOT NULL DEFAULT 'Rust',
                checksum TEXT NOT NULL DEFAULT '',
                tdg_score REAL NOT NULL DEFAULT 0.0,
                tdg_grade TEXT NOT NULL DEFAULT 'A',
                complexity INTEGER NOT NULL DEFAULT 1,
                cognitive_complexity INTEGER NOT NULL DEFAULT 1,
                big_o TEXT NOT NULL DEFAULT 'O(1)',
                satd_count INTEGER NOT NULL DEFAULT 0,
                loc INTEGER NOT NULL DEFAULT 0,
                commit_count INTEGER NOT NULL DEFAULT 0,
                churn_score REAL NOT NULL DEFAULT 0.0,
                clone_count INTEGER NOT NULL DEFAULT 0,
                pattern_diversity REAL NOT NULL DEFAULT 0.0,
                fault_annotations TEXT NOT NULL DEFAULT '[]'
            )"
        ).expect("create schema");

        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)",
            [],
        ).expect("insert A");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)",
            [],
        ).expect("insert B");
        drop(conn);

        // Override to min_tdg_grade = "B" to test that both A and B pass
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
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS functions (
                id INTEGER PRIMARY KEY, file_path TEXT NOT NULL,
                function_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function',
                doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '',
                start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0,
                language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '',
                tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A',
                complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1,
                big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0,
                loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0,
                churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0,
                pattern_diversity REAL NOT NULL DEFAULT 0.0,
                fault_annotations TEXT NOT NULL DEFAULT '[]'
            )"
        ).expect("create schema");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/lib.rs', 'good_fn', 'A', 3, 1)", [],
        ).expect("insert A");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/util.rs', 'ok_fn', 'B', 8, 10)", [],
        ).expect("insert B");
        drop(conn);

        // Write .pmat-gates.toml with min_grade = "B" (#221)
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nmin_grade = \"B\"\n",
        ).expect("write gates toml");

        let config = ComplyConfig::default(); // default is "A"
        let result = check_tdg_grade_gate(tmp.path(), &config);
        // .pmat-gates.toml overrides to "B", so B function should pass
        assert_eq!(result.status, CheckStatus::Pass);
    }

    #[test]
    fn test_pmat_gates_toml_exclude_override() {
        let tmp = tempfile::tempdir().expect("create tempdir");
        let pmat_dir = tmp.path().join(".pmat");
        std::fs::create_dir_all(&pmat_dir).expect("create .pmat");
        let db_path = pmat_dir.join("context.db");

        let conn = rusqlite::Connection::open(&db_path).expect("open db");
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS functions (
                id INTEGER PRIMARY KEY, file_path TEXT NOT NULL,
                function_name TEXT NOT NULL,
                signature TEXT NOT NULL DEFAULT '', definition_type TEXT NOT NULL DEFAULT 'function',
                doc_comment TEXT NOT NULL DEFAULT '', source TEXT NOT NULL DEFAULT '',
                start_line INTEGER NOT NULL DEFAULT 0, end_line INTEGER NOT NULL DEFAULT 0,
                language TEXT NOT NULL DEFAULT 'Rust', checksum TEXT NOT NULL DEFAULT '',
                tdg_score REAL NOT NULL DEFAULT 0.0, tdg_grade TEXT NOT NULL DEFAULT 'A',
                complexity INTEGER NOT NULL DEFAULT 1, cognitive_complexity INTEGER NOT NULL DEFAULT 1,
                big_o TEXT NOT NULL DEFAULT 'O(1)', satd_count INTEGER NOT NULL DEFAULT 0,
                loc INTEGER NOT NULL DEFAULT 0, commit_count INTEGER NOT NULL DEFAULT 0,
                churn_score REAL NOT NULL DEFAULT 0.0, clone_count INTEGER NOT NULL DEFAULT 0,
                pattern_diversity REAL NOT NULL DEFAULT 0.0,
                fault_annotations TEXT NOT NULL DEFAULT '[]'
            )"
        ).expect("create schema");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/core_generated.rs', 'gen_fn', 'D', 40, 10)", [],
        ).expect("insert generated D");
        conn.execute(
            "INSERT INTO functions (file_path, function_name, tdg_grade, complexity, start_line)
             VALUES ('src/real.rs', 'real_fn', 'D', 30, 5)", [],
        ).expect("insert real D");
        drop(conn);

        // Write .pmat-gates.toml with exclude for generated files (#221)
        std::fs::write(
            tmp.path().join(".pmat-gates.toml"),
            "[tdg]\nexclude = [\"**/*_generated.rs\"]\n",
        ).expect("write gates toml");

        let config = ComplyConfig::default();
        let result = check_tdg_grade_gate(tmp.path(), &config);
        assert_eq!(result.status, CheckStatus::Fail);
        assert!(result.message.contains("1 function(s)"));
        assert!(result.message.contains("real_fn"));
        assert!(!result.message.contains("gen_fn"));
    }
}
