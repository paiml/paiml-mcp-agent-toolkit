#![cfg_attr(coverage_nightly, coverage(off))]
//! Handler for `pmat sql` — direct SQL access to the function index database

use anyhow::{Context, Result};
use std::path::Path;

/// Output format for SQL results
#[derive(Debug, Clone, Copy)]
pub enum SqlOutputFormat {
    Table,
    Json,
    Csv,
}

impl SqlOutputFormat {
    pub fn from_str_opt(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "json" => Self::Json,
            "csv" => Self::Csv,
            _ => Self::Table,
        }
    }
}

/// Built-in example queries
const EXAMPLE_QUERIES: &[(&str, &str)] = &[
    (
        "grade-dist",
        "SELECT tdg_grade, count(*) as cnt FROM functions GROUP BY tdg_grade ORDER BY cnt DESC",
    ),
    (
        "worst-files",
        "SELECT file_path, count(*) as cnt FROM functions WHERE tdg_grade <> 'A' GROUP BY file_path ORDER BY cnt DESC LIMIT 20",
    ),
    (
        "satd-hotspots",
        "SELECT file_path, sum(satd_count) as satd FROM functions GROUP BY file_path HAVING sum(satd_count) > 5 ORDER BY satd DESC LIMIT 20",
    ),
    (
        "complex-funcs",
        "SELECT function_name, file_path, complexity, cognitive_complexity FROM functions WHERE complexity > 20 ORDER BY complexity DESC LIMIT 30",
    ),
    (
        "churn-leaders",
        "SELECT file_path, max(churn_score) as churn FROM functions GROUP BY file_path ORDER BY churn DESC LIMIT 20",
    ),
    (
        "lang-dist",
        "SELECT language, count(*) as cnt FROM functions GROUP BY language ORDER BY cnt DESC",
    ),
    (
        "grade-drivers",
        "SELECT CASE WHEN loc > 50 THEN 'LOC>50' ELSE 'LOC<=50' END as loc_bucket, CASE WHEN satd_count > 0 THEN 'HAS_SATD' ELSE 'NO_SATD' END as satd_bucket, CASE WHEN complexity >= 20 THEN 'CC>=20' ELSE 'CC<20' END as cc_bucket, count(*) as cnt FROM functions WHERE tdg_grade = 'B' GROUP BY 1, 2, 3 ORDER BY cnt DESC",
    ),
    (
        "big-funcs",
        "SELECT function_name, file_path, loc FROM functions WHERE loc > 100 ORDER BY loc DESC LIMIT 20",
    ),
    (
        "clone-clusters",
        "SELECT function_name, file_path, clone_count FROM functions WHERE clone_count > 2 ORDER BY clone_count DESC LIMIT 20",
    ),
    (
        "fault-density",
        "SELECT file_path, count(*) as fault_funcs FROM functions WHERE fault_annotations <> '[]' GROUP BY file_path ORDER BY fault_funcs DESC LIMIT 20",
    ),
    (
        "entropy-violations",
        "SELECT file_path, pattern_type, repetitions, variation_score, estimated_loc_reduction, severity FROM entropy_violations ORDER BY repetitions DESC LIMIT 20",
    ),
    (
        "entropy-by-severity",
        "SELECT severity, count(*) as cnt, sum(estimated_loc_reduction) as total_loc_savings FROM entropy_violations GROUP BY severity ORDER BY cnt DESC",
    ),
    (
        "low-provability",
        "SELECT file_path, function_name, provability_score, verified_properties FROM provability_scores WHERE provability_score < 0.5 ORDER BY provability_score ASC LIMIT 20",
    ),
    (
        "provability-summary",
        "SELECT CASE WHEN provability_score >= 0.8 THEN 'high' WHEN provability_score >= 0.5 THEN 'medium' ELSE 'low' END as tier, count(*) as cnt, round(avg(provability_score), 3) as avg_score FROM provability_scores GROUP BY tier ORDER BY avg_score",
    ),
    // quality_violations table — populated by `pmat quality-gate`
    (
        "violations",
        "SELECT check_type, severity, file_path, line, substr(message, 1, 80) as message FROM quality_violations ORDER BY check_type, severity DESC LIMIT 30",
    ),
    (
        "violation-summary",
        "SELECT check_type, severity, count(*) as cnt FROM quality_violations GROUP BY check_type, severity ORDER BY cnt DESC",
    ),
    (
        "complexity-violations",
        "SELECT file_path, line, message FROM quality_violations WHERE check_type = 'complexity' ORDER BY file_path LIMIT 30",
    ),
    (
        "satd-violations",
        "SELECT file_path, line, message FROM quality_violations WHERE check_type = 'satd' ORDER BY file_path LIMIT 30",
    ),
];

/// Handle --schema flag: print table schemas
pub fn handle_schema(db_path: &Path) -> Result<()> {
    let conn = open_readonly(db_path)?;
    let mut stmt =
        conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' ORDER BY name")?;
    let schemas: Vec<String> = stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    for schema in &schemas {
        println!("{};", schema);
        println!();
    }

    // Also show FTS5 virtual tables
    let mut fts_stmt =
        conn.prepare("SELECT sql FROM sqlite_master WHERE type='table' AND sql LIKE '%fts5%'")?;
    let fts_schemas: Vec<String> = fts_stmt
        .query_map([], |row| row.get::<_, String>(0))?
        .filter_map(|r| r.ok())
        .collect();

    if !fts_schemas.is_empty() {
        println!("-- FTS5 Virtual Tables:");
        for schema in &fts_schemas {
            println!("{};", schema);
            println!();
        }
    }

    Ok(())
}

/// Handle --examples flag: print built-in example queries
pub fn handle_examples() {
    println!("Built-in example queries (use with: pmat sql <name>):\n");
    for (name, query) in EXAMPLE_QUERIES {
        println!("  {name}:");
        println!("    {query}\n");
    }
}

/// Handle SQL query execution
pub fn handle_sql(query: &str, format: SqlOutputFormat, db_path: &Path) -> Result<()> {
    // Check if query is a named example
    let resolved_query = resolve_query(query);

    let conn = open_readonly(db_path)?;
    let mut stmt = conn
        .prepare(resolved_query)
        .with_context(|| format!("SQL error in: {resolved_query}"))?;

    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("?").to_string())
        .collect();

    let mut rows: Vec<Vec<String>> = Vec::new();
    let mut row_iter = stmt.query([])?;

    while let Some(row) = row_iter.next()? {
        let mut values = Vec::with_capacity(column_count);
        for i in 0..column_count {
            let val = format_cell(row, i);
            values.push(val);
        }
        rows.push(values);
    }

    match format {
        SqlOutputFormat::Table => print_table(&column_names, &rows),
        SqlOutputFormat::Json => print_json(&column_names, &rows),
        SqlOutputFormat::Csv => print_csv(&column_names, &rows),
    }

    Ok(())
}

/// Resolve a query: if it's a named example, return the example SQL
fn resolve_query(query: &str) -> &str {
    for (name, sql) in EXAMPLE_QUERIES {
        if *name == query {
            return sql;
        }
    }
    query
}

/// Format a single cell from a SQLite row
fn format_cell(row: &rusqlite::Row<'_>, idx: usize) -> String {
    // Try integer first, then float, then string
    if let Ok(v) = row.get::<_, i64>(idx) {
        return v.to_string();
    }
    if let Ok(v) = row.get::<_, f64>(idx) {
        return format!("{v:.2}");
    }
    if let Ok(v) = row.get::<_, String>(idx) {
        return v;
    }
    "NULL".to_string()
}

/// Open a read-only connection to the database
fn open_readonly(db_path: &Path) -> Result<rusqlite::Connection> {
    let conn = rusqlite::Connection::open_with_flags(
        db_path,
        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY | rusqlite::OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .with_context(|| format!("Failed to open {}", db_path.display()))?;
    Ok(conn)
}

/// Print results as aligned table
fn print_table(columns: &[String], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("(0 rows)");
        return;
    }

    // Calculate column widths
    let mut widths: Vec<usize> = columns.iter().map(|c| c.len()).collect();
    for row in rows {
        for (i, val) in row.iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(val.len());
            }
        }
    }

    // Header
    let header: Vec<String> = columns
        .iter()
        .enumerate()
        .map(|(i, c)| format!("{:width$}", c, width = widths[i]))
        .collect();
    println!("{}", header.join(" | "));

    // Separator
    let sep: Vec<String> = widths.iter().map(|w| "-".repeat(*w)).collect();
    println!("{}", sep.join("-+-"));

    // Rows
    for row in rows {
        let formatted: Vec<String> = row
            .iter()
            .enumerate()
            .map(|(i, v)| {
                let w = widths.get(i).copied().unwrap_or(0);
                format!("{:width$}", v, width = w)
            })
            .collect();
        println!("{}", formatted.join(" | "));
    }

    println!("\n({} rows)", rows.len());
}

/// Print results as JSON array
fn print_json(columns: &[String], rows: &[Vec<String>]) {
    print!("[");
    for (i, row) in rows.iter().enumerate() {
        if i > 0 {
            print!(",");
        }
        print!("{{");
        for (j, val) in row.iter().enumerate() {
            if j > 0 {
                print!(",");
            }
            let key = columns.get(j).map_or("?", |s| s.as_str());
            // Try to output numbers without quotes
            if val.parse::<i64>().is_ok() || val.parse::<f64>().is_ok() {
                print!("\"{key}\":{val}");
            } else {
                let escaped = val.replace('\\', "\\\\").replace('"', "\\\"");
                print!("\"{key}\":\"{escaped}\"");
            }
        }
        print!("}}");
    }
    println!("]");
}

/// Print results as CSV
fn print_csv(columns: &[String], rows: &[Vec<String>]) {
    println!("{}", columns.join(","));
    for row in rows {
        let escaped: Vec<String> = row
            .iter()
            .map(|v| {
                if v.contains(',') || v.contains('"') || v.contains('\n') {
                    format!("\"{}\"", v.replace('"', "\"\""))
                } else {
                    v.clone()
                }
            })
            .collect();
        println!("{}", escaped.join(","));
    }
}

/// Locate the best database path for the project
pub fn find_db_path(project_path: &Path, workspace: bool) -> Result<std::path::PathBuf> {
    let db_name = if workspace {
        "workspace.db"
    } else {
        "context.db"
    };

    let db_path = project_path.join(".pmat").join(db_name);
    if db_path.exists() && std::fs::metadata(&db_path)?.len() > 0 {
        return Ok(db_path);
    }

    // Fallback: try the other DB
    let fallback = if workspace {
        "context.db"
    } else {
        "workspace.db"
    };
    let fallback_path = project_path.join(".pmat").join(fallback);
    if fallback_path.exists() && std::fs::metadata(&fallback_path)?.len() > 0 {
        return Ok(fallback_path);
    }

    anyhow::bail!(
        "No index database found at {db_path}. Run `pmat query \"test\" --rebuild-index` first.",
        db_path = db_path.display()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_resolve_query_named() {
        let result = resolve_query("grade-dist");
        assert!(result.contains("tdg_grade"));
        assert!(result.contains("GROUP BY"));
    }

    #[test]
    fn test_resolve_query_passthrough() {
        let sql = "SELECT 1";
        assert_eq!(resolve_query(sql), sql);
    }

    #[test]
    fn test_format_table_empty() {
        let cols = vec!["a".to_string()];
        // Just verify it doesn't panic
        print_table(&cols, &[]);
    }

    #[test]
    fn test_format_csv_escaping() {
        let cols = vec!["name".to_string(), "value".to_string()];
        let rows = vec![vec!["hello,world".to_string(), "normal".to_string()]];
        // Should not panic, comma-containing value gets quoted
        print_csv(&cols, &rows);
    }

    #[test]
    fn test_sql_output_format_from_str() {
        assert!(matches!(
            SqlOutputFormat::from_str_opt("json"),
            SqlOutputFormat::Json
        ));
        assert!(matches!(
            SqlOutputFormat::from_str_opt("csv"),
            SqlOutputFormat::Csv
        ));
        assert!(matches!(
            SqlOutputFormat::from_str_opt("table"),
            SqlOutputFormat::Table
        ));
        assert!(matches!(
            SqlOutputFormat::from_str_opt("unknown"),
            SqlOutputFormat::Table
        ));
    }

    #[test]
    fn test_find_db_path_missing() {
        let result = find_db_path(&PathBuf::from("/nonexistent"), false);
        assert!(result.is_err());
    }

    #[test]
    fn test_example_queries_valid_sql() {
        // Verify all example queries are syntactically valid by preparing them
        // against an in-memory SQLite with all table schemas
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE functions (
                id INTEGER PRIMARY KEY,
                file_path TEXT, function_name TEXT, signature TEXT,
                definition_type TEXT, doc_comment TEXT, source TEXT,
                start_line INTEGER, end_line INTEGER, language TEXT,
                checksum TEXT, tdg_score REAL, tdg_grade TEXT,
                complexity INTEGER, cognitive_complexity INTEGER,
                big_o TEXT, satd_count INTEGER, loc INTEGER,
                commit_count INTEGER, churn_score REAL,
                clone_count INTEGER, pattern_diversity REAL,
                fault_annotations TEXT
            );
            CREATE TABLE entropy_violations (
                id INTEGER PRIMARY KEY,
                file_path TEXT, pattern_type TEXT, pattern_hash TEXT,
                repetitions INTEGER, variation_score REAL,
                estimated_loc_reduction INTEGER, severity TEXT, example_code TEXT
            );
            CREATE TABLE provability_scores (
                id INTEGER PRIMARY KEY,
                function_id INTEGER, file_path TEXT, function_name TEXT,
                provability_score REAL, verified_properties INTEGER
            );
            CREATE TABLE quality_violations (
                id INTEGER PRIMARY KEY,
                check_type TEXT, severity TEXT, file_path TEXT,
                line INTEGER, message TEXT, details_json TEXT,
                created_at TEXT
            );",
        )
        .unwrap();

        for (name, sql) in EXAMPLE_QUERIES {
            conn.prepare(sql)
                .unwrap_or_else(|e| panic!("Example query '{name}' is invalid SQL: {e}"));
        }
    }

    #[test]
    fn test_handle_sql_with_in_memory_db() {
        let conn = rusqlite::Connection::open_in_memory().unwrap();
        conn.execute_batch(
            "CREATE TABLE functions (
                id INTEGER PRIMARY KEY,
                file_path TEXT, function_name TEXT, tdg_grade TEXT,
                complexity INTEGER, loc INTEGER, satd_count INTEGER
            );
            INSERT INTO functions VALUES (1, 'src/main.rs', 'main', 'A', 5, 20, 0);
            INSERT INTO functions VALUES (2, 'src/lib.rs', 'parse', 'B', 15, 80, 1);
            INSERT INTO functions VALUES (3, 'src/lib.rs', 'analyze', 'C', 25, 120, 3);",
        )
        .unwrap();

        // Save to temp file
        let dir = tempfile::tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        conn.execute(&format!("VACUUM INTO '{}'", db_path.display()), [])
            .unwrap();

        // Test table format
        let result = handle_sql(
            "SELECT tdg_grade, count(*) as cnt FROM functions GROUP BY tdg_grade ORDER BY cnt DESC",
            SqlOutputFormat::Table,
            &db_path,
        );
        assert!(result.is_ok());

        // Test named query
        let result = handle_sql("grade-dist", SqlOutputFormat::Table, &db_path);
        assert!(result.is_ok());

        // Test JSON format
        let result = handle_sql(
            "SELECT * FROM functions WHERE tdg_grade = 'A'",
            SqlOutputFormat::Json,
            &db_path,
        );
        assert!(result.is_ok());

        // Test CSV format
        let result = handle_sql(
            "SELECT function_name, complexity FROM functions",
            SqlOutputFormat::Csv,
            &db_path,
        );
        assert!(result.is_ok());
    }
}
