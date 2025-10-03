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

### Phase 1 (Current) ✅
- ✅ WASM binary parser
- ✅ DWARF v5 parser framework
- ✅ Source map handler
- ✅ Basic quality gates
- ✅ CLI interface
- ✅ MCP tool integration
- ✅ Markdown report generation

### Phase 2 (Weeks 5-8)
- Source-to-WASM correlation engine
- Type flow analysis
- Optimization comparison
- Issue detection
- Enhanced DWARF integration

### Phase 3 (Weeks 9-12)
- Execution tracing
- Performance profiling
- Chrome DevTools integration
- Ruchy deadlock detection
- HTML report generation

## Support

- **Documentation:** https://docs.pmat.dev/deep-wasm
- **Examples:** https://github.com/pmat/examples/deep-wasm
- **Issues:** https://github.com/pmat/issues

---

**Last Updated:** 2025-10-03
**PMAT Version:** 2.109.0
**Deep WASM Version:** 1.0.0 (Phase 1)
