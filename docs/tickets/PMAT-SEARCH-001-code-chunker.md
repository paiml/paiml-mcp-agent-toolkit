# PMAT-SEARCH-001: AST-Aware Code Chunker

**Sprint**: 29
**Status**: 🔴 RED PHASE
**Estimated**: 3 hours
**Actual**: TBD

## 🎯 Objective

Implement intelligent code chunking using PMAT's existing AST parsers to extract semantic units (functions, classes, modules) for embedding generation.

## 📋 Requirements

**Must Support:**
- Rust: functions, impl blocks, modules
- TypeScript: functions, classes, interfaces
- Python: functions, classes
- C/C++: functions, classes
- Go: functions, structs

**Chunk Metadata:**
- `file_path`: Full path
- `chunk_type`: "function" | "class" | "module" | "file"
- `chunk_name`: Identifier
- `language`: Language name
- `start_line`, `end_line`: Location
- `content`: Full source with docstrings
- `content_checksum`: SHA256 for incremental updates

## 🔴 RED Phase: Tests First

### Test Suite

```rust
// tests/unit_code_chunker.rs

#[test]
fn test_chunk_rust_functions() {
    let source = r#"
        /// Calculate sum
        fn add(a: i32, b: i32) -> i32 { a + b }

        /// Calculate product
        fn multiply(a: i32, b: i32) -> i32 { a * b }
    "#;

    let chunks = chunk_code(source, Language::Rust)?;
    assert_eq!(chunks.len(), 2);
    assert_eq!(chunks[0].chunk_type, ChunkType::Function);
    assert_eq!(chunks[0].chunk_name, "add");
    assert!(chunks[0].content.contains("Calculate sum"));
}

#[test]
fn test_chunk_typescript_class() {
    let source = r#"
        class Calculator {
            add(a: number, b: number): number { return a + b; }
        }
    "#;

    let chunks = chunk_code(source, Language::TypeScript)?;
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].chunk_type, ChunkType::Class);
    assert_eq!(chunks[0].chunk_name, "Calculator");
}

#[test]
fn test_chunk_checksum() {
    let source = "fn foo() { }";
    let chunks = chunk_code(source, Language::Rust)?;
    let checksum1 = &chunks[0].content_checksum;

    let chunks2 = chunk_code(source, Language::Rust)?;
    let checksum2 = &chunks2[0].content_checksum;

    assert_eq!(checksum1, checksum2); // Deterministic
}
```

**Total Tests**: 20
- Per language: Rust, TypeScript, Python, C/C++, Go (5 tests)
- Edge cases: Empty file, large file, nested structures (5 tests)

## 🟢 GREEN Phase: Implementation

**File**: `server/src/services/semantic/chunker.rs`

**Key Functions:**
- `chunk_code(source: &str, language: Language) -> Result<Vec<CodeChunk>>`
- `chunk_rust_file(source: &str) -> Vec<CodeChunk>`
- `chunk_typescript_file(source: &str) -> Vec<CodeChunk>`
- `compute_checksum(content: &str) -> String`

**Leverage Existing:**
- Use PMAT's existing AST parsers (already have tree-sitter!)
- Reuse language detection logic
- Use existing unified AST types

## 🔵 REFACTOR Phase: Quality

**Complexity Target**: ≤10 cyclomatic per function
**Coverage Target**: ≥95%
**SATD**: 0 violations

## ✅ Exit Criteria

- [ ] 20 tests passing
- [ ] Supports top 5 languages
- [ ] Checksums are deterministic
- [ ] Includes docstrings in chunks
- [ ] Cyclomatic ≤10 for all functions
- [ ] Zero clippy warnings
