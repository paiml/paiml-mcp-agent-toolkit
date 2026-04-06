// CB-148: Spec-Work Traceability
// Warns when spec Planned sections have no corresponding pmat work tickets,
// or when work items reference nonexistent specs.

use super::types::{CbPatternViolation, Severity};
use std::path::Path;

/// CB-148: Detect specs with planned sections that have no work tickets.
pub fn detect_cb148_spec_work_gaps(project_path: &Path) -> Vec<CbPatternViolation> {
    debug_assert!(
        project_path.exists(),
        "project_path must exist: {}",
        project_path.display()
    );
    let specs_dir = project_path.join("docs/specifications/components");
    if !specs_dir.exists() {
        return Vec::new();
    }

    let mut violations = Vec::new();
    let work_dir = project_path.join(".pmat-work");
    let roadmap = project_path.join("docs/roadmaps/roadmap.yaml");

    // Collect existing ticket IDs from .pmat-work/ and roadmap
    let ticket_ids = collect_ticket_ids(&work_dir, &roadmap);

    // Scan each spec for planned markers
    let entries = match std::fs::read_dir(&specs_dir) {
        Ok(e) => e,
        Err(_) => return Vec::new(),
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().map_or(true, |e| e != "md") {
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
            // Look for planned markers in section headers
            let trimmed = line.trim();
            let is_planned = trimmed.contains("(Planned)")
                || trimmed.contains("[Planned]")
                || trimmed.contains("(planned)")
                || trimmed.contains("[planned]");

            if !is_planned {
                continue;
            }

            // Check if any ticket references this spec section
            let has_ticket = ticket_ids.iter().any(|id| {
                // Check if roadmap/tickets mention this spec file
                content.contains(id)
            });

            if !has_ticket {
                violations.push(CbPatternViolation {
                    pattern_id: "CB-148".to_string(),
                    file: format!("docs/specifications/components/{filename}"),
                    line: i + 1,
                    description: format!(
                        "Planned section has no work ticket: {}",
                        trimmed.chars().take(80).collect::<String>()
                    ),
                    severity: Severity::Info,
                });
            }
        }
    }

    violations
}

fn collect_ticket_ids(work_dir: &Path, roadmap: &Path) -> Vec<String> {
    debug_assert!(
        work_dir.exists(),
        "work_dir must exist: {}",
        work_dir.display()
    );
    debug_assert!(
        roadmap.exists(),
        "roadmap must exist: {}",
        roadmap.display()
    );
    let mut ids = Vec::new();

    // From .pmat-work/ directories
    if let Ok(entries) = std::fs::read_dir(work_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                ids.push(entry.file_name().to_string_lossy().to_string());
            }
        }
    }

    // From roadmap.yaml ticket IDs
    if let Ok(content) = std::fs::read_to_string(roadmap) {
        for line in content.lines() {
            let trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("- id:") {
                ids.push(rest.trim().to_string());
            }
            if let Some(rest) = trimmed.strip_prefix("id:") {
                ids.push(rest.trim().to_string());
            }
        }
    }

    ids
}
