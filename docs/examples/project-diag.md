# Project Diagnostics Example Guide

This guide demonstrates how to use `pmat project-diag` for comprehensive Rust project health assessment.

## Overview

The `project-diag` command runs 20 diagnostic checks across 5 categories, providing an overall health score and actionable recommendations. It matches the functionality of lltop Tab 8 (Project Diagnostics) but works on any Rust project.

## Quick Start

```bash
# Run diagnostics on current directory
pmat project-diag

# Specify a path
pmat project-diag --path /path/to/rust/project

# Filter by category
pmat project-diag --category build

# Output formats
pmat project-diag --format summary   # Human-readable (default)
pmat project-diag --format json      # For CI/CD
pmat project-diag --format markdown  # For documentation
pmat project-diag --format andon     # Toyota Way Andon board
```

## Command Aliases

```bash
pmat project-diag  # Full command
pmat diag          # Short alias
pmat diagnose      # Alternative alias
```

## The 20 Diagnostic Checks

### Cargo Config (6 checks, 30 points max)

| Check | Description | Points |
|-------|-------------|--------|
| Edition 2021+ | Validates Rust edition is 2021 or 2024 | 5 |
| Resolver v2 | Checks for resolver = "2" or edition 2021+ | 5 |
| Dependencies <= 50 | Counts dependencies, warns if >50 | 5 |
| LTO Enabled | Checks for lto = true/thin/fat in [profile.release] | 5 |
| Workspace Lints | Validates [workspace.lints] configuration | 5 |
| Workspace Deps | Checks for [workspace.dependencies] | 5 |

### Dependencies (3 checks, 15 points max)

| Check | Description | Points |
|-------|-------------|--------|
| Target Dir <= 10GB | Measures target/ directory size | 5 |
| Cargo.lock Present | Ensures reproducible builds | 5 |
| Audit Config | Checks for deny.toml or audit.toml | 5 |

### Build Performance (4 checks, 20 points max)

| Check | Description | Points |
|-------|-------------|--------|
| Cargo Config | Validates .cargo/config.toml exists | 5 |
| Incremental Builds | Checks incremental compilation settings | 5 |
| Codegen Units | Validates codegen-units = 1 for release | 5 |
| Build System | Checks for Makefile/justfile/build.rs | 5 |

### Code Quality (4 checks, 20 points max)

| Check | Description | Points |
|-------|-------------|--------|
| Clippy Config | Checks for .clippy.toml or [lints.clippy] | 5 |
| Rustfmt Config | Validates rustfmt.toml exists | 5 |
| Tests Present | Checks for tests/ directory and #[test] | 5 |
| README | Validates README.md exists and has content | 5 |

### Advanced (3 checks, 15 points max)

| Check | Description | Points |
|-------|-------------|--------|
| MSRV Defined | Checks for rust-version in Cargo.toml | 5 |
| Benchmarks | Validates benches/ directory and Criterion | 5 |
| CI Configured | Checks for .github/workflows/ or .gitlab-ci.yml | 5 |

## Health Status Interpretation

| Status | Score Range | Meaning |
|--------|-------------|---------|
| GREEN | >= 85% | Production ready, all critical checks pass |
| YELLOW | 60-84% | Some issues need attention before release |
| RED | < 60% | Critical issues must be resolved |

## Example Output

### Summary Format (Default)

```
Project Diagnostics: /path/to/project
==================================================

Overall: [YELLOW] 78.0/100.0 (78.0%)

Cargo Config [5/6]
Dependencies [2/3]
Build Performance [4/4]
Code Quality [2/4]
Advanced [1/3]

Checks:
--------------------------------------------------
[OK]   Edition 2021+ - Edition: 2024
[OK]   Resolver v2 - Using resolver v2
[WARN] Dependencies <= 50 - 52 dependencies (2 over limit)
[OK]   LTO Enabled - lto = "thin"
[SKIP] Workspace Lints - Single-crate project
[SKIP] Workspace Deps - Single-crate project
[OK]   Target Dir <= 10GB - 2.3 GB
[OK]   Cargo.lock Present - Present
[FAIL] Audit Config - No deny.toml or audit.toml found
...
```

### Andon Board Format

```
+=================================================================+
|                    PROJECT DIAGNOSTICS                          |
|                      (Andon Board)                              |
+=================================================================+
|  Score: [##############################----------] 75.0%        |
+=================================================================+
|  [YELLOW] Cargo Config         5/6 checks passed               |
|  [GREEN]  Dependencies         3/3 checks passed               |
|  [GREEN]  Build Performance    4/4 checks passed               |
|  [YELLOW] Code Quality         2/4 checks passed               |
|  [RED]    Advanced             0/3 checks passed               |
+=================================================================+
|  ANDON CORD TRIGGERED - Issues require attention:               |
|    - MSRV Defined                                               |
|    - Benchmarks                                                 |
|    - CI Configured                                              |
+=================================================================+
```

### JSON Format (for CI/CD)

```json
{
  "project_path": "/path/to/project",
  "percentage": 78.0,
  "earned": 78.0,
  "max": 100.0,
  "health_status": "Yellow",
  "categories": {
    "cargo_config": {"passed": 5, "total": 6, "status": "Yellow"},
    "dependencies": {"passed": 2, "total": 3, "status": "Yellow"},
    "build_performance": {"passed": 4, "total": 4, "status": "Green"},
    "code_quality": {"passed": 2, "total": 4, "status": "Yellow"},
    "advanced": {"passed": 1, "total": 3, "status": "Red"}
  },
  "checks": [...]
}
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Project Diagnostics
on: [push, pull_request]

jobs:
  diagnostics:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Run Project Diagnostics
        run: |
          pmat project-diag --format json --output diag.json

          # Parse and check score
          SCORE=$(jq '.percentage' diag.json)
          if (( $(echo "$SCORE < 80" | bc -l) )); then
            echo "Diagnostics score $SCORE is below 80%"
            exit 1
          fi

      - name: Upload Report
        uses: actions/upload-artifact@v4
        with:
          name: diagnostics-report
          path: diag.json
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
pmat project-diag --failures-only --quiet
if [ $? -ne 0 ]; then
    echo "Project diagnostics found critical issues"
    exit 1
fi
```

## Category Filtering

Focus on specific categories:

```bash
# Cargo configuration only
pmat project-diag --category cargo

# Dependency checks only
pmat project-diag --category deps

# Build performance only
pmat project-diag --category build

# Code quality only
pmat project-diag --category quality

# Advanced checks only
pmat project-diag --category advanced
```

## Failures Only Mode

Show only failing checks:

```bash
pmat project-diag --failures-only
```

## Save to File

```bash
# Save JSON report
pmat project-diag --format json --output diag.json

# Save Markdown report
pmat project-diag --format markdown --output DIAGNOSTICS.md
```

## Toyota Way Integration

The `--format andon` option displays an Andon-style status board following Toyota Production System principles:

- **GREEN**: Line running smoothly (production ready)
- **YELLOW**: Minor issues, continue with caution
- **RED**: Stop the line, address issues immediately

This visualization helps teams quickly identify project health at a glance, following the Jidoka (built-in quality) principle.

## Relationship to rust-project-score

`project-diag` provides a quick, focused assessment of 20 common project configuration checks. For comprehensive quality scoring (159 points across 10 categories), use:

```bash
pmat rust-project-score --full
```

The two commands complement each other:
- `project-diag`: Quick configuration health check (20 checks)
- `rust-project-score`: Full quality assessment (10 scorers, 159 points)

## Running the Example

```bash
# Run the example demo
cargo run --example project_diag_demo

# Run diagnostics on PMAT itself
pmat project-diag
```
