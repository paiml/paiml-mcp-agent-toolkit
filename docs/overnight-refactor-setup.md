# Automated Overnight Code Repair Setup

## Overview

The automated overnight code repair system uses a state machine to execute an 8-12 hour autonomous refactoring pipeline. It generates an actionable checklist while applying fixes via the RefactorStateMachine.

## Components Created

### 1. Scripts

#### `/scripts/overnight-refactor.sh`
- Main orchestration script for overnight repair
- Configures and launches the refactor state machine
- Monitors progress and handles interrupts
- Creates checkpoints for resumability

#### `/scripts/pre-release-analysis.sh`
- Comprehensive pre-release quality analysis
- Generates detailed checklist in `docs/bugs/prepare-release.md`
- Identifies SATD, complexity, and other quality issues
- Provides actionable commands for fixes

### 2. Configuration

The system uses a JSON configuration that defines:
- **State Machine**: Transitions between analyzing, planning, executing, validating states
- **Metrics Thresholds**: Complexity limits, TDG scores, SATD policies
- **Operations**: Extract function, flatten nesting, remove dead code, fix SATD
- **Validation**: Test execution, coverage checks, lint verification

### 3. Release Checklist

Generated at `docs/bugs/prepare-release.md` with:
- 🔴 Critical issues (release blockers)
- 🟡 High priority issues  
- ✅ Pre-release checklist items
- 📊 Repository statistics
- 🤖 Automated fix commands

## Current Status

### ✅ Completed
- Zero tolerance SATD policy implemented
- Refactor engine with state machine
- Excellence tracking metrics
- Scripts for analysis and repair

### ⚠️ Pending Issues
1. **TRACKED Comments**: 20+ locations need implementation
   - `deep_context_orchestrator.rs`: 6 instances
   - `unified_ast_engine.rs`: 5 instances
   - `ast_python.rs`: 4 instances
   - Others in code_intelligence, dead_code_analyzer

2. **High Complexity**: 
   - `cli/mod.rs`: Functions up to 75 complexity
   - Needs modularization

3. **Integration**:
   - Complete AST integration in deep context
   - Real metrics instead of placeholders

## Running the System

### Quick Analysis
```bash
# Generate release checklist
./scripts/pre-release-analysis.sh

# View checklist
cat docs/bugs/prepare-release.md
```

### Overnight Repair (When Fully Implemented)
```bash
# Start autonomous repair
./scripts/overnight-refactor.sh

# Monitor progress
tail -f refactor_overnight.log | grep -E "STATE:|FIXED:|ERROR:"

# Check state
cat .refactor_state/current_state.json

# Safe interrupt
kill -SIGUSR1 $(cat .refactor_state/refactor.pid)
```

### Manual Fixes Available Now
```bash
# Check for TRACKED comments
grep -r "TRACKED:" server/src/ --include="*.rs" | wc -l

# Analyze complexity
cargo run --release -- analyze complexity . --max-cyclomatic 20

# Check SATD
cargo run --release -- analyze satd .

# Run refactor interactively (once serve mode is complete)
cargo run --release -- refactor interactive server/src/cli/mod.rs
```

## State Machine Design

```
ANALYZING → PLANNING → EXECUTING → VALIDATING
    ↑                                   ↓
    ←───────── REVERTING ←──────────────┘
```

### States:
1. **ANALYZING**: Collects metrics (SATD, TDG, complexity, dead code)
2. **PLANNING**: Prioritizes violations, generates fix sequence
3. **EXECUTING**: Applies refactorings with AST transforms
4. **VALIDATING**: Runs tests, checks coverage
5. **REVERTING**: Rolls back failed transformations

### Features:
- Checkpoint every 5 minutes
- Resume from interruption
- Auto-commit with [skip ci]
- Parallel processing (8 threads)
- Memory limit (16GB)
- Max runtime (12 hours)

## Next Steps

1. **Complete TRACKED implementations** in deep_context_orchestrator
2. **Test refactor serve mode** with small targets
3. **Profile performance** of state machine
4. **Add progress visualization** dashboard
5. **Integrate with CI/CD** for automated quality gates

The system is designed to run autonomously overnight, fixing code quality issues while developers sleep, and providing a comprehensive report of changes made.