# WASM

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 17

## WebAssembly Analysis

### Bytecode Parsing

Static analysis of WASM binaries:
- Section parsing (type, function, memory, export)
- Instruction counting and categorization
- Memory usage estimation

### Resource Metrics

| Metric | Description |
|--------|-------------|
| Function count | Total WASM functions |
| Memory pages | Initial/max memory allocation |
| Import count | External dependencies |
| Export count | Public API surface |
| Instruction count | Total opcodes |

## Deep WASM Inspection

### Control Flow Analysis

- Basic block identification
- Loop detection and nesting depth
- Branch complexity measurement
- Unreachable code detection

### Phase 2 Plan

- Call graph construction across WASM modules
- Type-level analysis for interface validation
- Performance profiling instrumentation

## Quality Assurance

### WASM Integration Tests

```bash
cargo test --features wasm -- wasm_
```

- Validates bytecode parsing correctness
- Tests resource metric accuracy
- Verifies export/import resolution

## Presentar Pure-WASM Conversion

Convert demo visualizations from JavaScript to pure WASM:
- Presentar-core TUI framework
- No JavaScript dependencies
- Smaller bundle size

## Key Files

| File | Purpose |
|------|---------|
| `src/services/wasm/` | WASM bytecode analysis (module) |
| `src/services/wasm/binary.rs` | WASM binary format parsing |

## References

- Consolidated from: wasm-analysis-spec, wasm-quality-assurance,
  deep-wasm, deep-wasm-phase2-plan, convert-demo-vis-to-presentar-pure-WASM
