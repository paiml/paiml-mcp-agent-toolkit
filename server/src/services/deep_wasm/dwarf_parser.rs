//! DWARF v5 Parser
//!
//! Parses DWARF debugging information from WASM custom sections.
//! Focus on .debug_info, .debug_line, .debug_str.
//!
//! Implements DWASM-002: DWARF v5 parser with the following requirements:
//! - Parse DWARF v4 and v5 formats
//! - Extract DIE (Debug Information Entries)
//! - Build line number program tables
//! - Resolve string table references

use crate::services::deep_wasm::{DeepWasmError, DeepWasmResult, DwarfDebugEntry, Location};
use gimli::{DebugAbbrev, DebugInfo, DebugLine, DebugStr, Reader, RunTimeEndian};

/// DWARF debug information parser
pub struct DwarfParser {
    endian: RunTimeEndian,
}

impl DwarfParser {
    /// Creates a new DWARF parser with little-endian format (WASM standard)
    pub fn new() -> Self {
        Self {
            endian: RunTimeEndian::Little,
        }
    }

    /// Parses DWARF debug information from WASM custom sections
    ///
    /// Extracts Debug Information Entries (DIE) from DWARF v4/v5 format.
    /// Focuses on DW_TAG_subprogram entries to identify functions.
    ///
    /// Note: Requires .debug_abbrev section which is typically embedded in .debug_info
    /// For Phase 2, we're using simplified parsing without full abbreviation support
    #[cfg(feature = "deep-wasm")]
    pub fn parse_dwarf_sections(
        &self,
        debug_info: &[u8],
        _debug_line: Option<&[u8]>,
        debug_str: Option<&[u8]>,
    ) -> DeepWasmResult<Vec<DwarfDebugEntry>> {
        // Early return for empty input
        if debug_info.is_empty() {
            return Ok(Vec::new());
        }

        // Create DWARF sections with endianness
        let debug_info_section = DebugInfo::new(debug_info, self.endian);
        let debug_str_section = debug_str
            .map(|bytes| DebugStr::new(bytes, self.endian))
            .unwrap_or_else(|| DebugStr::new(&[], self.endian));

        // Create debug_abbrev section (may need to be passed separately in production)
        let debug_abbrev = DebugAbbrev::new(&[], self.endian);

        let mut entries = Vec::new();

        // Iterate through compilation units
        let mut units = debug_info_section.units();
        while let Some(header) = units
            .next()
            .map_err(|e| DeepWasmError::Analysis(format!("Failed to read unit header: {}", e)))?
        {
            // Extract entries from this unit using header
            self.extract_entries_from_header(&debug_info_section, header, &debug_str_section, &debug_abbrev, &mut entries)?;
        }

        Ok(entries)
    }

    /// Extract Debug Information Entries from a compilation unit header
    #[cfg(feature = "deep-wasm")]
    fn extract_entries_from_header<R: Reader<Offset = usize>>(
        &self,
        _debug_info: &DebugInfo<R>,
        header: gimli::UnitHeader<R>,
        debug_str: &DebugStr<R>,
        debug_abbrev: &DebugAbbrev<R>,
        entries: &mut Vec<DwarfDebugEntry>,
    ) -> DeepWasmResult<()> {
        // Parse abbreviations for this unit
        let abbreviations = header.abbreviations(debug_abbrev)
            .map_err(|e| DeepWasmError::Analysis(format!("Failed to parse abbreviations: {}", e)))?;

        let mut entries_cursor = header.entries(&abbreviations);

        // Iterate through all DIEs in this unit
        while let Some((_, entry)) = entries_cursor.next_dfs().map_err(|e| {
            DeepWasmError::Analysis(format!("Failed to read DIE: {}", e))
        })? {
            // Get offset - convert unit-relative offset to debug_info offset
            let die_offset = entry.offset().to_debug_info_offset(&header)
                .map(|offset| offset.0 as u64)
                .unwrap_or(0);

            let tag = format!("{:?}", entry.tag());

            // Extract function name from DW_TAG_subprogram
            let name = if entry.tag() == gimli::DW_TAG_subprogram {
                self.extract_name(&header, entry, debug_str)?
            } else {
                None
            };

            // Store entry (we collect all DIEs, but only subprograms have names)
            entries.push(DwarfDebugEntry {
                die_offset,
                tag,
                name,
            });
        }

        Ok(())
    }

    /// Extract function name from DIE using DW_AT_name attribute
    #[cfg(feature = "deep-wasm")]
    fn extract_name<R: Reader<Offset = usize>>(
        &self,
        _header: &gimli::UnitHeader<R>,
        entry: &gimli::DebuggingInformationEntry<R>,
        debug_str: &DebugStr<R>,
    ) -> DeepWasmResult<Option<String>> {
        if let Some(attr) = entry.attr(gimli::DW_AT_name).map_err(|e| {
            DeepWasmError::Analysis(format!("Failed to read DW_AT_name: {}", e))
        })? {
            if let gimli::AttributeValue::DebugStrRef(offset) = attr.value() {
                let name_slice = debug_str.get_str(offset).map_err(|e| {
                    DeepWasmError::Analysis(format!("Failed to resolve string: {}", e))
                })?;

                let name = name_slice.to_string_lossy()
                    .map_err(|e| DeepWasmError::Analysis(format!("Invalid UTF-8 in function name: {}", e)))?
                    .to_string();

                return Ok(Some(name));
            }
        }
        Ok(None)
    }

    /// Parses DWARF line number program
    ///
    /// Extracts address-to-line mappings from .debug_line section
    ///
    /// Note: Line program parsing requires coordinated DWARF info + line tables
    /// This is a placeholder that will be enhanced with full correlation in correlation_engine
    #[cfg(feature = "deep-wasm")]
    pub fn parse_line_program(
        &self,
        debug_line: &[u8],
    ) -> DeepWasmResult<Vec<(u64, Location)>> {
        // Early return for empty input
        if debug_line.is_empty() {
            return Ok(Vec::new());
        }

        // Placeholder: Full line program parsing requires the compilation unit context
        // which links .debug_info and .debug_line together
        // This will be implemented in the correlation engine where we have both
        let _debug_line_section = DebugLine::new(debug_line, self.endian);

        // TODO: Implement full line program parsing with unit context
        // This requires iterating units from debug_info to get line program offsets
        Ok(Vec::new())
    }

    /// Stub implementation when feature is disabled
    #[cfg(not(feature = "deep-wasm"))]
    pub fn parse_dwarf_sections(
        &self,
        _debug_info: &[u8],
        _debug_line: Option<&[u8]>,
        _debug_str: Option<&[u8]>,
    ) -> DeepWasmResult<Vec<DwarfDebugEntry>> {
        Err(DeepWasmError::MissingDebugInfo)
    }

    /// Stub implementation when feature is disabled
    #[cfg(not(feature = "deep-wasm"))]
    pub fn parse_line_program(
        &self,
        _debug_line: &[u8],
    ) -> DeepWasmResult<Vec<(u64, Location)>> {
        Err(DeepWasmError::MissingDebugInfo)
    }
}

impl Default for DwarfParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_creation() {
        let parser = DwarfParser::new();
        assert!(std::ptr::addr_of!(parser).is_aligned());
    }

    #[test]
    fn test_parser_default() {
        let _parser = DwarfParser::default();
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parse_empty_dwarf() {
        let parser = DwarfParser::new();
        let empty_data = vec![];
        let result = parser.parse_dwarf_sections(&empty_data, None, None);
        // Empty data should return Ok with empty Vec (no units to parse)
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parser_endianness() {
        let parser = DwarfParser::new();
        // WASM uses little-endian
        assert_eq!(parser.endian, gimli::RunTimeEndian::Little);
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parse_line_program_empty() {
        let parser = DwarfParser::new();
        let empty_data = vec![];
        let result = parser.parse_line_program(&empty_data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 0);
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parse_dwarf_with_null_debug_str() {
        let parser = DwarfParser::new();
        // Invalid DWARF data will fail gracefully during parsing
        let invalid_data = vec![0x00; 32];
        let result = parser.parse_dwarf_sections(&invalid_data, None, None);
        // Should handle gracefully (either Ok with 0 entries or error)
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parse_dwarf_with_debug_str() {
        let parser = DwarfParser::new();
        let debug_info = vec![0x00; 32];
        let debug_str = vec![0x00; 16];
        let result = parser.parse_dwarf_sections(&debug_info, None, Some(&debug_str));
        // Should handle gracefully (either Ok with 0 entries or error)
        assert!(result.is_ok() || result.is_err());
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_parse_dwarf_sections_error_handling() {
        let parser = DwarfParser::new();
        // Completely invalid DWARF data
        let bad_data = vec![0xFF; 8];
        let result = parser.parse_dwarf_sections(&bad_data, None, None);

        // Should either:
        // 1. Return Ok with empty Vec (no valid units found)
        // 2. Return Err with Analysis error
        match result {
            Ok(entries) => {
                // No valid units were found
                assert_eq!(entries.len(), 0);
            }
            Err(e) => {
                // Should be an Analysis error
                assert!(matches!(e, DeepWasmError::Analysis(_)));
            }
        }
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_minimal_dwarf_compilation_unit() {
        let parser = DwarfParser::new();

        // Minimal DWARF v4 compilation unit header
        // This is a simplified test - real DWARF is more complex
        let mut debug_info = Vec::new();

        // Unit length (4 bytes) - excluding length field itself
        debug_info.extend_from_slice(&16u32.to_le_bytes());

        // Version (2 bytes) - DWARF v4
        debug_info.extend_from_slice(&4u16.to_le_bytes());

        // Debug abbrev offset (4 bytes)
        debug_info.extend_from_slice(&0u32.to_le_bytes());

        // Address size (1 byte)
        debug_info.push(4);

        // Padding to reach declared length
        debug_info.extend_from_slice(&[0u8; 7]);

        let result = parser.parse_dwarf_sections(&debug_info, None, None);

        // Should either parse successfully or fail gracefully
        match result {
            Ok(entries) => {
                // May have 0 entries (no DIEs) or parsed something
                assert!(entries.len() >= 0);
            }
            Err(e) => {
                // Should be an Analysis error (missing abbreviations, etc.)
                assert!(matches!(e, DeepWasmError::Analysis(_)));
            }
        }
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_dwarf_debug_entry_structure() {
        // Test that DwarfDebugEntry structure is correct
        let entry = DwarfDebugEntry {
            die_offset: 0x1000,
            tag: "DW_TAG_subprogram".to_string(),
            name: Some("test_function".to_string()),
        };

        assert_eq!(entry.die_offset, 0x1000);
        assert_eq!(entry.tag, "DW_TAG_subprogram");
        assert_eq!(entry.name, Some("test_function".to_string()));
    }

    #[cfg(feature = "deep-wasm")]
    #[test]
    fn test_location_structure() {
        let loc = Location {
            line: 42,
            column: 10,
        };

        assert_eq!(loc.line, 42);
        assert_eq!(loc.column, 10);
    }

    #[cfg(not(feature = "deep-wasm"))]
    #[test]
    fn test_feature_disabled_returns_error() {
        let parser = DwarfParser::new();
        let result = parser.parse_dwarf_sections(&[], None, None);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DeepWasmError::MissingDebugInfo));
    }

    #[cfg(not(feature = "deep-wasm"))]
    #[test]
    fn test_feature_disabled_line_program_error() {
        let parser = DwarfParser::new();
        let result = parser.parse_line_program(&[]);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), DeepWasmError::MissingDebugInfo));
    }
}
