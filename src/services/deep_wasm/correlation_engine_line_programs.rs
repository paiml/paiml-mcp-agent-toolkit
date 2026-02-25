// correlation_engine_line_programs.rs — DWARF line program correlation and confidence scoring
// Included by correlation_engine.rs — no `use` imports or inner attributes allowed

impl CorrelationEngine {
    /// Enhanced correlation with DWARF line program data
    ///
    /// Integrates line program mappings (address -> location) for improved confidence scoring
    /// and more accurate source-to-WASM correlations.
    pub fn correlate_with_line_programs(
        &self,
        dwarf_entries: &[DwarfDebugEntry],
        source_map_entries: &[SourceMapEntry],
        line_program_mappings: &[(u64, Location)],
    ) -> DeepWasmResult<Vec<SourceToWasmMapping>> {
        let mut mappings = Vec::new();

        // Build DWARF function map
        let dwarf_functions: HashMap<String, &DwarfDebugEntry> = dwarf_entries
            .iter()
            .filter(|e| e.tag.contains("subprogram") && e.name.is_some())
            .map(|e| (e.name.as_ref().expect("filtered for is_some").clone(), e))
            .collect();

        // Build source map by file and name
        let mut source_map_by_name: HashMap<String, &SourceMapEntry> = HashMap::new();
        for entry in source_map_entries {
            if let Some(name) = &entry.name {
                source_map_by_name.insert(name.clone(), entry);
            }
        }

        // Correlate DWARF entries with line programs
        for (func_name, dwarf_entry) in &dwarf_functions {
            let source_map_entry = source_map_by_name.get(func_name.as_str()).copied();

            // Find line program entry closest to this DWARF entry's offset
            let line_program_location =
                self.find_location_for_dwarf_entry(dwarf_entry, line_program_mappings);

            // Calculate enhanced confidence
            let has_dwarf = true;
            let has_source_map = source_map_entry.is_some();
            let line_match =
                if let (Some(sm), Some(lp)) = (source_map_entry, &line_program_location) {
                    sm.original_line == lp.line
                } else {
                    false
                };
            let column_match =
                if let (Some(sm), Some(lp)) = (source_map_entry, &line_program_location) {
                    sm.original_column == lp.column
                } else {
                    false
                };

            let confidence =
                self.calculate_confidence(has_dwarf, has_source_map, line_match, column_match);

            // Create mapping with best available location data
            let (source_file, source_location) = match (source_map_entry, line_program_location) {
                (Some(sm), Some(lp)) if line_match => {
                    // Perfect match: use source map file + line program location
                    (PathBuf::from(&sm.source), lp)
                }
                (Some(sm), _) => {
                    // Use source map data
                    (
                        PathBuf::from(&sm.source),
                        Location {
                            line: sm.original_line,
                            column: sm.original_column,
                        },
                    )
                }
                (None, Some(lp)) => {
                    // Use line program data only
                    (PathBuf::from("unknown"), lp)
                }
                (None, None) => {
                    // No location data
                    (PathBuf::from("unknown"), Location { line: 0, column: 0 })
                }
            };

            mappings.push(SourceToWasmMapping {
                source_file,
                source_location,
                wasm_function_idx: estimate_function_index(dwarf_entry.die_offset),
                wasm_instruction_offset: 0,
                dwarf_die: Some((*dwarf_entry).clone()),
                source_map_entry: source_map_entry.cloned(),
                confidence,
            });
        }

        // Filter by confidence threshold
        mappings.retain(|m| m.confidence >= self.confidence_threshold);

        // Sort by confidence (highest first)
        mappings.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

        Ok(mappings)
    }

    /// Find location from line program for a given DWARF entry
    fn find_location_for_dwarf_entry(
        &self,
        _dwarf_entry: &DwarfDebugEntry,
        line_program_mappings: &[(u64, Location)],
    ) -> Option<Location> {
        // Return first mapping if available (simplified heuristic)
        // In production, would use DWARF entry's address range to find exact match
        line_program_mappings.first().map(|(_, loc)| loc.clone())
    }

    /// Calculate confidence score based on multiple signals
    ///
    /// Confidence scoring:
    /// - DWARF + SourceMap + Line + Column match: 1.0 (perfect)
    /// - DWARF + SourceMap + Line match: 0.98 (excellent)
    /// - DWARF + SourceMap + Column match: 0.92 (good, line more important)
    /// - DWARF + SourceMap (no line/column): 0.85 (moderate)
    /// - DWARF only: 0.75 (fair)
    /// - SourceMap only: 0.50 (weak)
    pub fn calculate_confidence(
        &self,
        has_dwarf: bool,
        has_source_map: bool,
        line_match: bool,
        column_match: bool,
    ) -> f64 {
        match (has_dwarf, has_source_map, line_match, column_match) {
            (true, true, true, true) => 1.0,    // Perfect match
            (true, true, true, false) => 0.98,  // Line match (most important)
            (true, true, false, true) => 0.92,  // Column match (less important)
            (true, true, false, false) => 0.85, // Name match only
            (true, false, _, _) => 0.75,        // DWARF only
            (false, true, _, _) => 0.50,        // SourceMap only
            (false, false, _, _) => 0.0,        // No data
        }
    }
}
