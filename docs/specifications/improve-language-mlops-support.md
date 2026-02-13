# Improved Language and MLOps Model Support Specification

**Version**: 1.0.0
**Status**: Draft
**Authors**: PAIML Team
**Date**: 2026-02-13
**Ticket**: PMAT-500

## Abstract

This specification extends PMAT's language detection, indexing, TDG grading, and compliance checking to cover three new categories of files: (1) structured data languages (SQL, YAML, Markdown), (2) a new programming language (Scala), and (3) ML model binary formats (GGUF, APR, SafeTensors). The model format support enables PMAT to provide quality audits, metadata extraction, and defect detection for ML pipelines—bridging the gap between code quality tooling and MLOps infrastructure across the batuta stack (aprender, realizar, whisper.apr, trueno).

## 1. Motivation

### 1.1 Language Coverage Gaps

PMAT currently supports 7 languages with full AST parsing (Rust, Python, TypeScript, JavaScript, C, C++, Lua) and 5 with placeholder strategies (Kotlin, Makefile, Shell, Cython, WebAssembly). Three common file types are defined in the `Language` enum but lack detection or analysis:

| File Type | Enum Variant | `extension_to_language()` | TDG Support | Compliance Checks |
|-----------|-------------|--------------------------|-------------|-------------------|
| SQL (.sql) | Missing | Missing | None | None |
| Scala (.scala, .sc) | Missing (polyglot only) | Missing | None | None |
| Markdown (.md) | `Markdown = 4` | Missing | None | None |
| YAML (.yaml, .yml) | `Yaml = 7` | Missing | None | None |

SQL is the most widely-used data language (Stack Overflow 2024: #4 overall), Scala is critical for Spark/big data pipelines, and Markdown/YAML are infrastructure-as-code staples.

### 1.2 MLOps Model Quality Gap

The batuta stack (aprender, realizar, whisper.apr) manages ML models across three formats—GGUF, APR, SafeTensors—but PMAT has no visibility into model file health. Known production bugs demonstrate the need:

| Bug ID | Project | Issue | Root Cause |
|--------|---------|-------|------------|
| BUG-GGUF-001 | aprender | OOM via huge tensor_count in header | No upper bound validation on parsed counts |
| BUG-GGUF-002 | aprender | Integer overflow in shape product | Missing MAX_TENSOR_ELEMENTS guard |
| BUG-EXPORT-004 | aprender | GGUF export missing tokenizer metadata | No metadata completeness check |
| BUG-212 | aprender | Sharded SafeTensors conversion fails | No sharded index detection |
| BUG-MERGE-006 | aprender | NaN/Inf weights not validated during merge | No finite-value assertions |
| BUG-1 | aprender | F32 GGUF incompatible with realizar kernels | Missing quantization format check |
| BUG-TOK-002 | aprender | Tokenizer not found in HuggingFace layout | Single-path lookup, no fallback |

These bugs would be caught by static model file analysis—exactly what PMAT's compliance framework is designed for.

### 1.3 Theoretical Foundation

**Meyerovich & Rabkin (2013)** [1]: Empirical analysis of language adoption shows that tooling quality (IDE support, linters, static analysis) is the #2 predictor of language adoption after existing community size. Supporting SQL/Scala/YAML directly improves PMAT's value proposition.

**Amershi et al. (2019)** [2]: "Software Engineering for Machine Learning: A Case Study" identifies model validation, data management, and configuration as the top 3 SE challenges in ML. PMAT's compliance framework maps directly to these needs.

**Sculley et al. (2015)** [3]: "Hidden Technical Debt in Machine Learning Systems" demonstrates that model binaries are the #1 source of hidden tech debt. Static validation of model files prevents silent failures in production.

## 2. Language Support

### 2.1 SQL (.sql)

#### 2.1.1 Scope

SQL file analysis targets database migration scripts, stored procedures, and query files. PMAT will NOT provide a full SQL parser—instead it uses pattern-based analysis similar to Shell/Makefile support.

#### 2.1.2 Language Enum

```rust
// src/ast/core.rs
pub enum Language {
    // ... existing variants ...
    Lua = 16,
    Sql = 17,       // NEW
    Scala = 18,     // NEW
}
```

#### 2.1.3 Detection

```rust
// src/services/enhanced_language_detection.rs
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        // ... existing ...
        "sql" | "ddl" | "dml" => Some("sql"),
        // ...
    }
}
```

#### 2.1.4 TDG Scoring (Pattern-Based)

| Component | Max Points | What It Measures |
|-----------|-----------|-----------------|
| Structural Complexity | 25 | Nested subqueries depth, JOIN count, UNION chains |
| Semantic Complexity | 20 | Column count in SELECT, parameter binding patterns |
| Duplication Ratio | 20 | Repeated WHERE clauses, duplicate subqueries |
| Coupling Score | 15 | Table reference count, cross-schema references |
| Doc Coverage | 10 | `--` comment lines preceding statements |
| Consistency Score | 10 | Keyword casing (UPPER vs lower), indentation |

#### 2.1.5 Compliance Checks: CB-700 Series (SQL Best Practices)

| ID | Check | Severity | What It Detects |
|----|-------|----------|-----------------|
| CB-700 | SELECT * Usage | Warning | `SELECT *` in production queries |
| CB-701 | Missing WHERE on UPDATE/DELETE | Error | `UPDATE`/`DELETE` without `WHERE` clause |
| CB-702 | Implicit JOIN (Comma Join) | Warning | `FROM a, b WHERE a.id = b.id` instead of explicit JOIN |
| CB-703 | SQL Injection Pattern | Warning | String concatenation in query construction |
| CB-704 | Missing Index Hint | Info | Large table JOINs without index annotations |
| CB-705 | N+1 Query Pattern | Info | Loop-embedded SQL in co-located code files |

#### 2.1.6 Function Extraction

SQL "functions" for indexing:
- `CREATE FUNCTION/PROCEDURE name(...)` → named function
- `CREATE VIEW name AS` → named view (treated as function)
- `CREATE TRIGGER name` → named trigger
- Named CTEs: `WITH name AS (...)` → named CTE

### 2.2 Scala (.scala, .sc)

#### 2.2.1 Scope

Scala support builds on the existing `ScalaMapper` in the polyglot module. The goal is to promote Scala from polyglot-only to first-class language support with TDG grading.

#### 2.2.2 Detection

```rust
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        // ... existing ...
        "scala" | "sc" | "sbt" => Some("scala"),
        // ...
    }
}
```

#### 2.2.3 AST Strategy

Use tree-sitter-scala for AST parsing (feature-gated: `scala-ast`):

```rust
// src/ast/languages/scala.rs (new file)
pub struct ScalaStrategy {
    // Uses tree-sitter-scala parser
}

impl LanguageStrategy for ScalaStrategy {
    fn language(&self) -> Language { Language::Scala }
    fn can_parse(&self, path: &Path) -> bool {
        matches!(path.extension().and_then(|e| e.to_str()), Some("scala" | "sc"))
    }
    // ...
}
```

#### 2.2.4 TDG Scoring

| Component | Max Points | Scala-Specific Measures |
|-----------|-----------|------------------------|
| Structural Complexity | 25 | Pattern match depth, for-comprehension nesting, implicit chains |
| Semantic Complexity | 20 | Type parameter count, implicit parameter count, path-dependent types |
| Duplication Ratio | 20 | Repeated case patterns, duplicated trait implementations |
| Coupling Score | 15 | Import count, package object dependencies, implicit imports |
| Doc Coverage | 10 | Scaladoc `/** */` coverage on public API |
| Consistency Score | 10 | Naming conventions (camelCase methods, PascalCase types) |

#### 2.2.5 Compliance Checks: CB-800 Series (Scala Best Practices)

| ID | Check | Severity | What It Detects |
|----|-------|----------|-----------------|
| CB-800 | Mutable Collection Usage | Warning | `mutable.Map`/`Buffer` in non-local scope |
| CB-801 | Null Usage | Warning | `null` literal outside Java interop |
| CB-802 | Unrestricted Wildcard Import | Info | `import pkg._` pulling in entire package |
| CB-803 | Return Statement | Info | Explicit `return` (anti-idiomatic in Scala) |
| CB-804 | var Declaration | Warning | `var` in non-local scope (prefer `val`) |
| CB-805 | Blocking in Future | Warning | `Thread.sleep`/`Await.result` inside `Future` block |

### 2.3 Markdown (.md, .mdx)

#### 2.3.1 Scope

Markdown analysis focuses on documentation quality—link validation, heading structure, and readability metrics. No AST parsing needed; pattern-based analysis suffices.

#### 2.3.2 Detection

```rust
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        // ... existing ...
        "md" | "mdx" | "markdown" => Some("markdown"),
        // ...
    }
}
```

#### 2.3.3 TDG Scoring

| Component | Max Points | What It Measures |
|-----------|-----------|-----------------|
| Structural Complexity | 25 | Heading hierarchy violations (h1→h3 skip), nesting depth |
| Semantic Complexity | 20 | Flesch-Kincaid readability, avg sentence length |
| Duplication Ratio | 20 | Repeated paragraphs/sections |
| Coupling Score | 15 | External link count, cross-document reference density |
| Doc Coverage | 10 | N/A for markdown (full score by default) |
| Consistency Score | 10 | List marker style (- vs *), heading style (# vs ===) |

#### 2.3.4 Compliance Checks: CB-900 Series (Markdown Best Practices)

| ID | Check | Severity | What It Detects |
|----|-------|----------|-----------------|
| CB-900 | Broken Internal Link | Warning | `[text](./path.md)` where target doesn't exist |
| CB-901 | Heading Hierarchy Skip | Info | Jump from `#` to `###` without `##` |
| CB-902 | Missing Alt Text | Info | `![](image.png)` without alt text |
| CB-903 | Bare URL | Info | Raw URL without link syntax |
| CB-904 | Long Line | Info | Lines exceeding 120 characters (configurable) |

### 2.4 YAML (.yaml, .yml)

#### 2.4.1 Scope

YAML analysis targets CI/CD configurations (GitHub Actions, GitLab CI), Kubernetes manifests, and infrastructure-as-code files.

#### 2.4.2 Detection

```rust
fn extension_to_language(ext: &str) -> Option<&'static str> {
    match ext {
        // ... existing ...
        "yaml" | "yml" => Some("yaml"),
        // ...
    }
}
```

#### 2.4.3 TDG Scoring

| Component | Max Points | What It Measures |
|-----------|-----------|-----------------|
| Structural Complexity | 25 | Nesting depth, anchor/alias usage, multi-document complexity |
| Semantic Complexity | 20 | Schema-specific patterns (k8s resources, GHA workflows) |
| Duplication Ratio | 20 | Repeated blocks, copy-paste configuration |
| Coupling Score | 15 | External reference count (`$ref`, `!include`), cross-file deps |
| Doc Coverage | 10 | `#` comment lines per top-level key |
| Consistency Score | 10 | Indentation width consistency, quoting style |

#### 2.4.4 Compliance Checks: CB-950 Series (YAML Best Practices)

| ID | Check | Severity | What It Detects |
|----|-------|----------|-----------------|
| CB-950 | Truthy String Ambiguity | Warning | Unquoted `yes`/`no`/`on`/`off`/`true`/`false` as values |
| CB-951 | Excessive Nesting | Info | Nesting depth > 8 levels |
| CB-952 | Missing Required Fields | Warning | CI/CD config missing `name`/`on`/`jobs` (GHA-specific) |
| CB-953 | Pinned Action Version | Warning | `uses: actions/checkout@main` instead of `@v4` |
| CB-954 | Secret in Plain Text | Error | Values matching `password`, `secret`, `token` patterns |

## 3. MLOps Model File Support

### 3.1 Overview

PMAT will analyze ML model binary files without loading tensor data—parsing only headers and metadata. This enables quality checks on model files in git repos and CI/CD pipelines.

### 3.2 Supported Formats

| Format | Extensions | Magic Bytes | Origin |
|--------|-----------|-------------|--------|
| GGUF | `.gguf` | `0x4655_4747` ("GGUF" LE) | llama.cpp (Gerganov, 2023) |
| APR | `.apr` | `APR2` (0x41505232) | aprender (batuta stack) |
| SafeTensors | `.safetensors` | 8-byte u64 header length | HuggingFace (2023) |

### 3.3 Language Enum Extension

Model files are NOT added to the `Language` enum (they are not source code). Instead, they get a dedicated `ModelFormat` enum:

```rust
// src/models/model_format.rs (new file)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ModelFormat {
    Gguf,
    Apr,
    SafeTensors,
}

impl ModelFormat {
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext {
            "gguf" => Some(Self::Gguf),
            "apr" => Some(Self::Apr),
            "safetensors" => Some(Self::SafeTensors),
            _ => None,
        }
    }

    pub fn from_magic(bytes: &[u8]) -> Option<Self> {
        if bytes.len() < 4 { return None; }
        match &bytes[..4] {
            b"GGUF" | [0x47, 0x47, 0x55, 0x46] => Some(Self::Gguf),
            b"APR2" => Some(Self::Apr),
            _ => {
                // SafeTensors: first 8 bytes are u64 LE header size, typically < 10MB
                if bytes.len() >= 8 {
                    let header_len = u64::from_le_bytes(bytes[..8].try_into().ok()?);
                    if header_len > 0 && header_len < 100_000_000 {
                        Some(Self::SafeTensors)
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
        }
    }
}
```

### 3.4 Model Metadata Extraction

PMAT extracts only header/metadata—never loads tensor data into memory:

```rust
// src/services/model_inspector.rs (new file)
pub struct ModelMetadata {
    pub format: ModelFormat,
    pub file_size_bytes: u64,
    pub tensor_count: usize,
    pub total_parameters: u64,
    pub quantization: Option<String>,  // "Q4_K_M", "F16", "F32", etc.
    pub architecture: Option<String>,  // "llama", "whisper", "qwen2", etc.
    pub vocab_size: Option<usize>,
    pub context_length: Option<usize>,
    pub metadata: BTreeMap<String, String>,  // All key-value metadata
    pub tensors: Vec<TensorInfo>,
}

pub struct TensorInfo {
    pub name: String,
    pub dtype: String,
    pub shape: Vec<u64>,
    pub offset: u64,
    pub size_bytes: u64,
}
```

#### 3.4.1 GGUF Header Parsing

Parse GGUF v3 header (magic + version + tensor_count + metadata_count + KV pairs + tensor info). Safety limits from BUG-GGUF-001 fix:

```rust
const MAX_TENSOR_COUNT: u64 = 100_000;
const MAX_METADATA_COUNT: u64 = 100_000;
const MAX_DIMS: u32 = 16;
const MAX_TENSOR_ELEMENTS: usize = 4_000_000_000;
```

#### 3.4.2 APR Header Parsing

Parse APR v2 header (4-byte magic + 4-byte metadata_len + JSON metadata + tensor index). Extract architecture, quantization type, and tensor inventory from JSON metadata block.

#### 3.4.3 SafeTensors Header Parsing

Parse header-only (8-byte length + JSON header). Extract tensor names, dtypes, shapes, and data_offsets from JSON. Detect sharded files (`.safetensors.index.json`) per BUG-212 fix.

### 3.5 Compliance Checks: CB-1000 Series (MLOps Model Quality)

| ID | Check | Severity | What It Detects | Bug Ref |
|----|-------|----------|-----------------|---------|
| CB-1000 | Missing Model Card | Warning | Model file without `README.md` or `model_card.md` in same directory | — |
| CB-1001 | Oversized Tensor Count | Error | `tensor_count > 100,000` (likely corrupt header) | BUG-GGUF-001 |
| CB-1002 | Missing Tokenizer | Warning | Language model without `tokenizer.json`/`tokenizer.model` nearby | BUG-TOK-002 |
| CB-1003 | NaN/Inf in Metadata | Error | Non-finite values in numerical metadata fields | BUG-MERGE-006 |
| CB-1004 | Missing Architecture | Warning | GGUF file without `general.architecture` key | BUG-EXPORT-004 |
| CB-1005 | Quantization Mismatch | Warning | Filename says "Q4_K_M" but tensor dtypes are F32 | BUG-1 |
| CB-1006 | Sharded Without Index | Error | Numbered SafeTensors shards without `.index.json` | BUG-212 |
| CB-1007 | Excessive File Size | Info | Single model file > 10GB (suggest quantization or sharding) | — |
| CB-1008 | APR Missing CRC | Warning | APR file without CRC32 footer checksum | — |
| CB-1009 | Row-Major Violation | Error | APR tensor with column-major layout flag | LAYOUT-002 |

### 3.6 Model Inventory Command

```bash
# List all model files in project with metadata summary
pmat analyze models --path /path/to/project

# Example output:
# Model Inventory (3 files, 4.2 GB total)
# ┌──────────────────────────────┬──────────┬─────────┬─────────┬───────────┐
# │ File                         │ Format   │ Params  │ Quant   │ Size      │
# ├──────────────────────────────┼──────────┼─────────┼─────────┼───────────┤
# │ models/qwen-1.5b-q4km.gguf  │ GGUF v3  │ 1.5B    │ Q4_K_M  │ 1.1 GB    │
# │ models/whisper-tiny.apr      │ APR v2   │ 39M     │ F16     │ 78 MB     │
# │ weights/model.safetensors    │ SafeT    │ 7B      │ F32     │ 28 GB     │
# └──────────────────────────────┴──────────┴─────────┴─────────┴───────────┘
#
# ⚠ CB-1007: weights/model.safetensors exceeds 10GB — consider quantization
# ⚠ CB-1000: models/ has no model card (README.md)

# JSON output for CI/CD
pmat analyze models --format json --path .
```

### 3.7 Git Integration

Model files are typically in `.gitignore` or Git LFS. PMAT detects both scenarios:

```rust
// Model file discovery
fn discover_model_files(path: &Path) -> Vec<ModelFile> {
    // 1. Walk directory for *.gguf, *.apr, *.safetensors
    // 2. Check .gitattributes for LFS-tracked patterns
    // 3. Check .gitignore for excluded model patterns
    // 4. Report: tracked (dangerous), LFS-tracked (good), ignored (ok)
}
```

| Status | Meaning | Advisory |
|--------|---------|----------|
| Git-tracked (non-LFS) | Binary in git history | Error: Use Git LFS |
| Git LFS tracked | Properly managed | OK |
| .gitignore'd | Not version controlled | Info: Consider DVC or LFS |
| Not in repo | Local-only file | No advisory |

## 4. Implementation Plan

### Phase 1: Language Detection & Enum (1-2 days)

- [ ] Add `Sql = 17`, `Scala = 18` to `Language` enum in `src/ast/core.rs`
- [ ] Add SQL, Scala, Markdown, YAML extensions to `extension_to_language()` in `enhanced_language_detection.rs`
- [ ] Add `ModelFormat` enum to `src/models/model_format.rs`
- [ ] Wire model format detection into `enhanced_language_detection` (separate from Language)
- [ ] Tests: extension mapping, magic byte detection

### Phase 2: SQL & YAML Pattern Analysis (2-3 days)

- [ ] SQL pattern-based function extraction (CREATE FUNCTION/VIEW/TRIGGER, CTEs)
- [ ] SQL TDG scorer in `analyzer_impl1.rs` or new `analyzer_sql.rs`
- [ ] SQL compliance checks CB-700 to CB-705
- [ ] YAML pattern-based analysis (nesting depth, truthy ambiguity)
- [ ] YAML TDG scorer
- [ ] YAML compliance checks CB-950 to CB-954
- [ ] Tests: 20+ for SQL, 15+ for YAML

### Phase 3: Scala AST Support (2-3 days)

- [ ] Add `tree-sitter-scala` dependency (feature-gated: `scala-ast`)
- [ ] Implement `ScalaStrategy` in `src/ast/languages/scala.rs`
- [ ] Scala function/type/import extraction
- [ ] Scala TDG scorer with complexity analysis
- [ ] Scala compliance checks CB-800 to CB-805
- [ ] Promote existing `ScalaMapper` from polyglot to use new strategy
- [ ] Tests: 25+ for AST parsing, TDG, compliance

### Phase 4: Markdown Analysis (1-2 days)

- [ ] Markdown pattern-based analysis (heading structure, links, readability)
- [ ] Markdown TDG scorer
- [ ] Markdown compliance checks CB-900 to CB-904
- [ ] Link validation (internal only—no HTTP requests)
- [ ] Tests: 15+ for Markdown analysis

### Phase 5: Model File Support (3-4 days)

- [ ] GGUF header-only parser (reuse patterns from aprender, no data loading)
- [ ] APR header-only parser (parse JSON metadata block only)
- [ ] SafeTensors header-only parser (parse JSON header only)
- [ ] `ModelMetadata` extraction for all three formats
- [ ] `pmat analyze models` CLI command
- [ ] CB-1000 to CB-1009 compliance checks
- [ ] Git LFS / .gitignore detection for model files
- [ ] Tests: 30+ covering all formats, edge cases, BUG regression tests

### Phase 6: Book & Documentation (1 day)

- [ ] pmat-book chapter for SQL best practices (CB-700)
- [ ] pmat-book chapter for Scala best practices (CB-800)
- [ ] pmat-book chapter for Markdown best practices (CB-900)
- [ ] pmat-book chapter for YAML best practices (CB-950)
- [ ] pmat-book chapter for MLOps model quality (CB-1000)
- [ ] Update README with new language count and model support

## 5. Testing Strategy

### 5.1 Unit Tests

| Module | Test Count | Coverage Target |
|--------|-----------|----------------|
| SQL detection + TDG | 20 | Pattern matching, complexity scoring |
| SQL compliance (CB-700) | 15 | Each check with positive/negative cases |
| Scala AST parsing | 15 | Function/type/import extraction |
| Scala compliance (CB-800) | 12 | Each check with positive/negative cases |
| Markdown analysis | 15 | Heading structure, link validation |
| YAML analysis | 15 | Nesting, truthy detection, consistency |
| GGUF header parsing | 10 | Valid v3, corrupt headers, safety limits |
| APR header parsing | 10 | Valid v2, compressed, missing CRC |
| SafeTensors header parsing | 10 | Valid, sharded, oversized headers |
| Model compliance (CB-1000) | 15 | Each check against fixture files |

**Total: ~137 new tests**

### 5.2 Integration Tests

- [ ] `pmat comply check` on a Lua+SQL+YAML mixed project
- [ ] `pmat analyze models` on aprender/models/ directory
- [ ] `pmat analyze tdg` on Scala, SQL, Markdown, YAML files
- [ ] `cargo run --example` for each new language

### 5.3 Regression Tests

Model format tests must cover all known bugs:

```rust
#[test]
fn test_gguf_rejects_oversized_tensor_count() {
    // BUG-GGUF-001: tensor_count = u64::MAX should be rejected
}

#[test]
fn test_gguf_rejects_overflow_shape_product() {
    // BUG-GGUF-002: shape [u64::MAX, u64::MAX] should not overflow
}

#[test]
fn test_safetensors_detects_sharded_without_index() {
    // BUG-212: model-00001-of-00003.safetensors without .index.json
}

#[test]
fn test_apr_validates_row_major_layout() {
    // LAYOUT-002: column-major flag should be rejected
}
```

## 6. Non-Functional Requirements

- [ ] Header-only parsing: Model files MUST NOT load tensor data (zero-copy metadata extraction)
- [ ] Memory bound: Model inspection must use < 10MB RAM regardless of model file size
- [ ] Performance: Model header parsing < 100ms per file
- [ ] Performance: SQL/YAML/Markdown TDG scoring < 50ms per file
- [ ] Feature gates: `scala-ast` for tree-sitter-scala, model support always enabled
- [ ] Test coverage: >= 85% for new code (files with `coverage(off)` excluded)
- [ ] Zero clippy warnings on new code
- [ ] WASM compatibility: Model format parsing must compile to `wasm32-unknown-unknown`

## 7. Compliance ID Allocation

| Series | Language/Domain | Range |
|--------|---------------|-------|
| CB-600 | Lua Best Practices | CB-600 to CB-607 (existing) |
| CB-700 | SQL Best Practices | CB-700 to CB-705 (new) |
| CB-800 | Scala Best Practices | CB-800 to CB-805 (new) |
| CB-900 | Markdown Best Practices | CB-900 to CB-904 (new) |
| CB-950 | YAML Best Practices | CB-950 to CB-954 (new) |
| CB-1000 | MLOps Model Quality | CB-1000 to CB-1009 (new) |

## 8. Success Criteria

- [ ] `pmat comply check` reports CB-700/800/900/950/1000 violations on test fixtures
- [ ] `pmat analyze tdg --path file.sql` returns valid 7-component score
- [ ] `pmat analyze models` displays model inventory table
- [ ] All 137+ new tests pass in both debug and release mode
- [ ] Zero regressions in existing 20,764 tests
- [ ] pmat-book chapters validate via `make validate-book`
- [ ] `extension_to_language()` covers .sql, .scala, .sc, .md, .yaml, .yml

## References

[1] Meyerovich, L.A. & Rabkin, A.S. (2013). "Empirical Analysis of Programming Language Adoption." ACM SIGPLAN Notices, 48(10), pp. 1-18.

[2] Amershi, S. et al. (2019). "Software Engineering for Machine Learning: A Case Study." IEEE/ACM 41st International Conference on Software Engineering: Software Engineering in Practice (ICSE-SEIP).

[3] Sculley, D. et al. (2015). "Hidden Technical Debt in Machine Learning Systems." Advances in Neural Information Processing Systems 28 (NeurIPS 2015).

[4] Gerganov, G. (2023). "GGUF Format Specification." llama.cpp project, GitHub.

[5] HuggingFace (2023). "SafeTensors: A Simple, Safe Way to Store and Distribute Tensors." GitHub.

[6] PAIML (2026). "APR Format Specification v2.1.0." aprender project, `/docs/specifications/APR-SPEC.md`.

[7] Odersky, M. et al. (2004). "An Overview of the Scala Programming Language." EPFL Technical Report IC/2004/64.

[8] Chamberlin, D.D. & Boyce, R.F. (1974). "SEQUEL: A Structured English Query Language." ACM SIGFIDET Workshop on Data Description, Access and Control.

[9] Ben-Kiki, O., Evans, C. & Net, I. (2009). "YAML Ain't Markup Language (YAML) Version 1.2." yaml.org.
