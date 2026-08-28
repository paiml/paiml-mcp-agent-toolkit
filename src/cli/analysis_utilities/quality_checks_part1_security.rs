// Security checking functions - extracted from quality_checks_part1.rs (CB-040)
async fn check_security(project_path: &Path) -> Result<Vec<QualityViolation>> {
    check_security_with_scope(project_path)
        .await
        .map(|(v, _)| v)
}

/// How much of the tree the hardcoded-credential scan actually read.
///
/// `read` is the number of source files it opened; `descended` says whether it
/// looked below the project root at all. Both are reported because 0 findings
/// from 12 files in one directory and 0 findings from a whole repository are
/// not the same claim, and until this existed they were the same output.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct SecurityScope {
    pub read: usize,
    pub descended: bool,
}

/// The scan, plus the population it covered.
///
/// The scan itself is unchanged and deliberately so: it walks
/// `read_dir(project_path)` — ONE directory, no recursion — which for any
/// ordinary Rust project means it reads the handful of files sitting beside
/// `Cargo.toml` and never opens `src/`. Two byte-identical secrets, one in
/// `root_leak.rs` and one in `src/deep_leak.rs`:
///
/// ```text
///   "security_violations": 1        # root_leak.rs
///   "files_examined": 2             # both files counted as examined
/// ```
///
/// and on this repository, `--checks security` reports `security_violations: 0`
/// beside `files_examined: 5363`. Widening the walk is a behaviour change that
/// turns previously-passing gates red on a real finding, so it is left to be
/// decided deliberately rather than as a side effect of a disclosure fix (#1035
/// cluster 1: the scope is silently narrower than the claim). What is fixed here
/// is the SILENCE: the gate now states the population its zero was measured
/// over, so the narrowing is visible in the report instead of only in this
/// function's body.
async fn check_security_with_scope(
    project_path: &Path,
) -> Result<(Vec<QualityViolation>, SecurityScope)> {
    let mut violations = Vec::new();
    let patterns = get_security_patterns();
    let mut read = 0usize;

    use tokio::fs;

    if let Ok(mut entries) = fs::read_dir(project_path).await {
        while let Some(entry) = entries.next_entry().await? {
            let path = entry.path();
            if path.is_file() && is_source_file(&path) {
                read += 1;
                check_file_security(&path, &patterns, &mut violations).await?;
            }
        }
    }

    Ok((
        violations,
        SecurityScope {
            read,
            descended: false,
        },
    ))
}

/// The row that stops a narrow scan from reading as a clean tree.
///
/// Advisory, not blocking: the scan's reach is a property of pmat, not a defect
/// in the project being scanned, and failing every user's gate over our own
/// limitation would be the opposite of useful. It is still a row in the report,
/// which is the whole point — `security_violations: 0` alone is a claim about
/// the repository, and this says which repository-shaped thing it is a claim
/// about.
fn security_scope_disclosure(project_path: &Path, scope: SecurityScope) -> QualityViolation {
    QualityViolation {
        check_type: "scope".to_string(),
        severity: ADVISORY_SEVERITY.to_string(),
        file: project_path.display().to_string(),
        line: None,
        message: format!(
            "the security scan read {} source file(s) directly under {} and did NOT descend \
             into subdirectories, so anything under src/ was not examined — a zero here is \
             the scope's, not the tree's",
            scope.read,
            project_path.display()
        ),
        details: None,
    }
}
