//! YAML Roadmap Validation Test
//!
//! Finds acceptance_criteria entries that are maps instead of strings
//! Created to fix: "roadmap[10].acceptance_criteria[3]: invalid type: map, expected a string"

use pmat::models::roadmap::Roadmap;
use std::fs;

#[test]
fn validate_roadmap_acceptance_criteria() {
    let yaml_path = "../docs/roadmaps/roadmap.yaml";
    let yaml_content = fs::read_to_string(yaml_path).expect("Failed to read roadmap.yaml");

    // Try to deserialize - this will show the exact error
    let result: Result<Roadmap, _> = serde_yaml::from_str(&yaml_content);

    match result {
        Ok(roadmap) => {
            println!("✅ YAML parsed successfully");
            println!("Total roadmap items: {}", roadmap.roadmap.len());

            // Manually check each item's acceptance_criteria
            for (idx, item) in roadmap.roadmap.iter().enumerate() {
                println!("\nItem {}: {} ({})", idx, item.id, item.title);
                println!(
                    "  acceptance_criteria count: {}",
                    item.acceptance_criteria.len()
                );

                for (criteria_idx, criteria) in item.acceptance_criteria.iter().enumerate() {
                    println!("    [{}]: {}", criteria_idx, criteria);
                }
            }
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
    let yaml_path = "../docs/roadmaps/roadmap.yaml";
    let yaml_content = fs::read_to_string(yaml_path).expect("Failed to read roadmap.yaml");

    let raw_yaml: serde_yaml::Value =
        serde_yaml::from_str(&yaml_content).expect("Failed to parse as raw YAML");

    if let Some(roadmap_items) = raw_yaml.get("roadmap").and_then(|v| v.as_sequence()) {
        println!(
            "Scanning {} roadmap items for acceptance_criteria type mismatches...\n",
            roadmap_items.len()
        );

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

        println!("\n✅ All acceptance_criteria entries are strings");
    } else {
        panic!("Failed to find 'roadmap' array in YAML");
    }
}
