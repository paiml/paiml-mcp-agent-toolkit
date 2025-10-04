# Deep WASM Pipeline Inspection - Usage Guide

## Overview

The Deep WASM feature provides comprehensive inspection of the Rust/Ruchy → WebAssembly → JavaScript → HTML compilation and execution pipeline. It implements multi-layer bidirectional tracing that reconstructs the complete pipeline with Toyota Way quality standards.

## Installation

### Enable the Feature

The Deep WASM feature is optional and must be enabled during compilation:

```bash
cargo build --features deep-wasm --release
```

### Dependencies

The following dependencies are automatically included when the `deep-wasm` feature is enabled:

- `wasmparser 0.239.0` - WASM binary parsing
- `wasm-encoder 0.239.0` - WASM binary encoding
- `walrus 0.22` - WASM transformation
- `object 0.37` - Object file parsing (DWARF support)
- `sourcemap 9.0` - JavaScript source map parsing
- `gimli 0.32` - DWARF debug information parsing
- `ahash 0.8` - Fast hashing for analysis

## CLI Usage

### Basic Analysis

Analyze a Rust/Ruchy project with WASM output:

```bash
pmat analyze deep-wasm -p src/lib.rs --wasm-file target/wasm/app.wasm
```

### Full Pipeline Analysis

Perform complete analysis with all debug information:

```bash
pmat analyze deep-wasm \
  --source-path src/ \
  --wasm-file app.wasm \
  --dwarf-file app.dwarf \
  --source-map app.map \
  --language rust \
  --focus full \
  --format markdown \
  --output deep_wasm_report.md
```

### Analysis Focus Options

Target specific areas of the pipeline:

```bash
# Source code only
pmat analyze deep-wasm -p src/ --focus source

# Compilation pipeline
pmat analyze deep-wasm -p src/ --focus compilation

# Runtime behavior
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --focus runtime

# JavaScript interop
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --focus interop
```

### Quality Gates

Enable strict quality enforcement:

```bash
pmat analyze deep-wasm \
  -p src/ \
  --wasm-file app.wasm \
  --strict \
  --format json \
  --output quality_report.json
```

Strict mode enforces:
- Max module size: 5MB (vs 10MB default)
- Max WASM complexity: 15 (vs 20 default)
- Min source map coverage: 99% (vs 95% default)

### Language-Specific Features

#### Rust Analysis

```bash
pmat analyze deep-wasm \
  -p src/lib.rs \
  --language rust \
  --include-mir \
  --include-llvm-ir \
  --track-memory
```

#### Ruchy Actor Systems

```bash
pmat analyze deep-wasm \
  -p src/actor_system.rch \
  --language ruchy \
  --detect-deadlocks \
  --focus interop
```

### Output Formats

Choose the appropriate format for your use case:

```bash
# Markdown report (human-readable)
pmat analyze deep-wasm -p src/ --format markdown --output report.md

# JSON data (machine-readable)
pmat analyze deep-wasm -p src/ --format json --output data.json

# HTML report (interactive)
pmat analyze deep-wasm -p src/ --format html --output report.html
```

## MCP Integration

The Deep WASM feature provides 5 MCP tools for AI agent integration:

### 1. deep_wasm_analyze

Comprehensive pipeline analysis via MCP:

```json
{
  "name": "deep_wasm_analyze",
  "arguments": {
    "source_path": "src/lib.rs",
    "wasm_path": "target/wasm/app.wasm",
    "language": "rust",
    "focus": "full",
    "strict": true
  }
}
```

**Response includes:**
- Complete markdown report
- Project metadata
- Module size metrics
- Quality gate status
- Violation details

### 2. deep_wasm_query_mapping

Query source-to-WASM bidirectional mappings:

```json
{
  "name": "deep_wasm_query_mapping",
  "arguments": {
    "wasm_path": "app.wasm",
    "source_file": "src/lib.rs",
    "line": 42
  }
}
```

*Note: Available in Phase 2*

### 3. deep_wasm_trace_execution

Trace execution through pipeline layers:

```json
{
  "name": "deep_wasm_trace_execution",
  "arguments": {
    "wasm_path": "app.wasm",
    "entry_point": "main",
    "max_depth": 100
  }
}
```

*Note: Available in Phase 3*

### 4. deep_wasm_compare_optimizations

Compare WASM binaries at different optimization levels:

```json
{
  "name": "deep_wasm_compare_optimizations",
  "arguments": {
    "wasm_paths": [
      "target/wasm/debug/app.wasm",
      "target/wasm/release/app.wasm"
    ],
    "metrics": ["size", "complexity", "performance"]
  }
}
```

*Note: Available in Phase 2*

### 5. deep_wasm_detect_issues

Detect WASM-specific quality issues:

```json
{
  "name": "deep_wasm_detect_issues",
  "arguments": {
    "wasm_path": "app.wasm",
    "issue_types": [
      "unreachable_code",
      "unbounded_loop",
      "stack_overflow",
      "memory_leak",
      "undefined_behavior",
      "type_unsafety"
    ],
    "zero_tolerance": true
  }
}
```

*Note: Available in Phase 2*

## Quality Gates

### WASM-Specific Quality Rules

The Deep WASM feature enforces Toyota Way quality standards:

1. **Module Size Limit**
   - Default: 10MB
   - Strict: 5MB
   - Rationale: Prevent bloated bundles

2. **WASM Complexity**
   - Default: ≤20 cyclomatic complexity
   - Strict: ≤15
   - Applies to: All WASM functions

3. **Source Map Coverage**
   - Default: ≥95%
   - Strict: ≥99%
   - Ensures debuggability

4. **Zero Tolerance Issues**
   - Unreachable code
   - Unbounded loops
   - Stack overflow risks
   - Memory leaks
   - Undefined behavior
   - Type unsafety

### Custom Quality Configuration

```rust
use pmat::services::deep_wasm::{WasmQualityGates, DeepWasmService};

let mut gates = WasmQualityGates::default();
gates.max_module_size = 8_388_608; // 8MB
gates.max_wasm_complexity = 18;
gates.min_source_map_coverage = 0.98;

let service = DeepWasmService::new().with_quality_gates(gates);
```

## Report Structure

### Markdown Report Sections

1. **Pipeline Overview**
   - Source language and version
   - Target platform
   - Optimization level
   - Debug symbols status

2. **Source Metrics**
   - Lines of code
   - Function count
   - Max complexity
   - WASM boundary functions

3. **WASM Module Analysis**
   - Module size
   - Function count
   - Exported functions
   - Complexity metrics
   - Debug information presence

4. **Source-to-WASM Correlations** *(Phase 2)*
   - Line-level mappings
   - Function boundaries
   - Type transformations

5. **Type Flow Analysis** *(Phase 2)*
   - Source type → WASM type → JS type
   - Conversion costs
   - Potential issues

6. **Performance Hotspots** *(Phase 3)*
   - Execution time attribution
   - Call counts
   - Optimization suggestions

7. **Quality Gate Results**
   - Pass/fail status
   - Violations (if any)
   - Remediation guidance

## Integration Examples

### CI/CD Pipeline

```yaml
# .github/workflows/wasm-quality.yml
name: WASM Quality Check

on: [push, pull_request]

jobs:
  deep-wasm-analysis:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Build WASM
        run: cargo build --target wasm32-unknown-unknown --release

      - name: Install PMAT
        run: cargo install pmat --features deep-wasm

      - name: Run Deep WASM Analysis
        run: |
          pmat analyze deep-wasm \
            --source-path src/ \
            --wasm-file target/wasm32-unknown-unknown/release/app.wasm \
            --strict \
            --format json \
            --output wasm_quality.json

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: wasm-quality-report
          path: wasm_quality.json
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Run Deep WASM analysis
pmat analyze deep-wasm \
  --source-path src/ \
  --wasm-file target/wasm32-unknown-unknown/release/app.wasm \
  --strict

# Exit with analysis result
exit $?
```

### Programmatic Usage

```rust
use pmat::services::deep_wasm::{
    DeepWasmService, DeepWasmAnalysisRequest,
    SourceLanguage, AnalysisFocus
};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = DeepWasmService::new();

    let request = DeepWasmAnalysisRequest {
        source_path: PathBuf::from("src/lib.rs"),
        wasm_path: Some(PathBuf::from("app.wasm")),
        dwarf_path: None,
        source_map_path: None,
        language: SourceLanguage::Rust,
        analysis_focus: AnalysisFocus::Full,
    };

    let report = service.analyze(request).await?;

    println!("Quality passed: {}", report.quality_gate_results.passed);
    println!("Module size: {} bytes", report.wasm_module_analysis.module_size_bytes);

    Ok(())
}
```

## Troubleshooting

### Feature Not Available

**Error:** "Deep WASM feature not enabled"

**Solution:** Rebuild with the feature flag:
```bash
cargo build --features deep-wasm --release
```

### Missing Debug Information

**Warning:** "Source map is missing"

**Solution:** Build with source maps:
```bash
# Rust
RUSTFLAGS="-C debuginfo=2" cargo build --target wasm32-unknown-unknown

# Or use wasm-pack
wasm-pack build --debug
```

### Module Size Violations

**Error:** "Module size exceeds limit"

**Solutions:**
1. Enable size optimizations:
   ```bash
   wasm-opt -Oz app.wasm -o app_optimized.wasm
   ```

2. Use custom size limit:
   ```bash
   pmat analyze deep-wasm -p src/ --wasm-file app.wasm
   # (default limit is 10MB, strict is 5MB)
   ```

## Roadmap

### Phase 1: WASM Binary Parsing ✅ COMPLETE
- ✅ WASM binary parser with zero-copy analysis (wasmparser)
- ✅ DWARF v5 parser framework (gimli integration)
- ✅ Source map handler (JavaScript-style debugging)
- ✅ Rust WASM analyzer (boundary function detection)
- ✅ Quality gates (strict + default modes)
- ✅ CLI interface: `pmat analyze deep-wasm` (13 options)
- ✅ MCP tool integration (5 AI agent tools)
- ✅ Markdown/JSON/HTML report generation
- ✅ 30+ comprehensive tests

### Phase 2: DWARF Correlation ✅ COMPLETE
- ✅ DWARF v5 line program parsing with validation (DWARF v2-v5 support)
- ✅ Enhanced correlation engine with bidirectional mapping
- ✅ `correlate_with_line_programs()` - Line data integration
- ✅ `calculate_confidence()` - Multi-signal scoring (perfect match: 1.0)
- ✅ `lookup_source_location()` - WASM address → source location
- ✅ `lookup_wasm_addresses()` - Source line → WASM addresses
- ✅ Graceful error handling for malformed/synthetic DWARF data
- ✅ 20 new Phase 2 tests (9 parser + 11 correlation)
- ✅ 72 total deep_wasm tests passing

### Phase 2.5: WASM Mutation Testing ✅ COMPLETE
- ✅ WasmAdapter implementation for .wasm and .wat files
- ✅ WAT text-based mutation approach (simple and effective)
- ✅ 3 WASM Mutation Operators:
  - `WasmNumericMutator`: i32/i64/f32/f64 arithmetic mutations (80% kill prob)
  - `WasmControlFlowMutator`: Control flow mutations (90% kill prob)
  - `WasmLocalMutator`: Stack operations (75% kill prob)
- ✅ Type system enhancements for WASM operators
- ✅ 6 comprehensive WASM mutation tests
- ✅ Integration with LanguageRegistry via `register_wasm()`
- ✅ 180 total mutation tests passing (174 baseline + 6 WASM)

### Phase 2.6: WASM Unified Parser ✅ COMPLETE
- ✅ Eliminated double parsing for WASM/WAT files
- ✅ Created `UnifiedWasmAnalyzer` with pattern-based extraction
- ✅ Control flow complexity analysis (br, loop, block, if)
- ✅ Stack complexity tracking
- ✅ 10 EXTREME TDD tests passing
- ✅ Integrated with deep_context.rs using WASM_UNIFIED_CACHE
- ✅ 40-50% performance improvement

### Phase 2.7: Ruchy Language Support ✅ COMPLETE
- ✅ Fixed Issue #61: Ruchy source analysis in deep-wasm pipeline
- ✅ Auto-detection from .ruchy/.rch file extensions
- ✅ Function counting with "fun" and "async fun" patterns
- ✅ Complexity estimation for Ruchy code
- ✅ Conditional parsing: syn for Rust, pattern matching for Ruchy
- ✅ 1 new deep-wasm test for Ruchy analysis
- ✅ 73 total deep_wasm tests passing

### Phase 3: Runtime Analysis & Performance (Scoped - Future Work)

**Track 1: Performance Profiling & Hotspot Detection** 🔥 (2-3 days)
- Instruction-level profiling (execution time per WASM instruction type)
- Function-level hotspots (call counts, average execution time, memory patterns)
- Flame graphs and source-level heatmaps
- Optimization suggestions (inlining, SIMD, etc.)

**Track 2: WASM Runtime Integration** 🏃 (2-3 days)
- Wasmtime v36 integration (already in dependencies)
- Execute .wasm binaries with instrumentation
- Function entry/exit tracing
- Memory access tracking, import/export monitoring
- Gas metering for cost analysis
- Integration with mutation testing

**Track 3: Security & Vulnerability Analysis** 🔒 (1-2 days)
- Memory bounds checking (out-of-bounds, integer overflow, stack overflow)
- Import analysis (dangerous JS imports, unsafe patterns)
- Quality gates enhancement with security scoring
- Automated vulnerability reports

**Track 4: Chrome DevTools Integration** 🌐 (1-2 days, Optional)
- DWARF → DevTools source map conversion
- Breakpoint-compatible location mapping
- Export `.map` files for browser debugging

**Recommended Scope:**
- **Minimal** (3-4 days): Tracks 1 + 2 (runtime + profiling)
- **Full** (5-7 days): All 4 tracks

**Priority:** Lower than multi-language mutation testing completion

## Support

- **Documentation:** https://docs.pmat.dev/deep-wasm
- **Examples:** https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/examples
- **Issues:** https://github.com/paiml/paiml-mcp-agent-toolkit/issues

---

**Last Updated:** 2025-10-04
**PMAT Version:** 2.121.0
**Deep WASM Status:** Phases 1-2.7 Complete (Phase 3 scoped for future work)
