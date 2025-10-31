# Bug Report: Dead Code Analysis Requires Cargo.toml for Non-Rust Projects

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: High
**Component**: Dead code analyzer

## Description

When analyzing dead code in a C project (or any non-Rust project), the analyzer errors because it cannot find `Cargo.toml`. The dead code analyzer should detect the project language and use appropriate tooling, not assume all projects are Rust.

## Steps to Reproduce

```bash
cd /path/to/c-project  # e.g., cpython
pmat analyze dead-code --path .
```

## Actual Output

```
💀 Analyzing dead code in project...
⏰ Analysis timeout set to 60 seconds
2025-10-29T19:00:51.582103Z ERROR Error: Cargo check failed: error: could not find `Cargo.toml` in `/home/alfredo/code/cpython/Python` or any parent directory
```

## Expected Behavior

The dead code analyzer should:
1. Detect the project language (C, C++, Python, etc.)
2. Use appropriate dead code detection tooling for that language
3. For C/C++ projects, use tools like `clang` with `-Wunused` or similar
4. For Python projects, use tools like `vulture` or similar
5. Only use `cargo check` for Rust projects

## Analysis

- Dead code analyzer is hardcoded to use `cargo check`
- No language detection or multi-language support
- Breaks PMAT's promise of multi-language support

## Impact

- **CRITICAL**: Dead code analysis completely broken for non-Rust projects
- Limits PMAT usefulness to Rust-only codebases
- Contradicts documentation about multi-language support

## Suggested Fix

1. Add language detection to dead code analyzer
2. Create language-specific dead code detection strategies:
   - Rust: `cargo check` with unused warnings
   - C/C++: Clang static analysis or `cppcheck`
   - Python: `vulture` or AST-based analysis
   - JavaScript/TypeScript: ESLint unused-vars or similar
3. Provide clear error when language not supported

## Files to Investigate

- `server/src/cli/handlers/analyze.rs` - Dead code handler
- `server/src/services/dead_code.rs` or similar - Dead code analyzer implementation
- Language detection logic

## Test Case

```rust
#[test]
fn test_dead_code_analysis_c_project() {
    let result = analyze_dead_code(Path::new("./fixtures/c-project"));
    assert!(result.is_ok());
    assert!(!result.unwrap().contains("Cargo.toml"));
}
```
