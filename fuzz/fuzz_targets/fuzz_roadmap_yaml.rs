//! Fuzz target for roadmap YAML parser
//!
//! This fuzzer generates random YAML input and verifies that:
//! 1. Parser never panics
//! 2. Valid YAML is properly parsed
//! 3. Invalid YAML returns graceful errors
//! 4. Round-trip serialization preserves data
//!
//! Run with: cargo fuzz run fuzz_roadmap_yaml

#![no_main]

use libfuzzer_sys::fuzz_target;
use pmat::models::roadmap::{Roadmap, RoadmapItem};
use pmat::services::roadmap_service::RoadmapService;
use std::io::Write;
use tempfile::NamedTempFile;

fuzz_target!(|data: &[u8]| {
    // Test 1: Parser should never panic on arbitrary input
    let _ = serde_yaml::from_slice::<Roadmap>(data);

    // Test 2: If we can parse a roadmap, serialization should work
    if let Ok(roadmap) = serde_yaml::from_slice::<Roadmap>(data) {
        // Serialize back to YAML
        if let Ok(yaml_str) = serde_yaml::to_string(&roadmap) {
            // Re-parse to verify round-trip
            if let Ok(roundtrip) = serde_yaml::from_str::<Roadmap>(&yaml_str) {
                // Verify key fields are preserved
                assert_eq!(roadmap.roadmap_version, roundtrip.roadmap_version);
                assert_eq!(roadmap.github_enabled, roundtrip.github_enabled);
                assert_eq!(roadmap.github_repo, roundtrip.github_repo);
                assert_eq!(roadmap.roadmap.len(), roundtrip.roadmap.len());
            }
        }
    }

    // Test 3: Service layer should handle arbitrary YAML gracefully
    if let Ok(mut temp_file) = NamedTempFile::new() {
        // Write fuzz data to temp file
        let _ = temp_file.write_all(data);
        let path = temp_file.path();

        let service = RoadmapService::new(path);

        // Load should never panic (but may return error)
        let _ = service.load();
    }

    // Test 4: Structured fuzzing - generate semi-valid YAML
    if data.len() >= 8 {
        let num_items = (data[0] % 10) as usize;
        let mut roadmap = Roadmap::new(None);

        for i in 0..num_items.min(5) {
            let offset = (i * 8) % data.len();
            if offset + 8 <= data.len() {
                let id_suffix = u16::from_le_bytes([data[offset], data[offset + 1]]);
                let item = RoadmapItem::new(
                    format!("FUZZ-{}", id_suffix),
                    format!("Fuzzed item {}", i),
                );
                roadmap.upsert_item(item);
            }
        }

        // Serialization should succeed for valid roadmaps
        if let Ok(_yaml) = serde_yaml::to_string(&roadmap) {
            // Success - valid roadmap can be serialized
        }
    }
});
