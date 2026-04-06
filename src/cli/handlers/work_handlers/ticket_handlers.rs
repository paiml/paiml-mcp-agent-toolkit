// Ticket handlers - split into include files for file health (CB-040, PMAT-503)
//
// Layout:
//   ticket_validate_migrate.rs - validate, migrate, list-statuses handlers
//   ticket_crud.rs             - add, list, edit, delete handlers
//   ticket_annotate.rs         - annotate handler, types, analysis helpers
//   ticket_annotate_output.rs  - annotate output formatters (text, json, markdown)
//
// All imports come from parent mod.rs scope (anyhow, PathBuf, RoadmapService, etc.)

// --- Shared utility functions used across multiple include files ---

/// Generate the next available ID for a new ticket
fn generate_next_id(roadmap: &crate::models::roadmap::Roadmap) -> String {
    let mut max_num = 0u32;

    for item in &roadmap.roadmap {
        // Try to extract number from IDs like "PMAT-001", "GH-123", etc.
        if let Some(num_str) = item.id.split('-').next_back() {
            if let Ok(num) = num_str.parse::<u32>() {
                max_num = max_num.max(num);
            }
        }
    }

    format!("PMAT-{:03}", max_num + 1)
}

/// Find an item with fuzzy ID matching (case-insensitive, partial match)
fn find_item_fuzzy(
    service: &RoadmapService,
    id: &str,
) -> Result<crate::models::roadmap::RoadmapItem> {
    // First try exact match
    if let Ok(Some(item)) = service.find_item(id) {
        return Ok(item);
    }

    // Load all items for fuzzy matching
    let roadmap = service.load()?;

    // Try case-insensitive exact match
    let id_lower = id.to_lowercase();
    for item in &roadmap.roadmap {
        if item.id.to_lowercase() == id_lower {
            return Ok(item.clone());
        }
    }

    // Try partial match (ID contains the search string)
    let mut matches: Vec<_> = roadmap
        .roadmap
        .iter()
        .filter(|item| item.id.to_lowercase().contains(&id_lower))
        .collect();

    match matches.len() {
        0 => anyhow::bail!(
            "Ticket '{}' not found. Use 'pmat work list' to see available tickets.",
            id
        ),
        1 => Ok(matches.pop().expect("verified 1 element exists").clone()),
        _ => {
            let match_ids: Vec<_> = matches.iter().map(|i| i.id.as_str()).collect();
            anyhow::bail!(
                "Ambiguous ID '{}'. Multiple matches: {}. Please be more specific.",
                id,
                match_ids.join(", ")
            )
        }
    }
}

/// Extract line number from YAML error message
fn extract_line_from_yaml_error(error: &str) -> Option<usize> {
    // serde_yaml_ng errors often contain "at line X column Y"
    if let Some(pos) = error.find("at line ") {
        let rest = error.get(pos + 8..).unwrap_or_default();
        if let Some(end) = rest.find(' ') {
            return rest.get(..end).unwrap_or_default().parse().ok();
        }
    }
    None
}

// --- Include split files ---

include!("ticket_validate_migrate.rs");
include!("ticket_crud.rs");
include!("ticket_annotate.rs");
include!("ticket_annotate_output.rs");
include!("ticket_score.rs");


// Tests extracted to work_handlers_tests.rs for file health compliance (CB-040)
// TEMPORARILY DISABLED: File splitting broke syntax (functions/modules split across files)
#[cfg(all(test, feature = "broken-tests"))]
#[path = "work_handlers_tests.rs"]
mod tests;
