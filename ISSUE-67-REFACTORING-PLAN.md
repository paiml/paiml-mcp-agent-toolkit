# Issue #67: pmat Line Number Tracking Bug - EXTREME TDD Refactoring Plan

**Bug Report:** https://github.com/paiml/paiml-mcp-agent-toolkit/issues/67

## Executive Summary

**Problem:** When functions are extracted from one file to another (e.g., `utils.rs:500` → `attributes.rs:148`), pmat reports line numbers from the ORIGINAL file location, not the NEW file location.

**Root Cause:** The TDG (Technical Debt Graph) cache uses content hash as the primary key. When a function is moved between files, the content hash remains the same, so the cache returns stale line numbers from the old file location.

**Impact:** CRITICAL - Blocks git pre-commit hooks with false positive complexity violations on extracted files.

## Root Cause Analysis

### Current Architecture

```
handle_analyze_complexity (complexity_handlers.rs:471)
  ↓
analyze_single_file
  ↓
cli::language_analyzer::analyze_file_complexity
  ↓
ast_rust::analyze_rust_file_with_complexity
  ↓
TieredStore (tdg/storage.rs)
  - Cache Key: Blake3Hash (content only)
  - Returns: Cached line numbers from ORIGINAL file location
```

### The Problem

1. **File at OLD location:**
   - Path: `src/frontend/parser/utils.rs`
   - Function: `parse_rust_attribute_arguments`
   - Lines: 500-550
   - TDG Cache Key: `Blake3::hash(function_content)` → `abc123def456...`

2. **File EXTRACTED to NEW location:**
   - Path: `src/frontend/parser/utils_helpers/attributes.rs`
   - Function: `parse_rust_attribute_arguments` (SAME content)
   - Lines: 148-214 (DIFFERENT location)
   - TDG Cache Key: `Blake3::hash(function_content)` → `abc123def456...` (SAME!)

3. **pmat analyzes new file:**
   - Looks up `Blake3::hash(function_content)` in TDG cache
   - Finds cached entry with line numbers 500-550
   - Reports: "line 500-550" for a 214-line file (IMPOSSIBLE!)

### Evidence from Codebase

**TDG Storage** (`server/src/tdg/storage.rs:14-20`):
```rust
pub struct FileIdentity {
    pub path: PathBuf,                    // ✅ Has path
    pub content_hash: Blake3Hash,         // ❌ Used as PRIMARY key
    pub size_bytes: u64,
    pub modified_time: SystemTime,
}
```

**Cache Lookup** (`server/src/tdg/storage.rs:172-177`):
```rust
pub async fn retrieve_full(&self, hash: &Blake3Hash) -> Result<Option<FullTdgRecord>> {
    // Lookup by CONTENT HASH ONLY - ignores file path!
    if let Some(compressed) = self.warm_backend.get(hash.as_bytes())? {
        return Ok(Some(bincode::deserialize(&decompressed)?));
    }
}
```

**Complexity Metrics** (`server/src/services/complexity.rs:368-374`):
```rust
pub struct FunctionComplexity {
    pub name: String,
    pub line_start: u32,    // ❌ Cached from old location
    pub line_end: u32,      // ❌ Cached from old location
    pub metrics: ComplexityMetrics,
}
```

## Solution Design

### Phase 1: EXTREME TDD - RED Tests (✅ COMPLETED)

Created comprehensive test suite in `server/src/services/complexity_file_extraction_tests.rs`:

1. **RED Unit Tests:**
   - `red_test_file_extraction_line_numbers` - Core bug reproduction
   - `red_test_file_parameter_bypasses_cache` - Cache bypass verification
   - `red_test_same_function_different_files_different_line_numbers` - Content hash insufficiency

2. **RED Property-Based Tests:**
   - `prop_line_numbers_within_file_bounds` - Line numbers must NEVER exceed file size
   - `prop_file_path_affects_line_numbers` - File path changes force fresh analysis

3. **RED Fuzz Tests:**
   - `fuzz_line_number_bounds` - Edge cases (empty, single line, very long files)

### Phase 2: Implementation - GREEN (🔨 IN PROGRESS)

#### Step 1: Add Uncached Analysis Function

**File:** `server/src/services/complexity.rs`

Add new public function:

```rust
/// Analyze file complexity WITHOUT using TDG cache
///
/// This function performs fresh analysis and always reports accurate
/// line numbers from the current file location. Use this for:
/// - `--file` parameter (single file analysis)
/// - `--force-refresh` flag
/// - Pre-commit hooks
///
/// # Arguments
///
/// * `path` - File path to analyze
/// * `content` - File content (optional, reads from disk if None)
///
/// # Returns
///
/// Fresh `FileComplexityMetrics` with accurate line numbers
pub async fn analyze_file_complexity_uncached(
    path: &Path,
    content: Option<&str>,
) -> Result<FileComplexityMetrics> {
    // Implementation strategy:
    // 1. Read file content if not provided
    // 2. Detect language from file extension
    // 3. Call language-specific AST parser DIRECTLY
    // 4. Skip TDG cache lookup/storage
    // 5. Return fresh metrics with accurate line numbers

    todo!("GREEN phase implementation")
}
```

#### Step 2: Add --force-refresh Flag

**File:** `server/src/cli/commands.rs`

Update `AnalyzeComplexityCommand`:

```rust
/// Analyze code complexity
#[command(visible_aliases = &["comp"])]
Complexity {
    // ... existing fields ...

    /// Force fresh analysis, bypass TDG cache
    #[arg(long)]
    force_refresh: bool,
}
```

#### Step 3: Update Complexity Handler

**File:** `server/src/cli/handlers/complexity_handlers.rs`

Update `analyze_single_file` (line 68):

```rust
async fn analyze_single_file(
    file_path: &Path,
    config: &ComplexityConfig,
    force_refresh: bool,  // ✅ NEW parameter
) -> Result<Vec<FileComplexityMetrics>> {
    eprintln!("🔍 Analyzing complexity of file: {}", file_path.display());

    let full_path = if file_path.is_absolute() {
        file_path.to_path_buf()
    } else {
        config.project_path.join(file_path)
    };

    if !full_path.exists() {
        anyhow::bail!("File not found: {}", full_path.display());
    }

    let file_content = std::fs::read_to_string(&full_path)?;

    // ✅ CRITICAL FIX: Use uncached analysis for --file parameter
    let metrics = if force_refresh {
        crate::services::complexity::analyze_file_complexity_uncached(
            &full_path,
            Some(&file_content),
        ).await?
    } else {
        crate::cli::language_analyzer::analyze_file_complexity(&full_path, &file_content).await?
    };

    Ok(vec![metrics])
}
```

Update `handle_analyze_complexity` (line 471):

```rust
pub async fn handle_analyze_complexity(
    // ... existing parameters ...
    force_refresh: bool,  // ✅ NEW parameter
) -> Result<()> {
    // ...

    let mut file_metrics = if let Some(single_file) = file {
        // ✅ ALWAYS use uncached analysis for --file parameter
        analyze_single_file(&single_file, &config, true).await?  // force_refresh=true
    } else if force_refresh {
        // User explicitly requested fresh analysis
        analyze_multiple_files_uncached(&files, &config).await?
    } else {
        // Normal path with caching
        analyze_files_by_mode(file, files, &config).await?
    };

    // ...
}
```

### Phase 3: GREEN - Verify Tests Pass

Run test suite:

```bash
# Run RED tests (should now pass)
cargo test --lib complexity_file_extraction_tests::red_phase_tests --nocapture

# Run property tests with high iteration count
PROPTEST_CASES=10000 cargo test --lib complexity_file_extraction_tests::property_tests

# Run fuzz tests
cargo test --lib complexity_file_extraction_tests::fuzz_test_compatibility
```

### Phase 4: Mutation Testing

```bash
# Install cargo-mutants if not installed
cargo install cargo-mutants

# Run mutation testing on the fix
cargo mutants --file server/src/services/complexity.rs --test-threads 8

# Target: >80% mutation score
```

### Phase 5: Coverage Verification

```bash
# Generate coverage report
cargo llvm-cov --html --output-dir target/llvm-cov

# Verify coverage meets 85%+ threshold
# Focus on:
# - server/src/services/complexity.rs
# - server/src/cli/handlers/complexity_handlers.rs
# - server/src/services/complexity_file_extraction_tests.rs
```

### Phase 6: Integration Testing

Test the fix with the actual scenario from Issue #67:

```bash
# 1. Create test scenario
mkdir -p /tmp/pmat-test/src/frontend/parser/utils_helpers
cat > /tmp/pmat-test/src/frontend/parser/utils_helpers/attributes.rs << 'EOF'
// 147 lines of preamble
fn parse_rust_attribute_arguments(
    tokens: &[Token],
    start: usize,
) -> Result<(Vec<AttributeArg>, usize), String> {
    let mut args = Vec::new();
    let mut current = start;

    while current < tokens.len() {
        if tokens[current].is_comma() {
            current += 1;
            continue;
        }

        let (arg, next) = parse_single_arg(tokens, current)?;
        args.push(arg);
        current = next;
    }

    Ok((args, current))
}
// Total: 214 lines
EOF

# 2. Run pmat with --file parameter
pmat analyze complexity --file /tmp/pmat-test/src/frontend/parser/utils_helpers/attributes.rs

# 3. Verify output shows correct line numbers (148-XXX, not 500-550)

# 4. Test pre-commit hook scenario
cd /tmp/pmat-test
git init
git add src/frontend/parser/utils_helpers/attributes.rs
pmat analyze complexity --fail-on-violation --max-cyclomatic 10

# Should pass or fail based on ACTUAL complexity, not stale cache
```

## Testing Strategy Summary

| Test Type | Location | Purpose | Coverage Goal |
|-----------|----------|---------|---------------|
| Unit Tests (RED) | `complexity_file_extraction_tests.rs:18-168` | Bug reproduction | 100% of bug scenarios |
| Property Tests | `complexity_file_extraction_tests.rs:170-272` | Invariant verification | 10,000+ cases |
| Fuzz Tests | `complexity_file_extraction_tests.rs:274-322` | Edge case discovery | Comprehensive |
| Integration Tests | Manual (Phase 6) | Real-world validation | Pre-commit hooks |
| Mutation Tests | `cargo mutants` | Quality of tests | >80% mutation score |
| Coverage Tests | `cargo llvm-cov` | Code coverage | >85% line coverage |

## Success Criteria

- ✅ All RED tests turn GREEN
- ✅ Property tests pass with 10,000+ iterations
- ✅ Fuzz tests find no crashes
- ✅ Integration test with Issue #67 scenario works correctly
- ✅ Mutation score >80%
- ✅ Line coverage >85%
- ✅ Pre-commit hooks no longer block with false positives

## Implementation Checklist

- [x] Phase 1: Write RED tests (COMPLETED)
- [ ] Phase 2: Implement `analyze_file_complexity_uncached`
- [ ] Phase 2: Add `--force-refresh` flag to CLI
- [ ] Phase 2: Update `handle_analyze_complexity` to use uncached path
- [ ] Phase 3: Verify all RED tests turn GREEN
- [ ] Phase 4: Run mutation testing (target >80%)
- [ ] Phase 5: Verify coverage (target >85%)
- [ ] Phase 6: Integration test with real scenario
- [ ] Phase 7: Update CHANGELOG.md
- [ ] Phase 8: Close Issue #67

## Files Modified

1. ✅ `server/src/services/complexity_file_extraction_tests.rs` - RED test suite
2. ✅ `server/src/services/mod.rs` - Module registration
3. ⏳ `server/src/services/complexity.rs` - Add `analyze_file_complexity_uncached`
4. ⏳ `server/src/cli/commands.rs` - Add `--force-refresh` flag
5. ⏳ `server/src/cli/handlers/complexity_handlers.rs` - Update handler logic

## References

- **Issue:** https://github.com/paiml/paiml-mcp-agent-toolkit/issues/67
- **TDG Architecture:** `server/src/tdg/storage.rs`
- **Complexity Service:** `server/src/services/complexity.rs`
- **CLI Handler:** `server/src/cli/handlers/complexity_handlers.rs`
- **EXTREME TDD:** Property-based + Mutation + Fuzz testing

## Next Steps for Implementation

1. Implement `analyze_file_complexity_uncached` in `complexity.rs`
2. Wire up `--force-refresh` flag through CLI → handler
3. Run tests and achieve GREEN phase
4. Execute mutation testing
5. Verify coverage thresholds
6. Test with real-world scenario from Issue #67
7. Document fix in CHANGELOG.md
