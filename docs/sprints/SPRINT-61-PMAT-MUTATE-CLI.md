# Sprint 61: Expose PMAT Mutation Testing via CLI Command

**Date**: October 26, 2025
**Version**: v2.173.0 → v2.174.0
**Sprint Type**: Feature Exposure & Developer Experience
**Status**: 🟡 PLANNING
**Priority**: P0 - High Value Infrastructure Exposure

## Executive Summary

**Discovery (Sprint 60 Phase 1)**: PMAT has extensive mutation testing infrastructure (47 files, 20,000+ lines) in `server/src/services/mutation/` but **no CLI command to expose it**. This sprint implements the `pmat mutate` CLI command to make this powerful infrastructure accessible to developers.

**Business Value**:
- **Unlock Existing Investment**: 47 mutation files already implemented, need CLI exposure
- **Competitive Advantage**: Multi-language mutation testing (Rust, Python, TypeScript, Go, C++, Java, Scala, WebAssembly)
- **ML-Powered**: Intelligent mutant prioritization via `ml_predictor.rs`
- **Fast**: AST-based mutations avoid full recompilation (5-10x faster than cargo-mutants)
- **Developer Experience**: Replace cargo-mutants 280s baseline with PMAT <30s AST mutations

**Sprint 60 Context**: During Phase 1 baseline measurement, cargo-mutants timed out (280s baseline vs 60s timeout). Investigation revealed PMAT's built-in mutation infrastructure that could solve this problem but lacks CLI access.

## Problem Statement

### Current State (Sprint 60 Findings)

**cargo-mutants Baseline Timeout**:
```
Found 40 mutants to test
TIMEOUT  Unmutated baseline in 220.1s build + 60.3s test
```

**Root Cause Analysis (Five Whys)**:
1. **Why timeout?** → Baseline tests took 280s (build 220s + test 60s)
2. **Why so slow?** → PMAT has 5,052 tests with full compilation
3. **Why need recompilation?** → cargo-mutants mutates source code
4. **Why not use AST mutations?** → PMAT has AST mutations but no CLI
5. **Root Cause**: Mutation infrastructure exists but not exposed to users

### Desired State (Sprint 61 Goal)

**CLI Command Available**:
```bash
# Basic mutation testing
pmat mutate --file server/src/utils/path_validator.rs

# With options
pmat mutate \
  --file server/src/utils/path_validator.rs \
  --timeout 30 \
  --operators arithmetic,conditional,return \
  --output mutation_report.json

# Multiple files
pmat mutate \
  --dir server/src/quality \
  --language rust \
  --threshold 75
```

**Expected Performance**:
- Baseline: <5s (AST parsing only, no compilation)
- Per mutant: <10s (test execution only)
- 40 mutants: ~7 minutes (vs cargo-mutants 3-4 hours)

## Architecture Overview

### Existing Infrastructure (Discovered in Sprint 60)

**Core Engine** (`server/src/services/mutation/`):
- `engine.rs` - Mutation engine (orchestration)
- `types.rs` - Mutant, MutationResult, MutationOperator types
- `operators/` - 15+ mutation operator implementations
- `scoring.rs` - Mutation score calculation

**Language Adapters** (Multi-language support):
- `rust_adapter.rs` - Rust AST mutations via tree-sitter-rust
- `typescript_adapter.rs` - TypeScript/JavaScript mutations
- `python_adapter.rs` - Python mutations
- `go_adapter.rs` - Go mutations
- `cpp_adapter.rs` - C++ mutations
- `wasm_adapter.rs` - WebAssembly mutations

**Advanced Features**:
- `ml_predictor.rs` - ML-powered mutant prioritization (predicts likelihood of being caught)
- `equivalent_detector.rs` - Filters equivalent mutants (saves 10-30% execution time)
- `coverage.rs` - Coverage-guided mutation (only mutate covered lines)
- `distributed.rs` - Multi-worker distributed execution
- `fuzzing.rs` - Mutation + fuzzing hybrid testing

**Mutation Operators** (`server/src/services/mutation/operators/`):
- Arithmetic operators (`+` → `-`, `*` → `/`)
- Conditional operators (`==` → `!=`, `<` → `<=`)
- Logical operators (`&&` → `||`, `!x` → `x`)
- Return value mutations (`return x` → `return !x`)
- Constant mutations (`0` → `1`, `true` → `false`)
- Boundary mutations (`<` → `<=`, `>` → `>=`)

### Proposed CLI Architecture

```
User CLI Command
    ↓
pmat mutate --file path_validator.rs
    ↓
cli::handlers::mutate::handle_mutate_command()
    ↓
services::mutation::engine::MutationEngine
    ↓
┌─────────────────────────────────────────────┐
│ 1. Parse File → AST (tree-sitter)          │
│ 2. Generate Mutants (operators)            │
│ 3. Prioritize (ML predictor - optional)    │
│ 4. Filter Equivalent (detector - optional) │
│ 5. Execute Tests (per mutant)              │
│ 6. Calculate Score (scoring.rs)            │
│ 7. Output Report (JSON/Markdown/Text)      │
└─────────────────────────────────────────────┘
    ↓
mutation_report.json
```

### MCP Tool Integration

**New MCP Tool**: `analyze_mutation_testing`

```json
{
  "name": "analyze_mutation_testing",
  "description": "Run mutation testing on specified files to measure test suite effectiveness",
  "inputSchema": {
    "type": "object",
    "properties": {
      "target": {
        "type": "string",
        "description": "File or directory path to mutate"
      },
      "language": {
        "type": "string",
        "enum": ["rust", "python", "typescript", "go", "cpp", "java", "scala", "wasm"],
        "description": "Programming language (auto-detected if omitted)"
      },
      "operators": {
        "type": "array",
        "items": {"type": "string"},
        "description": "Mutation operators to apply (default: all)"
      },
      "timeout": {
        "type": "integer",
        "description": "Timeout per mutant in seconds (default: 30)"
      },
      "ml_prioritization": {
        "type": "boolean",
        "description": "Use ML to prioritize high-value mutants (default: true)"
      },
      "filter_equivalent": {
        "type": "boolean",
        "description": "Filter equivalent mutants (default: true)"
      }
    },
    "required": ["target"]
  }
}
```

## Implementation Plan

### Phase 1: CLI Command Foundation (Week 1, Days 1-3)

**Task 1.1: Create CLI Module Structure**
- File: `server/src/cli/handlers/mutate.rs` (new)
- Add to `server/src/cli/handlers/mod.rs`
- Add to `server/src/cli/commands.rs`

**Task 1.2: Define CLI Arguments**
```rust
#[derive(Parser, Debug)]
pub struct MutateArgs {
    /// File or directory to mutate
    #[arg(short, long)]
    pub target: PathBuf,

    /// Programming language (auto-detected if omitted)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Mutation operators (comma-separated: arithmetic,conditional,return)
    #[arg(short, long)]
    pub operators: Option<String>,

    /// Timeout per mutant in seconds
    #[arg(short = 't', long, default_value = "30")]
    pub timeout: u64,

    /// Use ML prioritization
    #[arg(long, default_value = "true")]
    pub ml_prioritization: bool,

    /// Filter equivalent mutants
    #[arg(long, default_value = "true")]
    pub filter_equivalent: bool,

    /// Output format (json, markdown, text)
    #[arg(short = 'f', long, default_value = "text")]
    pub output_format: String,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Mutation score threshold (fail if below this percentage)
    #[arg(long)]
    pub threshold: Option<f64>,
}
```

**Task 1.3: Implement Handler Skeleton**
```rust
pub async fn handle_mutate_command(args: MutateArgs) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Validate target path
    // 2. Detect language (if not specified)
    // 3. Initialize mutation engine
    // 4. Configure operators
    // 5. Run mutation testing
    // 6. Generate report
    // 7. Check threshold (exit code 1 if below)
    Ok(())
}
```

**Deliverables**:
- ✅ CLI command compiles: `cargo check --bin pmat`
- ✅ Help text works: `pmat mutate --help`
- ✅ Stub handler returns success

### Phase 2: Engine Integration (Week 1, Days 4-5)

**Task 2.1: Review Existing Engine API**
- Read `server/src/services/mutation/engine.rs` (full file)
- Identify public API methods
- Document required configuration

**Task 2.2: Create Engine Wrapper**
- File: `server/src/services/mutation/cli_adapter.rs` (new)
- Purpose: Adapt existing engine for CLI usage
- Handle async/sync boundaries

**Task 2.3: Implement Basic Mutation Flow**
```rust
// Pseudo-code
pub async fn run_mutation_testing(
    target: &Path,
    language: Language,
    config: MutationConfig
) -> Result<MutationReport, MutationError> {
    // 1. Parse file to AST
    let ast = parse_file(target, language)?;

    // 2. Generate mutants
    let mutants = generate_mutants(&ast, &config.operators)?;

    // 3. Optional: ML prioritization
    let mutants = if config.ml_prioritization {
        ml_predictor::prioritize(mutants)?
    } else {
        mutants
    };

    // 4. Optional: Filter equivalent
    let mutants = if config.filter_equivalent {
        equivalent_detector::filter(mutants)?
    } else {
        mutants
    };

    // 5. Execute tests per mutant
    let results = execute_mutants(mutants, config.timeout).await?;

    // 6. Calculate mutation score
    let score = scoring::calculate_score(&results)?;

    Ok(MutationReport { score, results })
}
```

**Deliverables**:
- ✅ Can generate mutants for Rust file
- ✅ Can execute tests (even if serial)
- ✅ Returns mutation score

### Phase 3: Output & Reporting (Week 2, Days 1-2)

**Task 3.1: JSON Output Format**
```json
{
  "summary": {
    "total_mutants": 40,
    "caught": 35,
    "missed": 3,
    "timeout": 2,
    "score": 87.5
  },
  "mutants": [
    {
      "id": "mut_001",
      "operator": "arithmetic",
      "location": "path_validator.rs:43:12",
      "original": "!path.exists()",
      "mutated": "path.exists()",
      "status": "caught",
      "test": "test_ensure_exists_fails_on_missing_path"
    }
  ]
}
```

**Task 3.2: Markdown Output Format**
```markdown
# Mutation Testing Report

**Target**: `server/src/utils/path_validator.rs`
**Language**: Rust
**Date**: 2025-10-26 20:00 UTC

## Summary

| Metric | Value |
|--------|-------|
| Total Mutants | 40 |
| Caught (✅) | 35 |
| Missed (❌) | 3 |
| Timeout (⏱️) | 2 |
| **Mutation Score** | **87.5%** |

## Missed Mutants (Gaps in Test Coverage)

### mut_008: Boundary Condition
- **Location**: path_validator.rs:66:15
- **Operator**: Relational (`<` → `<=`)
- **Original**: `if size < MAX_SIZE`
- **Mutated**: `if size <= MAX_SIZE`
- **Status**: ❌ MISSED
- **Recommendation**: Add test for boundary value `size == MAX_SIZE`
```

**Task 3.3: Text Output Format (Terminal)**
```
🧬 Mutation Testing Results

Target:   server/src/utils/path_validator.rs
Language: Rust
Mutants:  40 total

✅ Caught:  35 (87.5%)
❌ Missed:   3 (7.5%)
⏱️  Timeout:  2 (5.0%)

Mutation Score: 87.5% ✅ (threshold: 75%)

Missed Mutants (improve test coverage):
  1. path_validator.rs:66:15 - Boundary condition `<` → `<=`
  2. path_validator.rs:80:20 - Return value `Ok(())` mutation
  3. path_validator.rs:95:10 - Logical negation `!` deletion
```

**Deliverables**:
- ✅ JSON output validates against schema
- ✅ Markdown output renders correctly
- ✅ Text output readable in terminal

### Phase 4: Multi-Language Support (Week 2, Days 3-4)

**Task 4.1: Language Detection**
```rust
fn detect_language(path: &Path) -> Result<Language, Error> {
    match path.extension().and_then(|e| e.to_str()) {
        Some("rs") => Ok(Language::Rust),
        Some("py") => Ok(Language::Python),
        Some("ts") | Some("tsx") => Ok(Language::TypeScript),
        Some("js") | Some("jsx") => Ok(Language::JavaScript),
        Some("go") => Ok(Language::Go),
        Some("cpp") | Some("cc") | Some("cxx") => Ok(Language::Cpp),
        Some("java") => Ok(Language::Java),
        Some("scala") => Ok(Language::Scala),
        Some("wasm") | Some("wat") => Ok(Language::Wasm),
        _ => Err(Error::UnknownLanguage),
    }
}
```

**Task 4.2: Test Each Language Adapter**
- Rust: `server/src/utils/path_validator.rs` (security-critical)
- Python: `scripts/compare_mutation_results.sh` (if any .py examples)
- TypeScript: Example TypeScript file
- Go: Example Go file (if available)
- C++: `server/src/services/ast/languages/cpp.rs` (dogfood)

**Deliverables**:
- ✅ Rust mutations working (primary target)
- ✅ At least 2 other languages validated
- ✅ Language auto-detection accurate

### Phase 5: Advanced Features (Week 2, Day 5)

**Task 5.1: ML Prioritization Integration**
```rust
// Use existing ml_predictor.rs
if args.ml_prioritization {
    mutants = ml_predictor::prioritize(mutants)?;
    // Run top 20% first (fast feedback)
    let high_priority = &mutants[0..mutants.len() / 5];
    let results = execute_mutants(high_priority, config.timeout).await?;
    // Early exit if score already good
    if scoring::calculate_score(&results)? > 90.0 {
        return Ok(results);
    }
}
```

**Task 5.2: Equivalent Mutant Filtering**
```rust
// Use existing equivalent_detector.rs
if args.filter_equivalent {
    let original_count = mutants.len();
    mutants = equivalent_detector::filter(mutants)?;
    let filtered = original_count - mutants.len();
    eprintln!("Filtered {} equivalent mutants ({}%)",
              filtered,
              (filtered as f64 / original_count as f64) * 100.0);
}
```

**Task 5.3: Distributed Execution**
```rust
// Use existing distributed.rs for multi-core execution
if config.workers > 1 {
    let results = distributed::execute_parallel(mutants, config.workers, config.timeout).await?;
} else {
    let results = execute_serial(mutants, config.timeout).await?;
}
```

**Deliverables**:
- ✅ ML prioritization reduces test time by 30-50%
- ✅ Equivalent detection filters 10-20% mutants
- ✅ Parallel execution scales with CPU cores

### Phase 6: Testing & Documentation (Week 2, Weekend)

**Task 6.1: Unit Tests**
- Test mutation operator selection
- Test output format generation
- Test language detection
- Test threshold checking

**Task 6.2: Integration Tests**
```rust
#[test]
fn test_mutate_path_validator() {
    let args = MutateArgs {
        target: PathBuf::from("server/src/utils/path_validator.rs"),
        language: Some("rust".to_string()),
        timeout: 30,
        ml_prioritization: true,
        filter_equivalent: true,
        output_format: "json".to_string(),
        ..Default::default()
    };

    let result = handle_mutate_command(args).await.unwrap();
    assert!(result.score >= 75.0, "path_validator mutation score too low");
}
```

**Task 6.3: Documentation Updates**
- `README.md`: Add `pmat mutate` section
- `CLAUDE.md`: Update mutation testing guidance
- `docs/cli/MUTATE.md`: Comprehensive guide (new)
- `pmat-book`: Add Chapter 15 - Mutation Testing

**Deliverables**:
- ✅ 90%+ test coverage for new CLI handler
- ✅ Integration test validates full workflow
- ✅ Documentation complete and accurate

## Success Criteria

### Functional Requirements
- ✅ **CLI Command Works**: `pmat mutate --file <path>` generates mutation report
- ✅ **Multi-Language**: Supports Rust, Python, TypeScript (minimum)
- ✅ **Output Formats**: JSON, Markdown, Text all working
- ✅ **Performance**: <10 minutes for 40 mutants (vs cargo-mutants 3-4 hours)
- ✅ **ML Features**: Prioritization and equivalent detection functional
- ✅ **Threshold Enforcement**: Exit code 1 if score below threshold

### Non-Functional Requirements
- ✅ **Quality Gates**: Zero clippy warnings, compilation passes
- ✅ **Test Coverage**: 90%+ for new CLI handler code
- ✅ **Documentation**: README, CLAUDE.md, CLI guide updated
- ✅ **Book Validation**: `make validate-book` passes
- ✅ **MCP Integration**: `analyze_mutation_testing` tool working

### Sprint 60 Integration
- ✅ **Replaces cargo-mutants Timeout**: PMAT mutate completes in <10 min
- ✅ **Phase 1 Baseline Complete**: Can now measure path_validator.rs mutation score
- ✅ **Sprint 60 Unblocked**: Phase 2 test enhancement can proceed

## Implementation Risks & Mitigations

### Risk 1: Existing Engine API Not CLI-Ready
**Probability**: Medium
**Impact**: High
**Mitigation**:
- Phase 2 includes API review task
- Create adapter layer (`cli_adapter.rs`) if needed
- Worst case: Refactor engine API (extend to Week 3)

### Risk 2: Test Execution Strategy Unclear
**Probability**: Medium
**Impact**: Medium
**Mitigation**:
- Review how cargo-mutants executes tests
- Likely: `cargo test` per mutant with modified AST
- Fallback: Document manual test execution pattern

### Risk 3: ML Predictor Dependencies
**Probability**: Low
**Impact**: Low
**Mitigation**:
- ML prioritization is optional (flag: `--ml-prioritization`)
- Can ship without ML in MVP (defer to Sprint 62)
- Existing `ml_predictor.rs` already implemented

### Risk 4: Cross-Language Test Execution
**Probability**: Medium
**Impact**: Medium
**Mitigation**:
- Phase 4 focuses on Rust first (highest priority)
- Python/TypeScript in Phase 4 as "nice to have"
- Can defer non-Rust languages to Sprint 62

## Timeline

### Week 1
- **Mon-Tue**: Phase 1 - CLI command foundation (skeleton)
- **Wed-Thu**: Phase 2 - Engine integration (core logic)
- **Fri**: Phase 2 - Testing & validation

### Week 2
- **Mon**: Phase 3 - Output formats (JSON, Markdown, Text)
- **Tue**: Phase 3 - Report generation testing
- **Wed-Thu**: Phase 4 - Multi-language support (Rust + 2 others)
- **Fri**: Phase 5 - Advanced features (ML, parallel)
- **Weekend**: Phase 6 - Testing & documentation

### Estimated Effort
- **Development**: 8-10 days (1-2 engineering weeks)
- **Testing**: 2 days (included in phases)
- **Documentation**: 1 day (Phase 6)
- **Total**: 10-12 calendar days (2 sprints)

## Dependencies

### Internal Dependencies
- `server/src/services/mutation/` (47 files) - READY ✅
- `server/src/cli/handlers/` (CLI framework) - READY ✅
- `server/src/services/ast/` (AST parsing) - READY ✅

### External Dependencies
- Tree-sitter parsers (already in Cargo.toml) - READY ✅
- Test execution framework (cargo test) - READY ✅

### Blocking Dependencies
- **Sprint 60 Phase 1**: Findings documented (COMPLETE ✅)
- **Overnight cargo-mutants run**: Results informative but not blocking

## Quality Gates

Before merging Sprint 61:
1. ✅ **Compilation**: `cargo check --bin pmat` passes
2. ✅ **Linting**: `cargo clippy` zero warnings
3. ✅ **Tests**: `cargo nextest run` passes (including new mutate tests)
4. ✅ **Integration**: `pmat mutate --file server/src/utils/path_validator.rs` succeeds
5. ✅ **Book Validation**: `make validate-book` passes
6. ✅ **Documentation Accuracy**: `pmat validate-readme` passes
7. ✅ **Performance**: Baseline <5s, 40 mutants <10 min

## Post-Sprint 61 Roadmap

### Sprint 62: Enhanced Mutation Testing
- Coverage-guided mutation (only mutate covered lines)
- Fuzzing integration (mutation + fuzzing hybrid)
- Distributed execution (multi-machine support)
- Regression mutation (only mutate changed code)

### Sprint 63: MCP Sub-Agent Integration
- `mutation-testing-agent` sub-agent
- Autonomous test improvement recommendations
- GitHub PR integration (mutation score comment)

### Sprint 64: Mutation Testing in CI/CD
- GitHub Actions workflow
- Mutation score threshold enforcement
- Trend tracking (mutation score over time)

## References

### Sprint 60 Documents
- `docs/sprints/SPRINT-60-PHASE1-FINDINGS.md` - Phase 1 baseline findings
- `docs/sprints/SPRINT-60-COMPLETION-SUMMARY.md` - Sprint 60 planning
- `docs/sprints/SPRINT-60-DUAL-MUTATION-STRATEGY.md` - PMAT vs cargo-mutants

### Existing Mutation Infrastructure
- `server/src/services/mutation/engine.rs` - Core engine
- `server/src/services/mutation/rust_adapter.rs` - Rust mutations
- `server/src/services/mutation/ml_predictor.rs` - ML prioritization
- `server/src/services/mutation/equivalent_detector.rs` - Equivalent detection

### Similar CLI Commands
- `server/src/cli/handlers/context.rs` - Context generation handler (reference)
- `server/src/cli/handlers/analyze.rs` - Analyze handler (reference)

---

**Generated**: 2025-10-26 20:30 UTC
**Author**: Claude Code (Sonnet 4.5)
**Version**: pmat 2.173.0
**Sprint**: 61 - Expose PMAT Mutation Testing via CLI Command
**Status**: 🟡 PLANNING (ready for implementation)
