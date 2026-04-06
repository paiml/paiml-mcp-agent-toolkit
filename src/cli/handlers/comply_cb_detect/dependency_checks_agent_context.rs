// dependency_checks_agent_context.rs — included by dependency_checks.rs
// CB-130: Agent Context Adoption (PMAT-470)

/// Required patterns that should be in CLAUDE.md for agent context adoption
const REQUIRED_PATTERNS: &[&str] = &["pmat query", "NEVER use grep", "--faults"];

/// Forbidden patterns that indicate agents might use grep instead of pmat query
const FORBIDDEN_PATTERNS: &[&str] = &[
    "grep -r",
    "grep -rn",
    "find . -name",
    "find . -type f",
    "rg \"",
    "rg '",
];

/// Check the age of the RAG index by inspecting file metadata.
/// Prefers manifest.json mtime over directory mtime (directories don't update
/// when files inside are rewritten by --rebuild-index).
/// Returns (age_hours, is_stale). `is_stale` is true when age exceeds 24 hours.
fn check_index_age(index_path: &Path) -> (Option<f64>, bool) {
    let manifest_path = index_path.join("manifest.json");
    let check_path = if manifest_path.exists() {
        &manifest_path
    } else {
        index_path
    };
    let metadata = match fs::metadata(check_path) {
        Ok(m) => m,
        Err(_) => return (None, false),
    };
    let modified = match metadata.modified() {
        Ok(m) => m,
        Err(_) => return (None, false),
    };
    let age = std::time::SystemTime::now()
        .duration_since(modified)
        .unwrap_or_default();
    let hours = age.as_secs_f64() / 3600.0;
    (Some(hours), hours > 24.0)
}

/// Return true when the line appears to be a negative example (e.g. "never do X").
fn is_negative_example(line: &str) -> bool {
    debug_assert!(!line.is_empty(), "line must not be empty");
    let lower = line.to_lowercase();
    lower.contains("bad")
        || lower.contains("don't")
        || lower.contains("never")
        || lower.contains("avoid")
}

/// Scan content for forbidden patterns, excluding lines that are negative examples.
fn find_forbidden_patterns(content: &str) -> Vec<ForbiddenPatternMatch> {
    debug_assert!(!content.is_empty(), "content must not be empty");
    let mut forbidden = Vec::new();
    for (line_num, line) in content.lines().enumerate() {
        for &pattern in FORBIDDEN_PATTERNS {
            if line.contains(pattern) && !is_negative_example(line) {
                forbidden.push(ForbiddenPatternMatch {
                    pattern: pattern.to_string(),
                    line: line_num + 1,
                    context: line.chars().take(80).collect(),
                });
            }
        }
    }
    forbidden
}

/// Check CLAUDE.md for required and forbidden patterns.
/// Returns (configured, missing_required, forbidden_found).
/// `configured` is true when the file contains "pmat_query_code" or "pmat query".
fn check_claude_md_patterns(
    project_path: &Path,
) -> (bool, Vec<String>, Vec<ForbiddenPatternMatch>) {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let claude_md_path = project_path.join("CLAUDE.md");
    let content = match fs::read_to_string(&claude_md_path) {
        Ok(c) => c,
        Err(_) => {
            return (
                false,
                REQUIRED_PATTERNS.iter().map(|s| s.to_string()).collect(),
                vec![],
            );
        }
    };

    let configured = content.contains("pmat_query_code") || content.contains("pmat query");

    // Check for missing required patterns
    let missing: Vec<String> = REQUIRED_PATTERNS
        .iter()
        .filter(|&p| !content.to_lowercase().contains(&p.to_lowercase()))
        .map(|s| s.to_string())
        .collect();

    let forbidden = find_forbidden_patterns(&content);

    (configured, missing, forbidden)
}

/// CB-130: Detect agent context adoption issues
///
/// Checks:
/// 1. RAG index exists at .pmat/context.idx or .pmat/context.db
/// 2. Index is fresh (less than 24 hours old)
/// 3. CLAUDE.md references pmat_query_code (optional)
pub fn detect_cb130_agent_context_adoption(project_path: &Path) -> AgentContextReport {
    debug_assert!(project_path.exists(), "project_path must exist: {}", project_path.display());
    let index_path = project_path.join(".pmat/context.idx");
    let db_path = project_path.join(".pmat/context.db");

    let index_exists = index_path.exists() || db_path.exists();

    // Check freshness: prefer .db mtime, fall back to .idx/
    let age_check_path = if db_path.exists() {
        &db_path
    } else {
        &index_path
    };
    let (index_age_hours, index_stale) = if index_exists {
        check_index_age(age_check_path)
    } else {
        (None, false)
    };

    // Try to get function count from index
    let function_count = if index_exists {
        match crate::services::agent_context::AgentContextIndex::load(&index_path) {
            Ok(idx) => idx.manifest().function_count,
            Err(_) => 0,
        }
    } else {
        0
    };

    // Check CLAUDE.md for required and forbidden patterns
    let (claude_md_configured, missing_required_patterns, forbidden_patterns_found) =
        check_claude_md_patterns(project_path);

    AgentContextReport {
        index_exists,
        index_age_hours,
        index_stale,
        function_count,
        claude_md_configured,
        missing_required_patterns,
        forbidden_patterns_found,
    }
}
