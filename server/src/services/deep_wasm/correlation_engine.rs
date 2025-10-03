//! Correlation Engine
//!
//! Creates bidirectional mappings between source locations and WASM offsets.
//! Uses DWARF as primary source, source maps as fallback.
//!
//! Implements DWASM-010: Source-to-WASM correlation with:
//! - Bidirectional mapping (source ↔ WASM)
//! - DWARF as primary source
//! - Source maps as fallback
//! - Confidence scoring for each mapping

use crate::services::deep_wasm::{
    DeepWasmResult, DwarfDebugEntry, Location, SourceMapEntry, SourceToWasmMapping,
};
use std::collections::HashMap;
use std::path::PathBuf;

/// Estimate WASM function index from DWARF offset
/// This is a heuristic until we have full WASM module analysis
fn estimate_function_index(die_offset: u64) -> u32 {
    // Simple heuristic: assume functions are roughly 100 bytes apart in DWARF
    // This will be replaced with accurate mapping from WASM module analysis
    (die_offset / 100) as u32
}

/// Correlation engine for multi-layer mapping
pub struct CorrelationEngine {
    /// Confidence threshold for mappings (0.0-1.0)
    confidence_threshold: f64,
}

impl CorrelationEngine {
    pub fn new() -> Self {
        Self {
            confidence_threshold: 0.7, // 70% confidence minimum
        }
    }

    /// Create correlation engine with custom confidence threshold
    pub fn with_confidence_threshold(threshold: f64) -> Self {
        Self {
            confidence_threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Correlate DWARF and source map entries to create source-to-WASM mappings
    ///
    /// Strategy:
    /// 1. Use DWARF entries as primary source (higher confidence)
    /// 2. Use source map entries as fallback
    /// 3. Match by function name where possible
    /// 4. Assign confidence scores based on matching quality
    pub fn correlate(
        &self,
        dwarf_entries: &[DwarfDebugEntry],
        source_map_entries: &[SourceMapEntry],
    ) -> DeepWasmResult<Vec<SourceToWasmMapping>> {
        let mut mappings = Vec::new();

        // Build DWARF function map (name -> entry)
        let dwarf_functions: HashMap<String, &DwarfDebugEntry> = dwarf_entries
            .iter()
            .filter(|e| e.tag.contains("subprogram") && e.name.is_some())
            .map(|e| (e.name.as_ref().unwrap().clone(), e))
            .collect();

        // Build source map by file
        let source_map_by_file: HashMap<String, Vec<&SourceMapEntry>> = {
            let mut map: HashMap<String, Vec<&SourceMapEntry>> = HashMap::new();
            for entry in source_map_entries {
                map.entry(entry.source.clone())
                    .or_insert_with(Vec::new)
                    .push(entry);
            }
            map
        };

        // Correlate DWARF entries (primary, high confidence)
        for (func_name, dwarf_entry) in &dwarf_functions {
            // Try to find matching source map entry
            let confidence = if let Some(source_entries) = source_map_by_file.values().find(|entries| {
                entries.iter().any(|e| e.name.as_ref() == Some(func_name))
            }) {
                // Found exact name match in source map
                if let Some(source_entry) = source_entries.iter().find(|e| e.name.as_ref() == Some(func_name)) {
                    mappings.push(SourceToWasmMapping {
                        source_file: PathBuf::from(&source_entry.source),
                        source_location: Location {
                            line: source_entry.original_line,
                            column: source_entry.original_column,
                        },
                        wasm_function_idx: estimate_function_index(dwarf_entry.die_offset),
                        wasm_instruction_offset: 0, // Will be determined by WASM analysis
                        dwarf_die: Some((*dwarf_entry).clone()),
                        source_map_entry: Some((*source_entry).clone()),
                        confidence: 0.95, // High confidence: DWARF + source map match
                    });
                    continue;
                }
                0.85
            } else {
                0.75 // DWARF only, no source map correlation
            };

            // Create mapping from DWARF only
            mappings.push(SourceToWasmMapping {
                source_file: PathBuf::from("unknown"),
                source_location: Location { line: 0, column: 0 },
                wasm_function_idx: estimate_function_index(dwarf_entry.die_offset),
                wasm_instruction_offset: 0,
                dwarf_die: Some((*dwarf_entry).clone()),
                source_map_entry: None,
                confidence,
            });
        }

        // Add source map entries without DWARF correlation (fallback, lower confidence)
        for (source_file, entries) in &source_map_by_file {
            for entry in entries {
                // Skip if already correlated with DWARF
                if let Some(name) = &entry.name {
                    if dwarf_functions.contains_key(name) {
                        continue;
                    }
                }

                mappings.push(SourceToWasmMapping {
                    source_file: PathBuf::from(source_file),
                    source_location: Location {
                        line: entry.original_line,
                        column: entry.original_column,
                    },
                    wasm_function_idx: 0, // Unknown without DWARF
                    wasm_instruction_offset: 0,  // Unknown without DWARF
                    dwarf_die: None,
                    source_map_entry: Some((*entry).clone()),
                    confidence: 0.50, // Lower confidence: source map only
                });
            }
        }

        // Filter by confidence threshold
        mappings.retain(|m| m.confidence >= self.confidence_threshold);

        // Sort by confidence (highest first)
        mappings.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        Ok(mappings)
    }

    /// Create mapping from DWARF entry with default values
    #[allow(dead_code)] // May be used in future iterations
    fn create_dwarf_mapping(&self, dwarf_entry: &DwarfDebugEntry, confidence: f64) -> SourceToWasmMapping {
        SourceToWasmMapping {
            source_file: PathBuf::from("unknown"),
            source_location: Location { line: 0, column: 0 },
            wasm_function_idx: estimate_function_index(dwarf_entry.die_offset),
            wasm_instruction_offset: 0,
            dwarf_die: Some(dwarf_entry.clone()),
            source_map_entry: None,
            confidence,
        }
    }

}

impl Default for CorrelationEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_engine_creation() {
        let _engine = CorrelationEngine::new();
    }

    #[test]
    fn test_correlate_empty_inputs() {
        let engine = CorrelationEngine::new();
        let result = engine.correlate(&[], &[]);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[test]
    fn test_confidence_threshold() {
        let engine = CorrelationEngine::with_confidence_threshold(0.9);
        assert_eq!(engine.confidence_threshold, 0.9);

        // Test clamping
        let engine2 = CorrelationEngine::with_confidence_threshold(1.5);
        assert_eq!(engine2.confidence_threshold, 1.0);

        let engine3 = CorrelationEngine::with_confidence_threshold(-0.1);
        assert_eq!(engine3.confidence_threshold, 0.0);
    }

    #[test]
    fn test_correlate_dwarf_only() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![
            DwarfDebugEntry {
                die_offset: 100,
                tag: "DW_TAG_subprogram".to_string(),
                name: Some("test_function".to_string()),
            },
        ];

        let result = engine.correlate(&dwarf_entries, &[]).unwrap();

        // Should create mappings from DWARF only
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence, 0.75); // DWARF only
        assert!(result[0].dwarf_die.is_some());
        assert!(result[0].source_map_entry.is_none());
    }

    #[test]
    fn test_correlate_source_map_only() {
        let engine = CorrelationEngine::new();

        let source_map_entries = vec![
            SourceMapEntry {
                generated_line: 10,
                generated_column: 5,
                original_line: 42,
                original_column: 8,
                source: "test.rs".to_string(),
                name: Some("helper_function".to_string()),
            },
        ];

        let result = engine.correlate(&[], &source_map_entries).unwrap();

        // Should create mappings from source map only
        assert_eq!(result.len(), 0); // Filtered by default threshold (0.5 < 0.7)

        // Lower threshold to include source map only
        let engine2 = CorrelationEngine::with_confidence_threshold(0.4);
        let result2 = engine2.correlate(&[], &source_map_entries).unwrap();
        assert_eq!(result2.len(), 1);
        assert_eq!(result2[0].confidence, 0.50); // Source map only
    }

    #[test]
    fn test_correlate_dwarf_and_source_map_match() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![
            DwarfDebugEntry {
                die_offset: 200,
                tag: "DW_TAG_subprogram".to_string(),
                name: Some("matched_function".to_string()),
            },
        ];

        let source_map_entries = vec![
            SourceMapEntry {
                generated_line: 15,
                generated_column: 10,
                original_line: 100,
                original_column: 20,
                source: "lib.rs".to_string(),
                name: Some("matched_function".to_string()),
            },
        ];

        let result = engine.correlate(&dwarf_entries, &source_map_entries).unwrap();

        // Should create high-confidence mapping from DWARF + source map match
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].confidence, 0.95); // High confidence match
        assert!(result[0].dwarf_die.is_some());
        assert!(result[0].source_map_entry.is_some());
        assert_eq!(result[0].source_file, PathBuf::from("lib.rs"));
        assert_eq!(result[0].source_location.line, 100);
    }

    #[test]
    fn test_correlate_mixed_entries() {
        let engine = CorrelationEngine::with_confidence_threshold(0.5);

        let dwarf_entries = vec![
            DwarfDebugEntry {
                die_offset: 100,
                tag: "DW_TAG_subprogram".to_string(),
                name: Some("func_a".to_string()),
            },
            DwarfDebugEntry {
                die_offset: 200,
                tag: "DW_TAG_subprogram".to_string(),
                name: Some("func_b".to_string()),
            },
        ];

        let source_map_entries = vec![
            SourceMapEntry {
                generated_line: 10,
                generated_column: 5,
                original_line: 42,
                original_column: 8,
                source: "test.rs".to_string(),
                name: Some("func_b".to_string()), // Matches func_b
            },
            SourceMapEntry {
                generated_line: 20,
                generated_column: 15,
                original_line: 50,
                original_column: 10,
                source: "test.rs".to_string(),
                name: Some("func_c".to_string()), // No DWARF match
            },
        ];

        let result = engine.correlate(&dwarf_entries, &source_map_entries).unwrap();

        // Should have: 1 high-confidence (func_b), 1 DWARF-only (func_a), 1 source-map-only (func_c)
        assert_eq!(result.len(), 3);

        // Results should be sorted by confidence (highest first)
        assert!(result[0].confidence >= result[1].confidence);
        assert!(result[1].confidence >= result[2].confidence);
    }

    #[test]
    fn test_confidence_threshold_filtering() {
        let engine = CorrelationEngine::with_confidence_threshold(0.8);

        let dwarf_entries = vec![
            DwarfDebugEntry {
                die_offset: 100,
                tag: "DW_TAG_subprogram".to_string(),
                name: Some("test_func".to_string()),
            },
        ];

        let source_map_entries = vec![
            SourceMapEntry {
                generated_line: 10,
                generated_column: 5,
                original_line: 42,
                original_column: 8,
                source: "test.rs".to_string(),
                name: Some("other_func".to_string()),
            },
        ];

        let result = engine.correlate(&dwarf_entries, &source_map_entries).unwrap();

        // DWARF-only confidence (0.75) should be filtered out by threshold (0.8)
        // Source-map-only confidence (0.50) should be filtered out by threshold (0.8)
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_estimate_function_index() {
        assert_eq!(estimate_function_index(0), 0);
        assert_eq!(estimate_function_index(100), 1);
        assert_eq!(estimate_function_index(250), 2);
        assert_eq!(estimate_function_index(1000), 10);
    }
}
