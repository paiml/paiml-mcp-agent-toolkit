# Language Support

> Sub-spec of [pmat-spec.md](../pmat-spec.md) | Component 4

## Supported Languages

### Tier 1 (Full Analysis)

| Language | Extensions | AST Parser | Complexity | TDG |
|----------|-----------|------------|------------|-----|
| Rust | .rs | tree-sitter-rust, syn | Yes | Yes |
| Python | .py | tree-sitter-python | Yes | Yes |
| TypeScript | .ts, .tsx | tree-sitter-typescript | Yes | Yes |
| JavaScript | .js, .jsx | tree-sitter-javascript | Yes | Yes |

### Tier 2 (Partial Analysis)

| Language | Extensions | AST Parser | Complexity | TDG |
|----------|-----------|------------|------------|-----|
| Go | .go | tree-sitter-go | Yes | Limited |
| C/C++ | .c, .h, .cpp, .cc, .hpp | tree-sitter-c/cpp | Yes | Limited |
| CUDA | .cu, .cuh | tree-sitter-cuda | Yes | Limited |
| Java | .java | tree-sitter-java | Yes | Limited |
| Kotlin | .kt, .kts | tree-sitter-kotlin | Yes | Limited |
| C# | .cs | tree-sitter-c-sharp | Yes | Limited |

### Tier 3 (Basic Support)

| Language | Extensions | Support Level |
|----------|-----------|---------------|
| Shell (bash/zsh) | .sh, .bash, .zsh | Linting via bashrs, security analysis |
| R | .R, .Rmd | Function extraction, basic metrics |
| Julia | .jl | Function extraction, basic metrics |
| Haskell | .hs | Function extraction, type analysis |
| Erlang/Elixir | .erl, .ex | Module detection, basic metrics |
| Ruchy | .rcy | Full AST via ruchy-ast feature gate |
| Lean 4 | .lean | Formal verification support |

## Analysis Capabilities

### Per-Language Features

| Feature | Rust | Python | TS/JS | Go | C++ |
|---------|------|--------|-------|-----|-----|
| Cyclomatic complexity | Yes | Yes | Yes | Yes | Yes |
| Cognitive complexity | Yes | Yes | Yes | Yes | Yes |
| Function extraction | Yes | Yes | Yes | Yes | Yes |
| Call graph | Yes | Yes | Yes | Yes | Yes |
| Import graph | Yes | Yes | Yes | Yes | Yes |
| Dead code detection | Yes | Yes | Yes | No | No |
| Duplicate detection | Yes | Yes | Yes | No | Yes |
| Big-O analysis | Yes | Yes | Yes | No | No |

### CUDA/SIMD-Specific

- Kernel launch complexity (grid/block dimensions)
- Shared memory usage tracking
- Warp divergence detection
- PTX flow analysis for cross-project search

### Known Defects Database

Language-specific bug patterns:
- Rust: `unwrap()` in non-test code, `unsafe` blocks, `panic!` in libraries
- Python: bare `except:`, mutable default args, `eval()`
- TypeScript: `any` type usage, missing null checks
- C++: raw pointer arithmetic, missing virtual destructors

## Duplicate Detection Languages

```rust
match ext {
    "rs" => Some(Language::Rust),
    "ts" | "tsx" => Some(Language::TypeScript),
    "js" | "jsx" => Some(Language::JavaScript),
    "py" => Some(Language::Python),
    "c" | "h" => Some(Language::C),
    "cpp" | "cc" | "cxx" | "hpp" | "hxx" | "cu" | "cuh" => Some(Language::Cpp),
    "kt" | "kts" => Some(Language::Kotlin),
    _ => None,
}
```

## Key Files

| File | Purpose |
|------|---------|
| `src/services/language_analyzer.rs` | Multi-language complexity analysis |
| `src/services/duplicate_detector.rs` | Cross-language clone detection |
| `src/services/context.rs` | AST parsing per language |

## References

- Consolidated from: go-language-support, jvm-clr-language-support,
  functional-scientific-language-support, shell-support-spec, enhanced-ruchy-support,
  first-class-ruchy-spec, known-defects-languages-spec, lean-and-provable-contracts,
  improved-cpp-pmat-query, cuda-simd-tdg, improve-language-mlops-support
