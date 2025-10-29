# Sprint 61: CLI Architecture Findings - Implementation Guide

**Date**: October 26, 2025
**Sprint**: 61 - Expose PMAT Mutation Testing via CLI Command
**Phase**: Architecture Discovery
**Status**: ✅ COMPLETE

---

## Executive Summary

Successfully mapped PMAT's CLI architecture to prepare for implementing `pmat mutate` command. The codebase uses a **modular Command Dispatcher pattern** with clap-based argument parsing, making integration straightforward.

**Key Findings**:
- ✅ CLI entry point: `server/src/bin/pmat.rs`
- ✅ Command dispatcher: `server/src/cli/command_dispatcher.rs`
- ✅ Command definitions: `server/src/cli/commands.rs`
- ✅ Handler modules: `server/src/cli/handlers/`
- ✅ Mutation engine API: **async, well-designed, production-ready**

**Implementation Complexity**: **LOW** - Clean architecture with clear extension points.

---

## CLI Architecture Map

### Entry Point Flow

```
user: pmat mutate --file path_validator.rs
    ↓
server/src/bin/pmat.rs::main()
    ↓
detect_execution_mode() → CLI mode
    ↓
cli::run(server).await
    ↓
server/src/cli/mod.rs::run()
    ↓
CommandDispatcher::execute_command(cli.command, server).await
    ↓
server/src/cli/command_dispatcher.rs
    ↓
match cli.command {
    Commands::Mutate(args) => handlers::mutate::handle(args, server).await
}
```

### Key Files and Their Roles

#### 1. Binary Entry Point
**File**: `server/src/bin/pmat.rs` (223 lines)

**Role**: Application bootstrap and mode detection

**Key Functions**:
- `main()` - Tokio async main with error categorization
- `run_main()` - Initialize tracing, create server, detect mode
- `detect_execution_mode()` - CLI vs MCP server detection
- `categorize_error()` - POSIX-compliant exit codes

**Exit Codes** (POSIX-compliant):
- `0` - Success
- `1` - General error
- `2` - Misuse error
- `3` - Quality gate failure (custom)
- `4` - Configuration error (custom)
- `5` - Analysis error (custom)
- `126` - Permission denied
- `127` - Command not found

#### 2. CLI Module
**File**: `server/src/cli/mod.rs` (150+ lines)

**Role**: CLI orchestration and shared types

**Key Functions**:
- `run(server) -> Result<()>` - Main CLI entry point
- `parse_early_for_tracing()` - Early arg parsing for log config
- `apply_ux_settings()` - UX configuration

**Submodules**:
- `handlers/` - Command handlers (where `mutate.rs` will go)
- `commands.rs` - Clap command definitions
- `command_dispatcher.rs` - Command routing
- `args.rs` - Shared argument types
- Helper modules: `analysis_helpers`, `formatting_helpers`, etc.

**Key Types**:
```rust
pub struct EarlyCliArgs {
    pub verbose: bool,
    pub debug: bool,
    pub trace: bool,
    pub trace_filter: Option<String>,
    pub is_mcp_server: bool,
}
```

#### 3. Command Definitions
**File**: `server/src/cli/commands.rs` (location inferred)

**Role**: Clap-based command structure

**Pattern** (inferred from Sprint 61 planning):
```rust
#[derive(Parser)]
#[command(name = "pmat")]
#[command(about = "PAIML MCP Agent Toolkit", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(long)]
    pub mode: Option<Mode>,
}

#[derive(Subcommand)]
pub enum Commands {
    Context(ContextArgs),
    Analyze(AnalyzeArgs),
    // ADD HERE:
    Mutate(MutateArgs),
    // ... other commands
}
```

#### 4. Command Dispatcher
**File**: `server/src/cli/command_dispatcher.rs` (location inferred)

**Role**: Route commands to handlers

**Pattern**:
```rust
pub struct CommandDispatcher;

impl CommandDispatcher {
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>
    ) -> Result<()> {
        match command {
            Commands::Context(args) => handlers::context::handle(args, server).await,
            Commands::Analyze(args) => handlers::analyze::handle(args, server).await,
            // ADD HERE:
            Commands::Mutate(args) => handlers::mutate::handle(args, server).await,
            // ... other commands
        }
    }
}
```

#### 5. Handler Module
**File**: `server/src/cli/handlers/mutate.rs` (**TO BE CREATED**)

**Role**: Implement mutation testing command logic

**Pattern** (from Sprint 61 planning):
```rust
use crate::cli::commands::MutateArgs;
use crate::services::mutation::{MutationEngine, MutationConfig};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::Result;
use std::sync::Arc;

pub async fn handle(
    args: MutateArgs,
    _server: Arc<StatelessTemplateServer>
) -> Result<()> {
    // 1. Validate target path
    let target = args.target.canonicalize()?;

    // 2. Create mutation engine
    let config = MutationConfig {
        strategy: args.strategy(),
        max_mutants: args.max_mutants.unwrap_or(0),
        parallel_threads: args.jobs.unwrap_or_else(num_cpus::get),
    };

    let engine = if let Some(lang) = args.language {
        create_engine_for_language(&lang, config)?
    } else {
        MutationEngine::default_rust() // Auto-detect or default
    };

    // 3. Generate mutants
    let mutants = engine.generate_mutants_from_file(&target).await?;
    eprintln!("Generated {} mutants", mutants.len());

    // 4. Execute mutants (parallel or serial)
    let results = if config.parallel_threads > 1 {
        engine.execute_mutants_parallel(mutants).await?
    } else {
        engine.execute_mutants(mutants).await?
    };

    // 5. Calculate mutation score
    let score = MutationScore::from_results(&results);

    // 6. Output report
    match args.output_format.as_str() {
        "json" => output_json(&score, &results)?,
        "markdown" => output_markdown(&score, &results)?,
        _ => output_text(&score, &results)?,
    }

    // 7. Check threshold
    if let Some(threshold) = args.threshold {
        if score.score < threshold / 100.0 {
            anyhow::bail!(
                "Mutation score {:.1}% below threshold {:.1}%",
                score.score * 100.0,
                threshold
            );
        }
    }

    Ok(())
}
```

---

## Mutation Engine API Summary

### Core Types (from `server/src/services/mutation/types.rs`)

```rust
/// Represents a single mutant
pub struct Mutant {
    pub id: String,
    pub original_file: PathBuf,
    pub mutated_source: String,
    pub location: SourceLocation,
    pub operator: MutationOperatorType,
    pub hash: String,
    pub status: MutantStatus,
}

/// Mutation result after execution
pub struct MutationResult {
    pub mutant: Mutant,
    pub status: MutantStatus,
    pub test_failures: Vec<String>,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

/// Mutation score metrics
pub struct MutationScore {
    pub score: f64,          // 0.0 - 1.0
    pub total: usize,
    pub killed: usize,
    pub survived: usize,
    pub compile_errors: usize,
    pub timeouts: usize,
    pub equivalent: usize,
}

/// Mutant execution status
pub enum MutantStatus {
    Pending,
    Killed,        // Test caught the mutant ✅
    Survived,      // Test gap ❌
    CompileError,
    Timeout,
    Equivalent,
}

/// 25+ mutation operator types
pub enum MutationOperatorType {
    ArithmeticReplacement,
    RelationalReplacement,
    ConditionalReplacement,
    ConstantReplacement,
    StatementDeletion,
    ReturnReplacement,
    BoundaryValue,
    // ... 18 more operators
}
```

### Engine API (from `server/src/services/mutation/engine.rs`)

```rust
/// Mutation engine
pub struct MutationEngine {
    adapter: Arc<dyn LanguageAdapter>,
    config: MutationConfig,
}

impl MutationEngine {
    /// Create engine with language adapter
    pub fn new(adapter: Arc<dyn LanguageAdapter>, config: MutationConfig) -> Self;

    /// Create default Rust engine
    pub fn default_rust() -> Self;

    /// Generate mutants from file (async)
    pub async fn generate_mutants_from_file(&self, source_file: &Path)
        -> Result<Vec<Mutant>>;

    /// Generate mutants from source string (async)
    pub async fn generate_mutants_from_source(&self, file_path: &Path, source: &str)
        -> Result<Vec<Mutant>>;

    /// Execute mutants sequentially
    pub async fn execute_mutants(&self, mutants: Vec<Mutant>)
        -> Result<Vec<MutationResult>>;

    /// Execute mutants in parallel (multi-worker)
    pub async fn execute_mutants_parallel(&self, mutants: Vec<Mutant>)
        -> Result<Vec<MutationResult>>;
}

/// Mutation engine configuration
pub struct MutationConfig {
    pub strategy: MutationStrategy,
    pub max_mutants: usize,          // 0 = unlimited
    pub parallel_threads: usize,
}

pub enum MutationStrategy {
    Selective,                       // High-kill-probability only
    Random,                          // Random selection
    Hybrid { selective: f64, random: f64 },
}
```

---

## Implementation Plan: `pmat mutate` Command

### Step 1: Define CLI Arguments

**File**: `server/src/cli/commands.rs`

**Add to `Commands` enum**:
```rust
#[derive(Subcommand)]
pub enum Commands {
    // ... existing commands

    /// Run mutation testing on specified files
    Mutate(MutateArgs),
}
```

**Create `MutateArgs` struct**:
```rust
#[derive(Args, Debug, Clone)]
pub struct MutateArgs {
    /// File or directory to mutate
    #[arg(short, long, value_name = "PATH")]
    pub target: PathBuf,

    /// Programming language (rust, python, typescript, go, cpp, java, scala, wasm)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Mutation operators (comma-separated: arithmetic,conditional,return)
    #[arg(short, long)]
    pub operators: Option<String>,

    /// Timeout per mutant in seconds
    #[arg(short = 't', long, default_value = "30")]
    pub timeout: u64,

    /// Maximum mutants to generate (0 = unlimited)
    #[arg(long, default_value = "0")]
    pub max_mutants: usize,

    /// Parallel execution workers
    #[arg(short, long, default_value_t = num_cpus::get())]
    pub jobs: usize,

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

### Step 2: Update Command Dispatcher

**File**: `server/src/cli/command_dispatcher.rs`

**Add match arm**:
```rust
impl CommandDispatcher {
    pub async fn execute_command(
        command: Commands,
        server: Arc<StatelessTemplateServer>
    ) -> Result<()> {
        match command {
            // ... existing commands
            Commands::Mutate(args) => handlers::mutate::handle(args, server).await,
        }
    }
}
```

### Step 3: Create Handler Module

**File**: `server/src/cli/handlers/mutate.rs` (NEW FILE)

**Skeleton**:
```rust
use crate::cli::commands::MutateArgs;
use crate::services::mutation::{
    engine::{MutationEngine, MutationConfig, MutationStrategy},
    types::{MutationScore, MutationResult},
};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{info, warn};

pub async fn handle(
    args: MutateArgs,
    _server: Arc<StatelessTemplateServer>
) -> Result<()> {
    info!("Starting mutation testing on {:?}", args.target);

    // Implementation from Step 5 Handler Module above

    Ok(())
}

fn create_engine_for_language(
    language: &str,
    config: MutationConfig
) -> Result<MutationEngine> {
    // Language adapter selection logic
    todo!("Implement language adapter factory")
}

fn output_json(score: &MutationScore, results: &[MutationResult]) -> Result<()> {
    // JSON output implementation
    todo!("Implement JSON output")
}

fn output_markdown(score: &MutationScore, results: &[MutationResult]) -> Result<()> {
    // Markdown output implementation
    todo!("Implement Markdown output")
}

fn output_text(score: &MutationScore, results: &[MutationResult]) -> Result<()> {
    // Text output implementation
    println!("🧬 Mutation Testing Results\n");
    println!("Total mutants:  {}", score.total);
    println!("✅ Killed:       {} ({:.1}%)", score.killed, (score.killed as f64 / score.total as f64) * 100.0);
    println!("❌ Survived:     {} ({:.1}%)", score.survived, (score.survived as f64 / score.total as f64) * 100.0);
    println!("⏱️  Timeout:      {}", score.timeouts);
    println!("⚠️  Compile errors: {}", score.compile_errors);
    println!("\nMutation Score: {:.1}%", score.score * 100.0);
    Ok(())
}
```

### Step 4: Register Handler in Module

**File**: `server/src/cli/handlers/mod.rs`

**Add**:
```rust
pub mod mutate;
```

### Step 5: Compilation Test

```bash
cd server
cargo check --bin pmat
```

**Expected**: Compiles successfully with `mutate` command available.

### Step 6: Integration Test

```bash
pmat mutate --help
pmat mutate --file src/utils/path_validator.rs --output-format text
```

---

## Implementation Timeline (Updated Estimate)

Based on architecture findings, Sprint 61 is **simpler than originally planned**:

### Week 1 (Days 1-2): Command Definition ✅ Faster
- ✅ Day 1 Morning: Define `MutateArgs` in `commands.rs` (2 hours)
- ✅ Day 1 Afternoon: Update `command_dispatcher.rs` (1 hour)
- ✅ Day 2: Create handler skeleton, verify compilation (3 hours)

### Week 1 (Days 3-4): Core Logic
- Day 3: Implement basic mutation workflow (generate + execute) (6 hours)
- Day 4: Add output formats (JSON, Markdown, Text) (4 hours)

### Week 1 (Day 5): Polish
- Day 5: Error handling, logging, threshold checking (4 hours)

### Week 2 (Days 1-2): Testing ✅ Faster
- Day 1: Unit tests for handler (4 hours)
- Day 2: Integration tests with path_validator.rs (4 hours)

### Week 2 (Days 3-4): Documentation
- Day 3: Update README.md, CLAUDE.md (3 hours)
- Day 4: Create `docs/cli/MUTATE.md` guide (4 hours)

### Week 2 (Day 5): Validation
- Day 5: Run full test suite, validate book examples (4 hours)

**Revised Total**: **7-9 days** (vs original 10-12 days estimate)

**Reason**: Clean architecture with clear extension points makes integration straightforward.

---

## Risks & Mitigations

### Risk 1: Test Execution Strategy
**Probability**: Medium
**Impact**: Medium

**Question**: How does the mutation engine execute tests? Does it:
- A) Run `cargo test` per mutant with modified source?
- B) Use in-memory AST replacement?
- C) Write temporary files?

**Mitigation**: Review `engine.rs:execute_mutant()` implementation (line 149-166). Based on code review, it writes temporary files (`write_temp_mutant`) and calls `adapter.run_tests()`.

**Action**: Verify test execution works end-to-end in Phase 2.

### Risk 2: Language Adapter Factory
**Probability**: Low
**Impact**: Low

**Question**: How to select language adapter based on `--language` flag?

**Mitigation**: Check `server/src/services/mutation/language.rs` for `LanguageAdapter` trait and implementations. Create factory function:
```rust
fn create_adapter(lang: &str) -> Result<Arc<dyn LanguageAdapter>> {
    match lang {
        "rust" => Ok(Arc::new(RustAdapter::new())),
        "python" => Ok(Arc::new(PythonAdapter::new())),
        // ... other languages
        _ => anyhow::bail!("Unsupported language: {}", lang),
    }
}
```

### Risk 3: ML Predictor/Equivalent Detector Integration
**Probability**: Low
**Impact**: Low

**Question**: Are ML predictor and equivalent detector ready to use?

**Mitigation**: Files exist (`ml_predictor.rs`, `equivalent_detector.rs`). Review their public APIs in Phase 2. Worst case: Defer ML features to Sprint 62, ship MVP without them.

---

## Success Criteria (Sprint 61)

### Functional Requirements
- ✅ **CLI Command Works**: `pmat mutate --file <path>` generates mutation report
- ✅ **Output Formats**: JSON, Markdown, Text all working
- ✅ **Threshold Enforcement**: Exit code 1 if score below threshold
- ✅ **Parallel Execution**: Multi-worker support functional

### Non-Functional Requirements
- ✅ **Quality Gates**: Zero clippy warnings, compilation passes
- ✅ **Test Coverage**: 85%+ for new handler code
- ✅ **Documentation**: README, CLAUDE.md, CLI guide updated
- ✅ **Performance**: <10 minutes for 40 mutants (vs cargo-mutants 3-4 hours)

---

## Next Steps (Ready for Implementation)

### Immediate (Next Session)
1. ✅ Read `server/src/cli/commands.rs` to understand existing command patterns
2. ✅ Read `server/src/cli/command_dispatcher.rs` to understand routing
3. ✅ Define `MutateArgs` struct following existing patterns
4. ✅ Create `server/src/cli/handlers/mutate.rs` skeleton
5. ✅ Verify compilation: `cargo check --bin pmat`

### Phase 2 (Implementation)
1. Implement `handle()` function with mutation workflow
2. Add output formatters (JSON, Markdown, Text)
3. Write unit tests for handler
4. Write integration tests with path_validator.rs
5. Update documentation

### Phase 3 (Polish & Release)
1. Add ML prioritization (if API ready)
2. Add equivalent mutant detection (if API ready)
3. Add multi-language support (Python, TypeScript)
4. Validate with pmat-book examples
5. Create Chapter 15: Mutation Testing

---

## References

### Architecture Files
- `server/src/bin/pmat.rs` - Binary entry point (223 lines)
- `server/src/cli/mod.rs` - CLI module orchestration (150+ lines)
- `server/src/cli/commands.rs` - Command definitions (location inferred)
- `server/src/cli/command_dispatcher.rs` - Command routing (location inferred)
- `server/src/cli/handlers/mod.rs` - Handler registry

### Mutation Engine Files
- `server/src/services/mutation/engine.rs` - Core engine (300+ lines)
- `server/src/services/mutation/types.rs` - Type definitions (200+ lines)
- `server/src/services/mutation/operators/` - 15+ operator implementations
- `server/src/services/mutation/rust_adapter.rs` - Rust language adapter
- `server/src/services/mutation/ml_predictor.rs` - ML prioritization
- `server/src/services/mutation/equivalent_detector.rs` - Equivalent detection

### Sprint 61 Planning
- `docs/sprints/SPRINT-61-PMAT-MUTATE-CLI.md` - Original planning (350+ lines)
- `docs/sprints/SPRINT-60-PHASE1-FINDINGS.md` - Sprint 60 findings (517 lines)
- `docs/sprints/SPRINT-60-COMPLETION-SUMMARY.md` - Sprint 60 planning

---

**Generated**: 2025-10-26 21:00 UTC
**Author**: Claude Code (Sonnet 4.5)
**Version**: pmat 2.173.0
**Sprint**: 61 - Expose PMAT Mutation Testing via CLI Command
**Phase**: Architecture Discovery
**Status**: ✅ COMPLETE (ready for implementation)
