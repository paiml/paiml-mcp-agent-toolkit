# PMAT Coverage Improve Command - Specification

## Overview
First-class CLI command that autonomously improves test coverage to a target percentage using PMAT's own analysis tools and Extreme TDD methodology.

## Command Syntax
```bash
pmat coverage improve [OPTIONS]

# Examples:
pmat coverage improve --target 95                    # Default: use PMAT tools
pmat coverage improve --target 85 --fast            # Skip mutation testing
pmat coverage improve --target 95 --output report.md
pmat coverage improve --help
```

## Options
- `--target <PERCENT>`: Target coverage percentage (default: 95)
- `--path <PATH>`: Project path to analyze (default: current directory)
- `--output <FILE>`: Write progress report to file
- `--format <FORMAT>`: Output format: text, json, markdown (default: text)
- `--fast`: Skip mutation testing (faster but lower quality)
- `--max-iterations <N>`: Maximum improvement iterations (default: 10)
- `--focus <FILES>`: Focus on specific files/modules (glob patterns)
- `--exclude <FILES>`: Exclude files/modules (glob patterns)
- `--mutation-threshold <PERCENT>`: Minimum mutation score (default: 80)
- `--dry-run`: Show plan without generating tests

## Workflow (PMAT + Extreme TDD)

### Phase 1: Measure Baseline
1. Run `make coverage` to get current coverage %
2. Identify files with <target coverage

### Phase 2: Prioritize Targets (using PMAT tools)
1. **Complexity Analysis**: `pmat analyze complexity`
   - Prioritize high-complexity functions (CC > 10)
2. **SATD Detection**: `pmat analyze satd`
   - Prioritize TODO/FIXME markers indicating missing tests
3. **Dead Code**: `pmat analyze dead-code`
   - Exclude actually-dead code from coverage targets
4. **Git Churn**: `pmat analyze churn`
   - Prioritize frequently-changed files (higher defect risk)

### Phase 3: Generate Property-Based Tests
For each target file:
1. Parse AST to extract function signatures
2. Generate proptest templates based on:
   - Input types (ranges, edge cases)
   - Output types (invariants to check)
   - Complexity (more complex = more test cases)
3. Write tests to file with clear names

### Phase 4: Validate with Mutation Testing
1. Run `cargo nextest run` on new tests
2. Run `cargo mutants` on tested code
3. If mutation score < threshold:
   - Use Five Whys to diagnose weak tests
   - Refine property tests to kill more mutants
4. Repeat until mutation score >= threshold

### Phase 5: Iterate
1. Re-measure coverage with `make coverage`
2. If coverage < target: go to Phase 2
3. If coverage >= target: SUCCESS!
4. Generate completion report

## Output Formats

### Text (Default)
```
Coverage Improvement Report

Baseline: 49.87%
Target:   95.00%
Current:  67.23%

Iteration 1:
  Added tests for ast/parser.rs (+5.2%)
  Added tests for ast/engine.rs (+3.8%)
  Mutation score: 72% (below 80% threshold)

Iteration 2:
  Refined tests in ast/parser.rs (+1.3% mutation)
  Coverage: 58.3% -> 67.2%

Status: In Progress (3 more iterations needed)
```

### JSON
```json
{
  "baseline_coverage": 49.87,
  "target_coverage": 95.0,
  "current_coverage": 67.23,
  "iterations": [
    {
      "iteration": 1,
      "tests_added": ["ast/parser.rs", "ast/engine.rs"],
      "coverage_gain": 9.0,
      "mutation_score": 72.0
    }
  ],
  "status": "in_progress"
}
```

### Markdown
Full report with:
- Coverage progress chart
- Test quality metrics
- Mutation testing results
- Recommendations for manual review

## Implementation Plan

### 1. Service Layer
File: `server/src/services/coverage_improvement_service.rs`

```rust
pub struct CoverageImprovementService {
    complexity_analyzer: ComplexityAnalyzer,
    satd_detector: SatdDetector,
    dead_code_analyzer: DeadCodeAnalyzer,
    mutation_runner: MutationRunner,
}

impl CoverageImprovementService {
    pub async fn improve_coverage(&self, config: Config) -> Result<Report> {
        // Phase 1: Measure baseline
        let baseline = self.measure_coverage(&config.project_path).await?;

        // Phase 2-5: Iterate until target reached
        while current < target && iterations < max_iterations {
            let targets = self.prioritize_targets(&config).await?;
            let tests = self.generate_property_tests(&targets).await?;
            let mutation_score = self.validate_tests(&tests).await?;
            current = self.measure_coverage(&config.project_path).await?;
        }

        Ok(report)
    }
}
```

### 2. CLI Handler
File: `server/src/cli/handlers/coverage_improve_handler.rs`
- Parse command options
- Create service instance
- Stream progress to user
- Handle output formatting

### 3. CLI Command
File: `server/src/cli/commands.rs`

Add to main `Commands` enum:
```rust
/// Improve test coverage to target percentage
#[command(visible_aliases = &["improve-coverage", "cov-improve"])]
CoverageImprove {
    /// Target coverage percentage
    #[arg(long, short = 't', default_value = "95")]
    target: f64,

    /// Project path
    #[arg(long, short = 'p', default_value = ".")]
    project_path: PathBuf,

    // ... more options ...
}
```

### 4. Command Dispatcher
File: `server/src/cli/command_dispatcher.rs`
Wire `Commands::CoverageImprove` to handler

## Generalization for ANY Rust Project

This methodology works for any Rust project because:
1. **Tool-agnostic detection**: Uses cargo-llvm-cov (standard Rust tool)
2. **AST parsing**: Works with any valid Rust code
3. **Property testing**: Proptest is universal for Rust
4. **Mutation testing**: cargo-mutants works on any Rust codebase
5. **PMAT analysis**: Complexity/SATD/dead-code are language-agnostic concepts

To apply to another project:
```bash
cd /path/to/any/rust/project
pmat coverage improve --target 85
# Autonomous improvement using same PMAT + Extreme TDD workflow
```

## Success Criteria
1. Command `pmat coverage improve` exists as first-class CLI command
2. Uses PMAT's own tools (complexity, SATD, dead code, churn)
3. Generates property-based tests (not basic assertions)
4. Validates with mutation testing (>=80% mutation score)
5. Reaches target coverage (default 95%)
6. Works on PMAT itself (dogfooding proof)
7. Works on ANY Rust project (generalizable)

## References
- CLAUDE.md: Extreme TDD requirement (95% coverage + mutation testing)
- Existing: `pmat analyze incremental-coverage` (measures, doesn't improve)
- Existing: `server/src/services/incremental_coverage_analyzer.rs`
- Existing: `server/src/cli/handlers/incremental_coverage_handler.rs`
- Sprint 38+: Property-based testing infrastructure
- cargo-mutants: Mutation testing tool integration

## Rationale (Toyota Way - Jidoka)
Coverage improvement should be built-in quality (Jidoka), not manual labor. This command embodies:
- **Automation**: No manual test writing needed
- **Evidence-Based**: Uses PMAT analysis to prioritize
- **Quality-First**: Mutation testing validates test quality
- **Generalizable**: Works on any Rust project
- **Dogfooded**: Proved on PMAT itself
