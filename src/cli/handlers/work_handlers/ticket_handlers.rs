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
//
// QUARANTINED. The reason recorded here was "File splitting broke syntax
// (functions/modules split across files)", and that is not what stops it
// (#1023). Two measurements:
//
//   1. The `#[path]` named a sibling `work_handlers_tests.rs` that does not
//      exist. The real file is one directory ABOVE, at
//      src/cli/handlers/work_handlers_tests.rs. A `#[path]` under a disabled
//      `cfg` is never resolved by the compiler, so the wrong value was
//      invisible; it is corrected below. Nothing could have reached a syntax
//      error, because nothing could find the file.
//   2. The split it blames is the ordinary `include!` kind, and `include!`
//      reassembles it. work_tests_part{1,2,3,4}.rs are individually unbalanced
//      (+3, 0/-2, +1/-2, -4) and their concatenation in include order balances
//      to exactly 0 — part1 opens `mod tests {`, part4 closes it. That is the
//      shape include! exists for, not a broken split.
//
// What actually blocks revival is unverified: this module moved from
// src/cli/handlers/work_handlers.rs into work_handlers/, so the tests' `use
// super::*` now resolves against a different module. Enabling it is a name-
// resolution repair across ~62 KB of tests, and no one has done it. That is
// the honest reason, and it replaces a false one.
#[cfg(all(test, feature = "broken-tests"))]
#[path = "../work_handlers_tests.rs"]
mod tests;

#[cfg(test)]
mod ticket_handlers_pure_tests {
    //! Covers pure-compute helpers in ticket_handlers.rs (46 uncov on broad,
    //! 0% cov). Skips fuzzy-match (requires populated roadmap fixture).
    use super::*;

    // ── extract_line_from_yaml_error ──

    #[test]
    fn test_extract_line_from_yaml_error_finds_at_line_pattern() {
        let err = "parse error at line 42 column 5: bad yaml";
        assert_eq!(extract_line_from_yaml_error(err), Some(42));
    }

    #[test]
    fn test_extract_line_from_yaml_error_no_at_line_pattern() {
        assert_eq!(extract_line_from_yaml_error("generic error"), None);
        assert_eq!(extract_line_from_yaml_error(""), None);
    }

    #[test]
    fn test_extract_line_from_yaml_error_at_line_without_space_after() {
        // No space after the number → end-of-string fallback returns None.
        let err = "at line 42";
        assert_eq!(extract_line_from_yaml_error(err), None);
    }

    #[test]
    fn test_extract_line_from_yaml_error_non_numeric_value() {
        let err = "at line abc column 1";
        assert_eq!(extract_line_from_yaml_error(err), None);
    }

    // ── generate_next_id ──

    #[test]
    fn test_generate_next_id_empty_roadmap_starts_at_001() {
        use crate::models::roadmap::Roadmap;
        let roadmap = Roadmap::new(None);
        assert_eq!(generate_next_id(&roadmap), "PMAT-001");
    }

    #[test]
    fn test_generate_next_id_picks_max_plus_one() {
        use crate::models::roadmap::{Roadmap, RoadmapItem};
        let mut roadmap = Roadmap::new(None);
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-005".into(), "x".into()));
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-100".into(), "y".into()));
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-042".into(), "z".into()));
        assert_eq!(generate_next_id(&roadmap), "PMAT-101");
    }

    #[test]
    fn test_generate_next_id_handles_mixed_id_prefixes() {
        use crate::models::roadmap::{Roadmap, RoadmapItem};
        let mut roadmap = Roadmap::new(None);
        roadmap
            .roadmap
            .push(RoadmapItem::new("GH-50".into(), "x".into()));
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-007".into(), "y".into()));
        // max(50, 7) = 50, so next is PMAT-051.
        assert_eq!(generate_next_id(&roadmap), "PMAT-051");
    }

    #[test]
    fn test_generate_next_id_skips_non_numeric_suffixes() {
        use crate::models::roadmap::{Roadmap, RoadmapItem};
        let mut roadmap = Roadmap::new(None);
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-XX".into(), "non-num".into()));
        roadmap
            .roadmap
            .push(RoadmapItem::new("PMAT-009".into(), "num".into()));
        // Non-numeric suffix is ignored; next is 9 + 1 = 10.
        assert_eq!(generate_next_id(&roadmap), "PMAT-010");
    }

    #[test]
    fn test_generate_next_id_pads_to_3_digits() {
        use crate::models::roadmap::Roadmap;
        let roadmap = Roadmap::new(None);
        // Empty → max=0, next=1 → "PMAT-001" (3-digit zero pad).
        let id = generate_next_id(&roadmap);
        assert_eq!(id, "PMAT-001");
    }
}
