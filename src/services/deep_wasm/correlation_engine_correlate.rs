// correlation_engine_correlate.rs — Primary DWARF + source map correlation logic
// Included by correlation_engine.rs — no `use` imports or inner attributes allowed

impl CorrelationEngine {
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
            .map(|e| (e.name.as_ref().expect("filtered for is_some").clone(), e))
            .collect();

        // Build source map by file
        let source_map_by_file: HashMap<String, Vec<&SourceMapEntry>> = {
            let mut map: HashMap<String, Vec<&SourceMapEntry>> = HashMap::new();
            for entry in source_map_entries {
                map.entry(entry.source.clone()).or_default().push(entry);
            }
            map
        };

        // Correlate DWARF entries (primary, high confidence)
        for (func_name, dwarf_entry) in &dwarf_functions {
            correlate_dwarf_entry(func_name, dwarf_entry, &source_map_by_file, &mut mappings);
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
                    wasm_function_idx: 0,       // Unknown without DWARF
                    wasm_instruction_offset: 0, // Unknown without DWARF
                    dwarf_die: None,
                    source_map_entry: Some((*entry).clone()),
                    confidence: 0.50, // Lower confidence: source map only
                });
            }
        }

        // Filter by confidence threshold
        mappings.retain(|m| m.confidence >= self.confidence_threshold);

        // Sort by confidence (highest first)
        mappings.sort_by(|a, b| b.confidence.total_cmp(&a.confidence));

        Ok(mappings)
    }

}

fn correlate_dwarf_entry(
    func_name: &str,
    dwarf_entry: &DwarfDebugEntry,
    source_map_by_file: &HashMap<String, Vec<&SourceMapEntry>>,
    mappings: &mut Vec<SourceToWasmMapping>,
) {
    debug_assert!(!func_name.is_empty(), "func_name must not be empty");
    let confidence = if let Some(source_entries) = source_map_by_file
        .values()
        .find(|entries| entries.iter().any(|e| e.name.as_deref() == Some(func_name)))
    {
        if let Some(source_entry) = source_entries
            .iter()
            .find(|e| e.name.as_deref() == Some(func_name))
        {
            mappings.push(SourceToWasmMapping {
                source_file: PathBuf::from(&source_entry.source),
                source_location: Location {
                    line: source_entry.original_line,
                    column: source_entry.original_column,
                },
                wasm_function_idx: estimate_function_index(dwarf_entry.die_offset),
                wasm_instruction_offset: 0,
                dwarf_die: Some((*dwarf_entry).clone()),
                source_map_entry: Some((*source_entry).clone()),
                confidence: 0.95,
            });
            return;
        }
        0.85
    } else {
        0.75
    };

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

impl CorrelationEngine {
    /// Create mapping from DWARF entry with default values
    #[allow(dead_code)] // May be used in future iterations
    fn create_dwarf_mapping(
        &self,
        dwarf_entry: &DwarfDebugEntry,
        confidence: f64,
    ) -> SourceToWasmMapping {
        debug_assert!(confidence >= 0.0, "confidence must be non-negative");
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
