# PMAT Deep WASM - Verification & Usage Guide

## ✅ Verification Status

**Date**: 2025-10-03
**Version**: v2.110.0
**Feature**: `deep-wasm` (optional, feature-gated)

### Build Verification

```bash
# Build with deep-wasm feature (REQUIRED)
cargo build --features deep-wasm

# Verify command exists
target/debug/pmat analyze --help | grep "deep-wasm"
```

**Expected Output**:
```
  deep-wasm             Deep WASM pipeline inspection (Rust/Ruchy → WASM → JS)
```

### Test Verification

Successfully tested on minimal Rust file:

```rust
// /tmp/test_wasm.rs
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[no_mangle]
pub extern "C" fn multiply(x: i32, y: i32) -> i32 {
    x * y
}
```

**Command**:
```bash
pmat analyze deep-wasm -p /tmp/test_wasm.rs --language rust --focus source
```

**Output** (verified working):
- ✅ Source metrics: 9 LOC, 2 functions, complexity 3
- ✅ Pipeline overview generated
- ✅ Quality gates executed (reported missing WASM/source map as expected)
- ✅ Markdown report generated successfully

## How to Run PMAT Deep WASM

### 1. Enable the Feature

The `deep-wasm` feature is **opt-in** to minimize dependencies for users who don't need WASM analysis.

```bash
# Debug build (faster compilation, slower execution)
cargo build --features deep-wasm

# Release build (slower compilation, faster execution)
cargo build --release --features deep-wasm
```

### 2. Basic Usage

#### Source Code Analysis Only

```bash
pmat analyze deep-wasm -p src/lib.rs --language rust --focus source
```

This analyzes:
- Source code metrics (LOC, functions, complexity)
- WASM boundary functions (`#[no_mangle]`, `extern "C"`, `#[wasm_bindgen]`)
- Memory patterns (Box, Vec, String, RawPointer)

#### With WASM Binary

```bash
pmat analyze deep-wasm \
  -p src/lib.rs \
  --wasm-file target/wasm32-unknown-unknown/release/your_project.wasm \
  --language rust
```

This adds:
- WASM module analysis (size, functions, exports)
- Binary structure inspection
- Import/export mapping

#### Full Pipeline Analysis

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

This provides complete analysis including:
- Source-to-WASM correlation (requires DWARF)
- JavaScript source mapping
- Quality gate enforcement
- Comprehensive report generation

### 3. Focus Options

Target specific areas of the pipeline:

| Focus | Description | Use Case |
|-------|-------------|----------|
| `source` | Source code only | Quick analysis without WASM |
| `compilation` | Compilation pipeline | Rust/Ruchy → WASM transformation |
| `runtime` | Runtime behavior | WASM execution patterns |
| `interop` | JavaScript interop | WASM ↔ JS boundary analysis |
| `full` | Complete analysis (default) | Comprehensive pipeline inspection |

**Examples**:

```bash
# Source code patterns only
pmat analyze deep-wasm -p src/ --focus source

# Compilation pipeline tracking
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --focus compilation

# Runtime behavior analysis
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --focus runtime

# JavaScript interop inspection
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --source-map app.map --focus interop

# Full pipeline (all of the above)
pmat analyze deep-wasm -p src/ --wasm-file app.wasm --focus full
```

### 4. Quality Gates

Enable strict Toyota Way quality enforcement:

```bash
pmat analyze deep-wasm \
  -p src/ \
  --wasm-file app.wasm \
  --strict
```

**Default Mode**:
- Max module size: 10MB
- Max WASM complexity: 20
- Min source map coverage: 95%

**Strict Mode** (`--strict`):
- Max module size: 5MB
- Max WASM complexity: 15
- Min source map coverage: 99%

**Zero-Tolerance Checks** (both modes):
- ❌ Unreachable code
- ❌ Unbounded loops
- ❌ Stack overflow risks
- ❌ Memory leaks
- ❌ Undefined behavior
- ❌ Type unsafety

### 5. Advanced Features

#### Rust-Specific Analysis

```bash
pmat analyze deep-wasm \
  -p src/lib.rs \
  --language rust \
  --include-mir \
  --include-llvm-ir \
  --track-memory
```

**Flags**:
- `--include-mir`: Include MIR (Mid-level Intermediate Representation)
- `--include-llvm-ir`: Include LLVM IR analysis
- `--track-memory`: Track memory layout and patterns

#### Ruchy-Specific Analysis

```bash
pmat analyze deep-wasm \
  -p src/app.ruchy \
  --language ruchy \
  --detect-deadlocks \
  --track-memory
```

**Flags**:
- `--detect-deadlocks`: Detect actor system deadlocks
- `--track-memory`: Track concurrent memory access

#### Output Formats

```bash
# Markdown (default, human-readable)
pmat analyze deep-wasm -p src/ --format markdown -o report.md

# JSON (machine-readable, CI/CD integration)
pmat analyze deep-wasm -p src/ --format json -o report.json

# HTML (interactive, browser-based)
pmat analyze deep-wasm -p src/ --format html -o report.html
```

### 6. Real-World Example

Complete workflow for Rust → WASM project:

```bash
# 1. Build your WASM target
cargo build --target wasm32-unknown-unknown --release

# 2. Run deep WASM analysis
pmat analyze deep-wasm \
  -p src/lib.rs \
  --wasm-file target/wasm32-unknown-unknown/release/your_project.wasm \
  --language rust \
  --focus full \
  --strict \
  --track-memory \
  --output wasm_pipeline_analysis.md

# 3. Review the report
cat wasm_pipeline_analysis.md
```

### 7. CI/CD Integration

#### GitHub Actions

```yaml
- name: WASM Pipeline Analysis
  run: |
    cargo build --release --features deep-wasm
    cargo build --target wasm32-unknown-unknown --release
    ./target/release/pmat analyze deep-wasm \
      -p src/lib.rs \
      --wasm-file target/wasm32-unknown-unknown/release/app.wasm \
      --strict \
      --format json \
      --output wasm_analysis.json

    # Fail if quality gates violated
    if grep -q "violations detected" wasm_analysis.json; then
      echo "Quality gate violations found!"
      exit 1
    fi
```

#### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

if [[ $(git diff --cached --name-only | grep -E '\.(rs|ruchy)$') ]]; then
  echo "Running WASM analysis on changed files..."

  cargo build --features deep-wasm

  for file in $(git diff --cached --name-only | grep -E '\.(rs|ruchy)$'); do
    ./target/debug/pmat analyze deep-wasm -p "$file" --focus source --strict

    if [ $? -ne 0 ]; then
      echo "WASM analysis failed for $file"
      exit 1
    fi
  done
fi
```

### 8. MCP Integration

Use deep-wasm tools in AI agents via Model Context Protocol:

```json
{
  "name": "deep_wasm_analyze",
  "arguments": {
    "source_path": "src/lib.rs",
    "wasm_path": "app.wasm",
    "language": "rust",
    "focus": "full",
    "strict": true
  }
}
```

**Available MCP Tools**:
1. `deep_wasm_analyze` - Full analysis
2. `deep_wasm_query_mapping` - Source-to-WASM mapping
3. `deep_wasm_trace_execution` - Execution tracing
4. `deep_wasm_compare_optimizations` - Optimization comparison
5. `deep_wasm_detect_issues` - Issue detection

### 9. Troubleshooting

#### Command Not Found

**Problem**: `deep-wasm` subcommand not available

**Solution**: Rebuild with feature flag:
```bash
cargo build --features deep-wasm
./target/debug/pmat analyze deep-wasm --help
```

The deep-wasm feature is **opt-in** and not included in default builds.

#### WASM File Not Found

**Problem**: `WASM file not found: app.wasm`

**Solution**: Use absolute path or verify file exists:
```bash
# Absolute path
pmat analyze deep-wasm -p src/ --wasm-file $(pwd)/target/wasm32-unknown-unknown/release/app.wasm

# Verify file exists first
ls -lh target/wasm32-unknown-unknown/release/*.wasm
```

#### Source Language Detection Failed

**Problem**: Cannot auto-detect language from extension

**Solution**: Explicitly specify language:
```bash
pmat analyze deep-wasm -p src/lib.rs --language rust
# or
pmat analyze deep-wasm -p src/app.ruchy --language ruchy
```

#### Quality Gate Violations

**Problem**: Analysis fails with quality gate violations

**Solutions**:

1. **Disable strict mode** (use defaults):
   ```bash
   pmat analyze deep-wasm -p src/ --wasm-file app.wasm
   # (remove --strict flag)
   ```

2. **Provide missing files**:
   ```bash
   # Generate source map
   wasm-sourcemap app.wasm -o app.map

   # Include in analysis
   pmat analyze deep-wasm -p src/ --wasm-file app.wasm --source-map app.map
   ```

3. **Fix source issues** - Review violations and address:
   - Reduce module size (split into smaller modules)
   - Reduce WASM complexity (simplify control flow)
   - Add source maps for traceability

## Documentation References

- **Specification**: `docs/specifications/deep-wasm.md`
- **Usage Guide**: `docs/deep-wasm-usage.md` (480+ lines)
- **CLI Integration**: `server/src/cli/handlers/deep_wasm_handlers.rs`
- **MCP Tools**: `server/src/mcp_integration/deep_wasm_tools.rs`
- **Service**: `server/src/services/deep_wasm/`

## Version History

- **v2.110.0** (2025-10-03): Phase 1 complete - WASM parsing, DWARF framework, quality gates, CLI, MCP tools
- **Future (Phase 2)**: Source-to-WASM correlation, type flow analysis
- **Future (Phase 3)**: Execution tracing, performance profiling, Chrome DevTools integration
