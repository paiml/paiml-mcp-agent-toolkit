# GitHub Actions Workflows

This directory contains the CI/CD workflows for the MCP Agent Toolkit project.

## Workflow Architecture

### 🎯 Main Orchestrator (`main.yml`)
**The primary workflow that gates all other checks**
- **Triggers**: Push to main branches, Pull requests
- **Staged Execution**:
  1. **Stage 1: CI** (Must pass first)
     - Format checking
     - Linting  
     - Type checking
     - Tests with coverage
     - Build
  2. **Stage 2: Additional Checks** (Only run if CI passes)
     - Security audit
     - Code quality
     - Benchmarks
     - Dependency analysis

This staged approach prevents wasting CI resources when basic checks are failing.

## Individual Workflows

### 🔄 Continuous Integration (`ci.yml`)
- **Triggers**: Manual dispatch only (orchestrated by main.yml)
- **Jobs**:
  - Lint (rustfmt, clippy, deno)
  - Test with coverage reporting
  - Build binary
  - Security audit

### 📦 Release Workflows
- **`release.yml`**: Triggered by version tags (v*)
- **`cargo-dist.yml`**: Manages binary distribution
- **`automated-release.yml`**: Handles automated releases
- **`auto-tag-release.yml`**: Creates version tags

### 🔐 Dependencies (`dependencies.yml`)
- **Triggers**: Weekly schedule, Manual dispatch
- **Jobs**:
  - Updates Rust dependencies
  - Runs security audit
  - Creates PRs for updates
  - Creates issues for vulnerabilities

### 📊 Code Quality (`code-quality.yml`)
- **Triggers**: Manual dispatch only (orchestrated by main.yml)
- **Jobs**:
  - Code coverage analysis (60% minimum)
  - Complexity metrics
  - Documentation checks

### 🧪 Property Testing (`property-tests.yml` & `property-test-validation.yml`)
- **property-test-validation.yml** (NEW - Sprint 88)
  - **Triggers**: Push/PR on Rust changes, Daily schedule, Manual dispatch
  - **Purpose**: Enforces 80% property test file coverage threshold
  - **Jobs**:
    - Counts files with property tests (target: 431/539 files)
    - Validates all property tests compile and pass
    - Performance monitoring (5-minute timeout)
    - PR commenting with coverage status
- **property-tests.yml**
  - **Triggers**: Push/PR on Rust changes, Manual dispatch
  - **Jobs**:
    - Comprehensive property test execution
    - Coverage analysis with tarpaulin
    - Platform matrix testing (Linux, macOS, Windows)
    - QuickCheck fuzzing for PRs

### 🚀 Performance (`benchmark.yml`)
- **Triggers**: Manual dispatch only (orchestrated by main.yml)
- **Jobs**:
  - Performance benchmarks
  - Memory usage analysis
  - Startup time measurements

### ✅ PR Checks (`pr-checks.yml`)
- **Triggers**: Pull request events
- **Jobs**:
  - PR title validation (conventional commits)
  - Branch naming checks
  - PR size analysis

## Best Practices

1. **Always use the root Makefile** - Never use `cd server && make`
2. **Check main.yml first** - Most workflows are orchestrated through it
3. **CI gates everything** - No point running other checks if CI fails
4. **Manual dispatch available** - All workflows can be run manually if needed