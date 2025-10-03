# Deep WASM Phase 2 Implementation Plan

## Status: Phase 1 Complete (with deferrals)

### Phase 1 Completion Summary (v2.110.0)

**✅ Fully Implemented**:
1. ✅ WASM Binary Parser (wasm_inspector.rs)
   - Uses wasmparser 0.239
   - Module structure analysis
   - Function/import/export detection
   - Memory pattern tracking

2. ✅ Source Map Handler (source_map_handler.rs)
   - JavaScript-style source map parsing
   - VLQ decoding via sourcemap crate
   - Token mapping extraction
   - File loading and validation

3. ✅ Rust Analyzer for WASM (rust_wasm_analyzer.rs)
   - Detects #[wasm_bindgen] functions
   - extern "C" boundary detection
   - #[no_mangle] export tracking
   - Memory pattern analysis (Box, Vec, String)
   - 8 comprehensive tests

4. ✅ Infrastructure Complete:
   - CLI integration (`pmat analyze deep-wasm`)
   - 13 command-line options
   - 4 focus modes (source, compilation, runtime, interop, full)
   - MCP tools (5 tools for AI agent integration)
   - Quality gates with strict mode
   - Report generation (Markdown, JSON, HTML)

**⏳ Framework Only (Deferred to Phase 2)**:
1. ⏳ DWARF v5 Parser (dwarf_parser.rs) - DWASM-002
   - **Status**: Framework structure implemented
   - **Deferred**: Full gimli API integration
   - **Reason**: Complex gimli Reader/Abbreviations API requires deep DWARF expertise
   - **Current**: Returns empty Vec, compiles successfully

2. ⏳ Correlation Engine (correlation_engine.rs) - DWASM-010
   - **Status**: Framework with confidence scoring implemented
   - **Deferred**: Bidirectional source ↔ WASM mapping logic
   - **Reason**: Requires completed DWARF parser for source locations
   - **Current**: Placeholder returns empty Vec

3. ❌ Ruchy Analyzer - DWASM-005
   - **Status**: Not started
   - **Blocker**: Ruchy WASM compiler is broken (see Critical Issues below)
   - **Impact**: Cannot analyze Ruchy → WASM until compiler fixed

## Phase 2 Critical Tasks

### Priority 1: Complete DWARF v5 Parser (DWASM-002)

**Acceptance Criteria**:
- Parse DWARF v4 and v5 formats using gimli crate
- Extract DIE (Debug Information Entries) from .debug_info
- Build line number program tables from .debug_line
- Resolve string table references from .debug_str
- Map source locations to WASM instruction offsets

**Technical Requirements**:
```rust
// Required gimli API integration:
use gimli::{DebugInfo, DebugLine, DebugStr, DebugAbbrev};

// 1. Parse compilation units
let units = debug_info.units();

// 2. Extract DIE entries
for header in units {
    let abbrevs = debug_abbrev.abbreviations(&header)?;
    let unit = header.entries(&abbrevs)?;

    // Extract DW_TAG_subprogram (functions)
    // Extract DW_AT_name, DW_AT_decl_line, DW_AT_decl_file
}

// 3. Parse line number program
let line_program = debug_line.program(header, &debug_info, &debug_str)?;
let (address, location) mappings = line_program.rows();
```

**Implementation Steps**:
1. Study gimli::Reader API and EndianSlice requirements
2. Parse .debug_abbrev to get abbreviation tables
3. Use abbreviations to decode .debug_info DIE entries
4. Extract function names (DW_TAG_subprogram with DW_AT_name)
5. Parse line programs to get address-to-location mappings
6. Test with rustc-generated DWARF (cargo build --release)
7. Test with clang-generated DWARF (C/C++ projects)
8. Add fuzzing for malformed DWARF data

**Estimated Effort**: 3-5 days

### Priority 2: Complete Correlation Engine (DWASM-010)

**Acceptance Criteria**:
- Bidirectional mapping: source location → WASM offset
- Bidirectional mapping: WASM offset → source location
- DWARF as primary source (100% confidence)
- Source maps as fallback (75% confidence)
- Confidence scoring for each mapping
- O(log n) lookup time via binary search

**Technical Requirements**:
```rust
pub struct SourceToWasmMapping {
    pub source_file: PathBuf,
    pub source_location: Location,
    pub wasm_function_idx: u32,
    pub wasm_instruction_offset: u32,
    pub confidence: f64, // 0.0-1.0
}

impl CorrelationEngine {
    // Combine DWARF + source maps
    pub fn correlate(
        &self,
        dwarf_entries: &[DwarfDebugEntry],
        source_map_entries: &[SourceMapEntry],
        wasm_module: &WasmModule,
    ) -> Result<Vec<SourceToWasmMapping>>;

    // Bidirectional lookup
    pub fn find_wasm_offset(&self, file: &Path, line: u32, col: u32) -> Option<u32>;
    pub fn find_source_location(&self, wasm_offset: u32) -> Option<Location>;
}
```

**Implementation Steps**:
1. Extract WASM function offsets from wasmparser module
2. Correlate DWARF function names with WASM exports
3. Map DWARF line programs to WASM instruction offsets
4. Use source maps to fill gaps where DWARF is missing
5. Build sorted Vec for O(log n) binary search
6. Implement confidence scoring heuristics
7. Add caching for repeated lookups

**Estimated Effort**: 2-3 days

**Dependencies**: DWARF parser completion

### Priority 3: Ruchy Analyzer (DWASM-005)

**BLOCKED: Ruchy WASM Compiler Broken**

**Cannot proceed until ruchy project fixes**:
- Issue #27: WASM compiler 100% failure rate
- Issue #26: Turbofish syntax in lambda blocks

**When Unblocked**:
1. Implement Ruchy-specific boundary detection
2. Analyze actor model message passing
3. Detect potential deadlocks in actor systems
4. Track pattern matching exhaustiveness
5. Test with all ruchy examples

**Estimated Effort**: 3-4 days (after ruchy fixes)

## Critical Issues Found in Ruchy Project

### Issue #27: WASM Compiler 100% Failure Rate [CRITICAL]

**Impact**: Complete blocker for ruchy → WASM compilation

**Root Causes**:
1. **Stack Management Broken**: Expression results not dropped, causing stack overflow
2. **Type Inference Failures**: i32/f32 mixing causes type mismatches
3. **Control Flow Issues**: if/else blocks have stack underflow

**Example Failure**:
```ruchy
// This fails:
2 + 2
10 * 5

// Error: values remaining on stack at end of block
```

**PMAT Role**:
- Can analyze WASM when valid modules exist
- Successfully validates trivial WASM (0 vulnerabilities found)
- Cannot help until ruchy compiler generates valid WASM

**Recommendation for ruchy**:
1. Fix stack management (add `drop` instructions)
2. Fix type propagation (ruchy types → WASM types)
3. Add internal WASM validation before output
4. Create WASM compilation test suite

### Issue #26: Turbofish Syntax Parser Bug

**Impact**: Generic types fail in lambda blocks

**Example**:
```ruchy
test("demo", || {
    "42".parse::<i32>();  // ❌ Expected RightBrace error
    true
});
```

**Works at top level**:
```ruchy
let x = "42".parse::<i32>();  // ✅ No error
```

**Root Cause**: Context-sensitive parser rules inconsistent

**Impact on PMAT**: Cannot analyze ruchy code with generics in closures

## Phase 2 Timeline

**Week 1-2**: DWARF Parser Implementation
- Day 1-2: gimli API study and prototyping
- Day 3-4: DIE extraction and function name parsing
- Day 5-6: Line program parsing
- Day 7: Testing and fuzzing

**Week 3**: Correlation Engine
- Day 8-9: DWARF → WASM offset mapping
- Day 10: Source map fallback integration
- Day 11: Bidirectional lookup implementation
- Day 12: Performance optimization and caching

**Week 4**: Ruchy Analyzer (if unblocked)
- Day 13-14: Actor model analysis
- Day 15: Deadlock detection
- Day 16: Pattern matching analysis
- Day 17: Integration testing

**Week 5**: Integration & Testing
- Day 18-19: End-to-end pipeline testing
- Day 20: Documentation and examples
- Day 21: Performance benchmarking

**Total: 3-5 weeks (depending on ruchy fixes)**

## Success Metrics

**DWARF Parser**:
- ✅ Parse rustc-generated DWARF successfully
- ✅ Parse clang-generated DWARF successfully
- ✅ Extract 100% of function names
- ✅ Map source locations accurately
- ✅ Pass fuzzing tests (malformed DWARF)

**Correlation Engine**:
- ✅ O(log n) lookup time for 10,000+ mappings
- ✅ >95% confidence for DWARF-based mappings
- ✅ >75% confidence for source map fallbacks
- ✅ Zero false positives in bidirectional lookup

**Ruchy Analyzer** (when unblocked):
- ✅ Detect all actor message patterns
- ✅ Identify potential deadlocks
- ✅ Verify pattern match exhaustiveness
- ✅ Test with 100+ ruchy examples

## Deliverables

**Phase 2 Complete When**:
1. DWARF parser extracts all debug information
2. Correlation engine provides bidirectional source ↔ WASM mapping
3. Ruchy analyzer detects actor patterns (pending compiler fix)
4. All components pass Toyota Way quality gates
5. Documentation updated with Phase 2 features
6. End-to-end pipeline tested on real-world projects

## Next Steps After Phase 2

**Phase 3 (Weeks 6-8)**:
- Execution tracing with WASM runtime integration
- Performance profiling and hotspot detection
- Chrome DevTools integration for debugging
- Ruchy deadlock detection with actor tracing

---

**Document Version**: 1.0
**Last Updated**: 2025-10-03
**Status**: Phase 1 complete, Phase 2 ready to start
