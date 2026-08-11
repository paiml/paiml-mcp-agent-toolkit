//! YAML Roadmap Validation Test
//!
//! Finds acceptance_criteria entries that are maps instead of strings
//! Created to fix: "roadmap[10].acceptance_criteria[3]: invalid type: map, expected a string"

use pmat::models::roadmap::Roadmap;
use std::fs;

#[test]
fn validate_roadmap_acceptance_criteria() {
    // #709: the path used to start with `..`, stale since the workspace was
    // flattened to a single crate. Relative to the package root that escapes
    // the repo entirely, so the test read whatever happened to sit next to
    // the checkout (or panicked in `expect` on a clean one) and never looked
    // at the roadmap it is named for. Anchor on the manifest dir instead.
    let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/roadmaps/roadmap.yaml");
    let yaml_content = fs::read_to_string(yaml_path).expect("Failed to read roadmap.yaml");

    // Try to deserialize - this will show the exact error
    let result: Result<Roadmap, _> = serde_yaml_ng::from_str(&yaml_content);

    match result {
        Ok(roadmap) => {
            println!("✅ YAML parsed successfully");
            println!("Total roadmap items: {}", roadmap.roadmap.len());

            // Manually check each item's acceptance_criteria
            let mut checked = 0usize;
            for (idx, item) in roadmap.roadmap.iter().enumerate() {
                println!("\nItem {}: {} ({})", idx, item.id, item.title);
                println!(
                    "  acceptance_criteria count: {}",
                    item.acceptance_criteria.len()
                );

                for (criteria_idx, criteria) in item.acceptance_criteria.iter().enumerate() {
                    println!("    [{}]: {}", criteria_idx, criteria);
                    checked += 1;
                }
            }

            // #709: with the old `..` path this validator read a one-item file
            // with `acceptance_criteria: []`, so the loop above ran zero times
            // and the test passed without validating a single criterion.
            assert!(
                checked > 0,
                "validated no acceptance_criteria at all - wrong roadmap file?"
            );
        }
        Err(e) => {
            eprintln!("❌ YAML PARSE ERROR:");
            eprintln!("{}", e);
            eprintln!("\nThis error shows the EXACT location and type mismatch.");
            panic!("Roadmap YAML validation failed - see error above");
        }
    }
}

#[test]
fn validate_roadmap_with_raw_yaml() {
    // Parse as raw YAML first to inspect structure
    // #709: manifest-anchored — see the note on the test above.
    let yaml_path = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/roadmaps/roadmap.yaml");
    let yaml_content = fs::read_to_string(yaml_path).expect("Failed to read roadmap.yaml");

    let raw_yaml: serde_yaml_ng::Value =
        serde_yaml_ng::from_str(&yaml_content).expect("Failed to parse as raw YAML");

    if let Some(roadmap_items) = raw_yaml.get("roadmap").and_then(|v| v.as_sequence()) {
        println!(
            "Scanning {} roadmap items for acceptance_criteria type mismatches...\n",
            roadmap_items.len()
        );

        let mut checked = 0usize;
        for (idx, item) in roadmap_items.iter().enumerate() {
            if let Some(id) = item.get("id").and_then(|v| v.as_str()) {
                if let Some(criteria) = item
                    .get("acceptance_criteria")
                    .and_then(|v| v.as_sequence())
                {
                    for (criteria_idx, criterion) in criteria.iter().enumerate() {
                        if !criterion.is_string() {
                            eprintln!("🔴 FOUND PROBLEM:");
                            eprintln!("  Item index: {}", idx);
                            eprintln!("  Item ID: {}", id);
                            eprintln!("  acceptance_criteria[{}] is NOT a string", criteria_idx);
                            eprintln!("  Type: {:?}", criterion);
                            eprintln!("  Value: {:?}", criterion);
                            panic!("Found non-string acceptance_criteria entry");
                        } else {
                            checked += 1;
                            println!(
                                "✅ Item {}: {} - criteria[{}]: string",
                                idx, id, criteria_idx
                            );
                        }
                    }
                } else if let Some(criteria) = item.get("acceptance_criteria") {
                    if !criteria.is_sequence() && !criteria.is_null() {
                        eprintln!(
                            "🔴 acceptance_criteria is not an array for item {}: {}",
                            idx, id
                        );
                        eprintln!("  Value: {:?}", criteria);
                        panic!("acceptance_criteria is not an array");
                    }
                }
            }
        }

        // #709: same vacuity guard as the test above - the file this used to
        // read had a single item with no acceptance_criteria, so "all entries
        // are strings" was true of the empty set.
        assert!(
            checked > 0,
            "scanned no acceptance_criteria at all - wrong roadmap file?"
        );

        println!("\n✅ All acceptance_criteria entries are strings");
    } else {
        panic!("Failed to find 'roadmap' array in YAML");
    }
}
