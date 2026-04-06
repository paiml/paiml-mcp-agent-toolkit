// CB-533: Stale Path References
// Detects paths in Makefiles and CI YAML that reference non-existent files/directories.
// Derived from empirical analysis: 20/241 fix commits (8%) were stale path fixes.

use super::types::{CbPatternViolation, Severity};
use std::path::Path;

/// CB-533: Detect stale path references in Makefiles and CI workflows.
#[provable_contracts_macros::contract("pmat-core.yaml", equation = "path_exists")]
pub fn detect_cb533_stale_path_references(project_path: &Path) -> Vec<CbPatternViolation> {
    let mut violations = Vec::new();

    // 1. Check Makefile
    check_makefile(project_path, &mut violations);

    // 2. Check CI workflows
    check_ci_workflows(project_path, &mut violations);

    violations
}

fn check_makefile(project_path: &Path, violations: &mut Vec<CbPatternViolation>) {
    let makefile = project_path.join("Makefile");
    if !makefile.exists() {
        return;
    }
    let content = match std::fs::read_to_string(&makefile) {
        Ok(c) => c,
        Err(_) => return,
    };

    for (i, line) in content.lines().enumerate() {
        let trimmed = line.trim();
        // Skip comments and empty lines
        if trimmed.starts_with('#') || trimmed.is_empty() {
            continue;
        }

        // Check `cd <dir>` patterns
        if let Some(pos) = trimmed.find("cd ") {
            let rest = &trimmed[pos + 3..];
            let dir = rest.split([';', '&', ' ', '/']).next().unwrap_or("");
            if !dir.is_empty()
                && !dir.starts_with('$')
                && !dir.starts_with('-')
                && dir != "."
                && dir != ".."
            {
                let target = project_path.join(dir);
                if !target.exists() {
                    violations.push(CbPatternViolation {
                        pattern_id: "CB-533".to_string(),
                        file: "Makefile".to_string(),
                        line: i + 1,
                        description: format!(
                            "Stale path: `cd {dir}` references non-existent directory"
                        ),
                        severity: Severity::Warning,
                    });
                }
            }
        }
    }
}

fn check_ci_workflows(project_path: &Path, violations: &mut Vec<CbPatternViolation>) {
    let workflows_dir = project_path.join(".github/workflows");
    if !workflows_dir.exists() {
        return;
    }
    let entries = match std::fs::read_dir(&workflows_dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "yml" && e != "yaml") {
            continue;
        }
        let content = match std::fs::read_to_string(&path) {
            Ok(c) => c,
            Err(_) => continue,
        };
        let filename = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        for (i, line) in content.lines().enumerate() {
            let trimmed = line.trim();

            // Check working-directory references
            if let Some(pos) = trimmed.find("working-directory:") {
                let dir = trimmed[pos + 18..]
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                if !dir.starts_with('$') && !dir.starts_with('.') {
                    let target = project_path.join(dir);
                    if !target.exists() {
                        violations.push(CbPatternViolation {
                            pattern_id: "CB-533".to_string(),
                            file: format!(".github/workflows/{filename}"),
                            line: i + 1,
                            description: format!(
                                "Stale path: `working-directory: {dir}` references non-existent directory"
                            ),
                            severity: Severity::Warning,
                        });
                    }
                }
            }
        }
    }
}
