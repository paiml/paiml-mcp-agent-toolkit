# Current Metric Failures

This document tracks known issues with metric accuracy in the paiml-mcp-agent-toolkit.

## Uniform TDG Values

Files with suspiciously similar TDG values (2.43-2.45 range):

```bash
# Command to reproduce:
pmat analyze comprehensive server/ --json | \
  jq '.files[] | select(.tdg >= 2.43 and .tdg <= 2.45) | {path: .path, tdg: .tdg}'
```

### Root Cause
The current TDG calculation uses a simplistic formula that essentially multiplies LOC by a constant factor, resulting in uniform values across different files regardless of their actual complexity.

## Impossible Cognitive/Cyclomatic Ratios

Functions with ratios outside the expected 1.1-2.0 range:

```bash
# Command to find violations:
pmat analyze comprehensive server/ --json | \
  jq '.functions[] | 
  select(.cognitive > .cyclomatic * 2 or .cognitive < .cyclomatic * 1.1) | 
  {name: .name, cognitive: .cognitive, cyclomatic: .cyclomatic, ratio: (.cognitive / .cyclomatic)}'
```

### Examples Found
- Function with cognitive=96, cyclomatic=60 (ratio=1.6) - within bounds but suspiciously high
- Functions where cognitive < cyclomatic (impossible by definition)

## Dead Code False Positives

FFI-exported functions incorrectly marked as dead:

```bash
# Test case:
echo '#[no_mangle] pub extern "C" fn test() {}' > ffi_test.rs
pmat analyze dead-code ffi_test.rs --json | jq '.dead_functions'
```

### Missing Detection
- `#[no_mangle]` attributes
- `extern "C"` functions
- `#[export_name]` attributes
- WASM bindgen functions
- PyO3 exports

## AST Parser Complexity

Current complexity scores:
- `ast_cpp.rs`: CC=260 (56-function match expression)
- `ast_c.rs`: CC=190 (40-function match expression)
- `ast_typescript.rs`: CC=108

These parsers contain massive match expressions that should be refactored into dispatch tables.

## CLI Module Size

`server/src/cli/mod.rs`:
- 273 symbols
- 146 functions
- TDG 3.09
- 2000+ LOC in single file

This violates the single responsibility principle and makes the code difficult to maintain.