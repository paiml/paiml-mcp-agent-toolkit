//! Correlation Engine Phase 2 Tests - EXTREME TDD RED Phase
//!
//! Tests for enhanced correlation with DWARF line program integration.
//! These tests MUST fail until implementation is complete.

#[cfg(test)]
mod line_program_integration_tests {
    use crate::services::deep_wasm::correlation_engine::CorrelationEngine;
    use crate::services::deep_wasm::{DwarfDebugEntry, Location, SourceMapEntry};

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_correlate_with_line_program_data() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 100,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("test_function".to_string()),
        }];

        // Line program data: (address, location) pairs
        let line_program_mappings = vec![
            (0x1000, Location { line: 42, column: 10 }),
            (0x1010, Location { line: 43, column: 15 }),
            (0x1020, Location { line: 44, column: 5 }),
        ];

        let source_map_entries = vec![SourceMapEntry {
            generated_line: 10,
            generated_column: 5,
            original_line: 42,
            original_column: 10,
            source: "test.rs".to_string(),
            name: Some("test_function".to_string()),
        }];

        // Enhanced correlate method with line program data
        let result = engine.correlate_with_line_programs(
            &dwarf_entries,
            &source_map_entries,
            &line_program_mappings,
        );

        assert!(result.is_ok());
        let mappings = result.unwrap();

        // Should create mapping with line program validation
        assert!(!mappings.is_empty());

        // Confidence should be higher when line numbers match
        let mapping = &mappings[0];
        assert!(mapping.confidence >= 0.95);

        // Should have accurate source location from line program
        assert_eq!(mapping.source_location.line, 42);
        assert_eq!(mapping.source_location.column, 10);
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_boost_confidence_with_line_number_match() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 200,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("validated_function".to_string()),
        }];

        // Line program shows function starts at line 100
        let line_program_mappings = vec![(0x2000, Location { line: 100, column: 5 })];

        // Source map shows same line number
        let source_map_entries = vec![SourceMapEntry {
            generated_line: 20,
            generated_column: 10,
            original_line: 100,
            original_column: 5,
            source: "lib.rs".to_string(),
            name: Some("validated_function".to_string()),
        }];

        let result = engine
            .correlate_with_line_programs(&dwarf_entries, &source_map_entries, &line_program_mappings)
            .unwrap();

        // Line number match should boost confidence above standard 0.95
        assert!(result[0].confidence >= 0.98);
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_reduce_confidence_with_line_number_mismatch() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 300,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("mismatched_function".to_string()),
        }];

        // Line program shows function at line 50
        let line_program_mappings = vec![(0x3000, Location { line: 50, column: 0 })];

        // Source map shows different line number
        let source_map_entries = vec![SourceMapEntry {
            generated_line: 30,
            generated_column: 0,
            original_line: 150, // Mismatch!
            original_column: 0,
            source: "main.rs".to_string(),
            name: Some("mismatched_function".to_string()),
        }];

        let result = engine
            .correlate_with_line_programs(&dwarf_entries, &source_map_entries, &line_program_mappings)
            .unwrap();

        // Line number mismatch should reduce confidence
        assert!(result[0].confidence < 0.95);
        assert!(result[0].confidence >= 0.70); // Still reasonable
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_use_line_program_for_accurate_source_locations() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 400,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("precise_function".to_string()),
        }];

        // Line program provides precise locations
        let line_program_mappings = vec![
            (0x4000, Location { line: 75, column: 12 }),
            (0x4004, Location { line: 76, column: 8 }),
            (0x4008, Location { line: 77, column: 4 }),
        ];

        let result = engine
            .correlate_with_line_programs(&dwarf_entries, &[], &line_program_mappings)
            .unwrap();

        // Should use line program data for source location even without source map
        assert!(!result.is_empty());
        assert_eq!(result[0].source_location.line, 75);
        assert_eq!(result[0].source_location.column, 12);
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_handle_empty_line_program_gracefully() {
        let engine = CorrelationEngine::new();

        let dwarf_entries = vec![DwarfDebugEntry {
            die_offset: 500,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("test".to_string()),
        }];

        // Empty line program
        let line_program_mappings = vec![];

        let result = engine.correlate_with_line_programs(&dwarf_entries, &[], &line_program_mappings);

        // Should handle gracefully and fall back to existing behavior
        assert!(result.is_ok());
        let mappings = result.unwrap();

        // Should still create mapping from DWARF entry
        assert!(!mappings.is_empty());
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_map_wasm_addresses_to_source_locations() {
        let engine = CorrelationEngine::new();

        // Line program with address mappings
        let line_program_mappings = vec![
            (0x1000, Location { line: 10, column: 0 }),
            (0x1100, Location { line: 20, column: 0 }),
            (0x1200, Location { line: 30, column: 0 }),
        ];

        // Query specific WASM address
        let result = engine.lookup_source_location(0x1100, &line_program_mappings);

        assert!(result.is_some());
        let location = result.unwrap();
        assert_eq!(location.line, 20);
        assert_eq!(location.column, 0);
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_handle_address_not_in_line_program() {
        let engine = CorrelationEngine::new();

        let line_program_mappings = vec![(0x1000, Location { line: 10, column: 0 })];

        // Query address not in mappings
        let result = engine.lookup_source_location(0x9999, &line_program_mappings);

        // Should return None for unmapped address
        assert!(result.is_none());
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_support_bidirectional_lookup() {
        let engine = CorrelationEngine::new();

        let line_program_mappings = vec![
            (0x1000, Location { line: 42, column: 10 }),
            (0x1010, Location { line: 43, column: 15 }),
            (0x1020, Location { line: 44, column: 5 }),
        ];

        // Forward: source → WASM
        let addresses = engine.lookup_wasm_addresses(42, &line_program_mappings);
        assert!(addresses.is_ok());
        assert_eq!(addresses.unwrap(), vec![0x1000]);

        // Reverse: WASM → source
        let location = engine.lookup_source_location(0x1010, &line_program_mappings);
        assert!(location.is_some());
        assert_eq!(location.unwrap().line, 43);
    }
}

#[cfg(test)]
mod enhanced_confidence_scoring_tests {
    use crate::services::deep_wasm::correlation_engine::CorrelationEngine;
    use crate::services::deep_wasm::{DwarfDebugEntry, Location, SourceMapEntry};

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_calculate_confidence_from_multiple_signals() {
        let engine = CorrelationEngine::new();

        // Perfect match: name + line + column all match
        let confidence = engine.calculate_confidence(
            true,  // has_dwarf
            true,  // has_source_map
            true,  // line_match
            true,  // column_match
        );

        assert_eq!(confidence, 1.0); // Perfect confidence
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_penalize_missing_signals() {
        let engine = CorrelationEngine::new();

        // DWARF + source map, but no line match
        let confidence1 = engine.calculate_confidence(true, true, false, false);
        assert!(confidence1 < 0.95);

        // DWARF only
        let confidence2 = engine.calculate_confidence(true, false, false, false);
        assert_eq!(confidence2, 0.75);

        // Source map only
        let confidence3 = engine.calculate_confidence(false, true, false, false);
        assert_eq!(confidence3, 0.50);
    }

    #[test]
    #[cfg(feature = "deep-wasm")]
    fn red_must_weight_line_match_higher_than_column_match() {
        let engine = CorrelationEngine::new();

        // Line match but no column match
        let with_line = engine.calculate_confidence(true, true, true, false);

        // Column match but no line match
        let with_column = engine.calculate_confidence(true, true, false, true);

        // Line match should have higher confidence
        assert!(with_line > with_column);
    }
}
