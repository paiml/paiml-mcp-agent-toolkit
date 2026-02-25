// correlation_engine_unit_tests.rs — Unit tests for the correlation engine
// Included by correlation_engine.rs — no `use` imports or inner attributes allowed

#[cfg_attr(coverage_nightly, coverage(off))]
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

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 100,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("test_function".to_string()),
        }];

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

        let source_map_entries = vec![SourceMapEntry {
            generated_line: 10,
            generated_column: 5,
            original_line: 42,
            original_column: 8,
            source: "test.rs".to_string(),
            name: Some("helper_function".to_string()),
        }];

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

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 200,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("matched_function".to_string()),
        }];

        let source_map_entries = vec![SourceMapEntry {
            generated_line: 15,
            generated_column: 10,
            original_line: 100,
            original_column: 20,
            source: "lib.rs".to_string(),
            name: Some("matched_function".to_string()),
        }];

        let result = engine
            .correlate(&dwarf_entries, &source_map_entries)
            .unwrap();

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

        let result = engine
            .correlate(&dwarf_entries, &source_map_entries)
            .unwrap();

        // Should have: 1 high-confidence (func_b), 1 DWARF-only (func_a), 1 source-map-only (func_c)
        assert_eq!(result.len(), 3);

        // Results should be sorted by confidence (highest first)
        assert!(result[0].confidence >= result[1].confidence);
        assert!(result[1].confidence >= result[2].confidence);
    }

    #[test]
    fn test_confidence_threshold_filtering() {
        let engine = CorrelationEngine::with_confidence_threshold(0.8);

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 100,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("test_func".to_string()),
        }];

        let source_map_entries = vec![SourceMapEntry {
            generated_line: 10,
            generated_column: 5,
            original_line: 42,
            original_column: 8,
            source: "test.rs".to_string(),
            name: Some("other_func".to_string()),
        }];

        let result = engine
            .correlate(&dwarf_entries, &source_map_entries)
            .unwrap();

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
