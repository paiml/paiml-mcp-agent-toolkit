# Sprint 61: Implementation Kickoff - Next Session Guide

**Date**: October 26, 2025
**Sprint**: 61 - Expose PMAT Mutation Testing via CLI Command
**Status**: 🟢 READY FOR IMPLEMENTATION
**Estimated Time**: 7-9 days

---

## Quick Start (5 Steps to First Working Command)

### Step 1: Read Existing Command Pattern (15 min)
```bash
cd /home/noah/src/paiml-mcp-agent-toolkit/server

# Study existing command structure
cat src/cli/commands.rs | grep -A 20 "pub enum Commands"
cat src/cli/command_dispatcher.rs | grep -A 30 "execute_command"
```

**Goal**: Understand how existing commands (Context, Analyze) are structured.

### Step 2: Define Mutate Command (30 min)
**File**: `src/cli/commands.rs`

**Add to `Commands` enum**:
```rust
/// Run mutation testing on specified files
Mutate(MutateArgs),
```

**Add `MutateArgs` struct** (copy from `SPRINT-61-CLI-ARCHITECTURE-FINDINGS.md` lines 300-340):
```rust
#[derive(Args, Debug, Clone)]
pub struct MutateArgs {
    /// File or directory to mutate
    #[arg(short, long, value_name = "PATH")]
    pub target: PathBuf,

    /// Programming language (rust, python, typescript, go, cpp)
    #[arg(short, long)]
    pub language: Option<String>,

    /// Timeout per mutant in seconds
    #[arg(short = 't', long, default_value = "30")]
    pub timeout: u64,

    /// Parallel execution workers
    #[arg(short, long)]
    pub jobs: Option<usize>,

    /// Output format (json, markdown, text)
    #[arg(short = 'f', long, default_value = "text")]
    pub output_format: String,

    /// Output file (stdout if omitted)
    #[arg(short, long)]
    pub output: Option<PathBuf>,

    /// Mutation score threshold (fail if below)
    #[arg(long)]
    pub threshold: Option<f64>,
}
```

### Step 3: Update Command Dispatcher (15 min)
**File**: `src/cli/command_dispatcher.rs`

**Add match arm**:
```rust
Commands::Mutate(args) => handlers::mutate::handle(args, server).await,
```

### Step 4: Create Handler Skeleton (45 min)
**File**: `src/cli/handlers/mutate.rs` (NEW)

```rust
use crate::cli::commands::MutateArgs;
use crate::services::mutation::{
    engine::{MutationEngine, MutationConfig},
    types::{MutationScore, MutationResult},
};
use crate::stateless_server::StatelessTemplateServer;
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::info;

pub async fn handle(
    args: MutateArgs,
    _server: Arc<StatelessTemplateServer>
) -> Result<()> {
    info!("Starting mutation testing on {:?}", args.target);

    // 1. Validate target
    let target = args.target.canonicalize()
        .context("Target file not found")?;

    // 2. Create engine
    let config = MutationConfig {
        strategy: crate::services::mutation::engine::MutationStrategy::Selective,
        max_mutants: 0,
        parallel_threads: args.jobs.unwrap_or_else(num_cpus::get),
    };
    let engine = MutationEngine::default_rust();

    // 3. Generate mutants
    let mutants = engine.generate_mutants_from_file(&target).await?;
    eprintln!("Generated {} mutants", mutants.len());

    // 4. Execute mutants
    let results = if config.parallel_threads > 1 {
        engine.execute_mutants_parallel(mutants).await?
    } else {
        engine.execute_mutants(mutants).await?
    };

    // 5. Calculate score
    let score = MutationScore::from_results(&results);

    // 6. Output
    output_text(&score, &results)?;

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

fn output_text(score: &MutationScore, _results: &[MutationResult]) -> Result<()> {
    println!("\n🧬 Mutation Testing Results\n");
    println!("Total mutants:  {}", score.total);
    println!("✅ Killed:       {} ({:.1}%)", score.killed,
             (score.killed as f64 / score.total as f64) * 100.0);
    println!("❌ Survived:     {} ({:.1}%)", score.survived,
             (score.survived as f64 / score.total as f64) * 100.0);
    println!("\nMutation Score: {:.1}%\n", score.score * 100.0);
    Ok(())
}
```

**Register in `src/cli/handlers/mod.rs`**:
```rust
pub mod mutate;
```

### Step 5: Test Compilation (10 min)
```bash
cd server
cargo check --bin pmat

# Expected: Compiles successfully
# If errors: Fix imports, check types
```

---

## Verification (After Step 5)

```bash
# Test help text
cargo run --bin pmat -- mutate --help

# Expected output:
# Run mutation testing on specified files
#
# Usage: pmat mutate [OPTIONS] --target <PATH>
#
# Options:
#   -t, --target <PATH>           File or directory to mutate
#   -l, --language <STRING>       Programming language
#   ...
```

---

## Next Steps (Days 2-9)

### Day 2: Test on Real File
```bash
# Run on path_validator.rs (40 mutants)
cargo run --bin pmat -- mutate --target src/utils/path_validator.rs

# Expected:
# - Generates mutants (may take time)
# - Executes tests
# - Shows mutation score
```

### Day 3-4: Add Output Formats
- JSON output (`output_json()`)
- Markdown output (`output_markdown()`)
- File output support

### Day 5-6: Add Language Support
- Language detection
- Multi-language adapters
- Python, TypeScript support

### Day 7-8: Testing
- Unit tests for handler
- Integration tests
- Property tests

### Day 9: Documentation
- Update README.md
- Update CLAUDE.md
- Create docs/cli/MUTATE.md

---

## Reference Documents

### Architecture (Read These First)
1. **`SPRINT-61-CLI-ARCHITECTURE-FINDINGS.md`** (650 lines)
   - Complete CLI architecture map
   - Mutation engine API reference
   - Implementation examples

2. **`SPRINT-61-PMAT-MUTATE-CLI.md`** (350 lines)
   - Original planning document
   - Success criteria
   - Risk analysis

### Mutation Engine API
**Location**: `server/src/services/mutation/`

**Key Files**:
- `engine.rs` - MutationEngine, MutationConfig (300+ lines)
- `types.rs` - Mutant, MutationResult, MutationScore (200+ lines)
- `rust_adapter.rs` - Rust language mutations

**Key Methods**:
```rust
// Generate mutants from file
pub async fn generate_mutants_from_file(&self, path: &Path)
    -> Result<Vec<Mutant>>;

// Execute mutants in parallel
pub async fn execute_mutants_parallel(&self, mutants: Vec<Mutant>)
    -> Result<Vec<MutationResult>>;

// Calculate mutation score
impl MutationScore {
    pub fn from_results(results: &[MutationResult]) -> Self;
}
```

---

## Troubleshooting

### Issue: Compilation Errors
**Check**:
- Import paths (`use crate::services::mutation::...`)
- Type names match (`MutationEngine`, not `Engine`)
- Async/await syntax correct

**Fix**: Review `SPRINT-61-CLI-ARCHITECTURE-FINDINGS.md` for correct imports

### Issue: Test Execution Hangs
**Reason**: Running tests for 40 mutants takes time (5-10 min)

**Fix**:
- Start with `--max-mutants 5` for quick testing
- Use `--jobs 1` for serial execution (easier debugging)

### Issue: No Mutants Generated
**Check**:
- Target file has Rust code
- File is readable
- AST parsing succeeds

**Debug**: Add `eprintln!("Parsed file: {:?}", target);` before generation

---

## Success Criteria (Minimum Viable Product)

**Day 1 (Today's Goal)**:
- ✅ Command compiles: `cargo check --bin pmat`
- ✅ Help works: `pmat mutate --help`
- ✅ Stub handler returns success

**Week 1 Goal**:
- ✅ Generates mutants from Rust file
- ✅ Executes tests (even if slow)
- ✅ Shows mutation score (text format)

**Week 2 Goal**:
- ✅ JSON/Markdown output formats
- ✅ Threshold enforcement
- ✅ Tests passing
- ✅ Documentation complete

---

## Files to Modify

### Modify (3 files)
1. **`src/cli/commands.rs`** - Add `Commands::Mutate` + `MutateArgs`
2. **`src/cli/command_dispatcher.rs`** - Add match arm
3. **`src/cli/handlers/mod.rs`** - Add `pub mod mutate;`

### Create (1 file)
1. **`src/cli/handlers/mutate.rs`** - Handler implementation

---

## Expected Timeline

| Day | Task | Hours | Status |
|-----|------|-------|--------|
| 1 | Steps 1-5 (skeleton) | 2 | 🔄 Ready |
| 2 | Test on real file | 3 | ⏳ Next |
| 3-4 | Output formats | 8 | ⏳ Next |
| 5-6 | Language support | 8 | ⏳ Next |
| 7-8 | Testing | 8 | ⏳ Next |
| 9 | Documentation | 4 | ⏳ Next |

**Total**: 33 hours (~7-9 days at 4-5 hours/day)

---

## Overnight Mutation Test Status

**Process**: Running in background (Process ID: f0abe8)
**Command**: `cargo mutants --re "path_validator" --timeout 300 --no-shuffle --jobs 2`
**Output**: `mutation_results/cargo_path_validator_overnight.txt`
**Status**: 🔄 In progress (check next session)

**When Complete**:
- Analyze results for baseline mutation score
- Compare with PMAT mutation testing once implemented
- Document findings in Sprint 60 Phase 2

---

## Quick Commands Cheat Sheet

```bash
# Navigate to project
cd /home/noah/src/paiml-mcp-agent-toolkit/server

# Check compilation
cargo check --bin pmat

# Run command
cargo run --bin pmat -- mutate --help
cargo run --bin pmat -- mutate --target <file>

# Run tests
cargo nextest run handlers::mutate

# Check overnight results
tail -100 ../mutation_results/cargo_path_validator_overnight.txt
```

---

## Contact & Support

**Sprint Documentation**:
- Planning: `docs/sprints/SPRINT-61-PMAT-MUTATE-CLI.md`
- Architecture: `docs/sprints/SPRINT-61-CLI-ARCHITECTURE-FINDINGS.md`
- This guide: `docs/sprints/SPRINT-61-KICKOFF-NEXT-SESSION.md`

**Key Concepts**:
- Mutation testing: Introduce bugs to measure test quality
- Mutation operators: Types of changes (arithmetic, conditional, etc.)
- Mutation score: Percentage of mutants caught by tests

**Goals**:
- Expose PMAT's existing 47-file mutation infrastructure
- Provide CLI command: `pmat mutate --file <path>`
- 5-10x faster than cargo-mutants (AST-based vs recompilation)

---

**Generated**: 2025-10-26 21:15 UTC
**Author**: Claude Code (Sonnet 4.5)
**Sprint**: 61 - Expose PMAT Mutation Testing via CLI Command
**Status**: 🟢 READY FOR IMPLEMENTATION
**Next Session**: Start with Step 1 (read existing commands)
