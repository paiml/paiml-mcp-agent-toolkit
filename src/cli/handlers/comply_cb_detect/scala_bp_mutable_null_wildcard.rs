// CB-800: Mutable Collection Usage
// CB-801: Null Usage
// CB-802: Wildcard Import
//
// Included from scala_best_practices.rs — no `use` imports or `#!` attributes.

// =============================================================================
// CB-800: Mutable Collection Usage
// =============================================================================

/// Mutable collection types that should be avoided in non-local scope.
const MUTABLE_COLLECTIONS: &[&str] = &[
    "mutable.Map",
    "mutable.Set",
    "mutable.Buffer",
    "mutable.ListBuffer",
    "mutable.ArrayBuffer",
    "mutable.HashMap",
    "mutable.HashSet",
    "mutable.LinkedHashMap",
    "mutable.LinkedHashSet",
    "mutable.Queue",
    "mutable.Stack",
    "mutable.TreeMap",
    "mutable.TreeSet",
];

pub fn detect_cb800_mutable_collection(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            // Skip import lines (imports are fine)
            if line.starts_with("import ") {
                continue;
            }
            for mc in MUTABLE_COLLECTIONS {
                if line.contains(mc) {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-800".to_string(),
                        file: rel.clone(),
                        line: *line_num,
                        description: format!(
                            "Mutable collection `{}` — prefer immutable collections",
                            mc
                        ),
                        severity: Severity::Warning,
                    });
                    break; // One violation per line
                }
            }
        }
    }

    violations
}

// =============================================================================
// CB-801: Null Usage
// =============================================================================

pub fn detect_cb801_null_usage(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            // Look for `null` as a word (not inside identifiers like "nullable")
            if contains_null_literal(line) {
                // Allow Java interop annotations
                if line.contains("@Nullable")
                    || line.contains("@javax")
                    || line.contains("@java")
                    || line.contains("JNI")
                {
                    continue;
                }
                violations.push(CbPatternViolation {
                    pattern_id: "CB-801".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Null literal — use Option[T] instead".to_string(),
                    severity: Severity::Warning,
                });
            }
        }
    }

    violations
}

/// Check if line contains `null` as a standalone keyword.
fn contains_null_literal(line: &str) -> bool {
    let bytes = line.as_bytes();
    let null_bytes = b"null";
    let len = bytes.len();
    if len < 4 {
        return false;
    }
    for i in 0..=len - 4 {
        if &bytes[i..i + 4] == null_bytes {
            let before_ok = i == 0 || !bytes[i - 1].is_ascii_alphanumeric() && bytes[i - 1] != b'_';
            let after_ok =
                i + 4 >= len || !bytes[i + 4].is_ascii_alphanumeric() && bytes[i + 4] != b'_';
            if before_ok && after_ok {
                return true;
            }
        }
    }
    false
}

// =============================================================================
// CB-802: Unrestricted Wildcard Import
// =============================================================================

pub fn detect_cb802_wildcard_import(project_path: &Path) -> Vec<CbPatternViolation> {
    let files = walkdir_scala_files(project_path);
    let mut violations = Vec::new();

    for file_path in &files {
        if is_scala_test_file(file_path) {
            continue;
        }
        let content = match fs::read_to_string(file_path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let prod_lines = compute_scala_production_lines(&content);
        let rel = file_path
            .strip_prefix(project_path)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for (line_num, line) in &prod_lines {
            if !line.starts_with("import ") {
                continue;
            }
            // Scala 2: import pkg._ | Scala 3: import pkg.*
            if line.ends_with("._") || line.ends_with(".*") {
                // Allow standard library wildcards
                if is_allowed_wildcard_import(line) {
                    continue;
                }
                violations.push(CbPatternViolation {
                    pattern_id: "CB-802".to_string(),
                    file: rel.clone(),
                    line: *line_num,
                    description: "Wildcard import — import specific members".to_string(),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

/// Wildcard imports from standard library are generally acceptable.
fn is_allowed_wildcard_import(line: &str) -> bool {
    let allowed = [
        "scala.collection.",
        "scala.concurrent.",
        "scala.util.",
        "java.lang.",
        "java.util.",
        "scala.Predef.",
    ];
    allowed.iter().any(|prefix| line.contains(prefix))
}
