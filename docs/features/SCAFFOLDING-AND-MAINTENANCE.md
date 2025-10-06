# Project Scaffolding and Maintenance System

**Version**: v2.139.0
**Status**: Production Ready
**Test Coverage**: >90%

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Agent Scaffolding](#agent-scaffolding)
4. [WASM Scaffolding](#wasm-scaffolding)
5. [Project Maintenance](#project-maintenance)
6. [Quality Gates](#quality-gates)
7. [Git Hooks](#git-hooks)
8. [CLI Reference](#cli-reference)
9. [MCP Integration](#mcp-integration)
10. [Testing Guide](#testing-guide)

---

## Overview

The PMAT Scaffolding and Maintenance System provides comprehensive project lifecycle support with extreme quality standards (Complexity <10, Coverage >80%, Mutation Testing).

### Key Features

- **Agent Scaffolding**: Create MCP-compliant agents in <5 minutes
- **WASM Scaffolding**: Generate WebAssembly projects with quality gates
- **Project Maintenance**: Automated roadmap and ticket management
- **Quality Gates**: Enforce code quality standards
- **Git Hooks**: Auto-validation on commit
- **Progress Indicators**: Visual feedback for long operations
- **Enhanced Errors**: Actionable suggestions for common problems

### Supported Templates

**Agent Templates**:
- `basic`: Simple MCP server
- `stateful`: Server with state management
- `hybrid`: Deterministic core + LLM wrapper

**WASM Frameworks**:
- `wasm-labs`: Full-featured WASM development
- `pure-wasm`: Minimal WASM setup

---

## Quick Start

### Install

```bash
cargo install pmat
```

### Create Your First Agent

```bash
# Create a basic agent
pmat scaffold agent --name my_agent --template basic

# Build and test
cd my_agent
cargo build
cargo test

# Run quality gates
pmat quality-gates
```

### Create Your First WASM Project

```bash
# Create WASM project
pmat scaffold wasm --name my_wasm --framework wasm-labs

# Build
cd my_wasm
make wasm-full

# Test locally
python3 -m http.server 8000
# Open http://localhost:8000
```

---

## Agent Scaffolding

### Basic Usage

```bash
# Simple agent
pmat scaffold agent --name my_agent --template basic

# With features
pmat scaffold agent --name my_agent \
  --template stateful \
  --features logging,metrics \
  --quality extreme

# Dry run (preview only)
pmat scaffold agent --name my_agent --template basic --dry-run

# Interactive mode
pmat scaffold agent --interactive
```

### Templates

#### Basic Template

Simple MCP server for straightforward use cases.

```bash
pmat scaffold agent --name echo_agent --template basic
```

**Generated Structure**:
```
echo_agent/
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── handlers.rs
│   └── lib.rs
├── tests/
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

**Features**:
- MCP protocol compliance
- Basic request/response handling
- Unit test scaffolding
- Quality gate configuration

#### Stateful Template

Agent with state management.

```bash
pmat scaffold agent --name data_agent --template stateful
```

**Additional Features**:
- State machine patterns
- Persistence layer
- Event sourcing support
- Transaction handling

#### Hybrid Template

Deterministic core with LLM wrapper.

```bash
pmat scaffold agent --name smart_agent \
  --template hybrid \
  --deterministic-core property-tests \
  --probabilistic-wrapper gpt4
```

**Architecture**:
- Deterministic core (property-tested)
- LLM wrapper (probabilistic)
- Automatic fallback to deterministic
- Confidence threshold management

### Features

Add optional features to your agent:

```bash
pmat scaffold agent --name featured_agent \
  --template basic \
  --features logging,metrics,tracing
```

**Available Features**:
- `logging`: Structured logging with `tracing`
- `metrics`: Metrics collection with Prometheus
- `tracing`: Distributed tracing with OpenTelemetry

### Quality Levels

```bash
# Standard: Coverage ≥60%, Complexity ≤15
pmat scaffold agent --name prototype --quality standard

# Strict (default): Coverage ≥80%, Complexity ≤10
pmat scaffold agent --name production --quality strict

# Extreme: Coverage ≥85%, Mutation testing, Property tests
pmat scaffold agent --name critical --quality extreme
```

### Examples

```bash
# Simple echo agent
pmat scaffold agent --name echo_agent --template basic

# File system agent with logging
pmat scaffold agent --name fs_agent \
  --template stateful \
  --features logging,metrics

# Code assistant with extreme quality
pmat scaffold agent --name code_assistant \
  --template hybrid \
  --features logging,metrics,tracing \
  --quality extreme
```

---

## WASM Scaffolding

### Basic Usage

```bash
# WasmLabs framework
pmat scaffold wasm --name my_wasm --framework wasm-labs

# PureWasm (minimal)
pmat scaffold wasm --name minimal_wasm --framework pure-wasm

# With features
pmat scaffold wasm --name full_wasm \
  --framework wasm-labs \
  --features logging,metrics \
  --quality extreme

# Dry run
pmat scaffold wasm --name test_wasm --framework wasm-labs --dry-run
```

### Frameworks

#### WasmLabs Framework

Full-featured WASM development with quality gates.

```bash
pmat scaffold wasm --name advanced_wasm --framework wasm-labs
```

**Features**:
- Complete build toolchain
- Test infrastructure
- Quality gates (85%+ coverage)
- Makefile automation
- Local development server

**Generated Structure**:
```
advanced_wasm/
├── src/
│   └── lib.rs
├── tests/
├── Makefile
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

#### PureWasm Framework

Minimal WASM setup for simple projects.

```bash
pmat scaffold wasm --name simple_wasm --framework pure-wasm
```

**Features**:
- Minimal dependencies
- Quick builds
- Basic testing
- Essential tooling only

### Development Workflow

```bash
# 1. Create project
pmat scaffold wasm --name my_wasm --framework wasm-labs

cd my_wasm

# 2. Build WASM
make wasm-full

# 3. Serve locally
python3 -m http.server 8000

# 4. Test
make test

# 5. Quality gates
pmat quality-gates
```

---

## Project Maintenance

### Roadmap Management

```bash
# Show roadmap health
pmat maintain roadmap

# Validate roadmap
pmat maintain roadmap --validate

# Fix checkbox status
pmat maintain roadmap --fix

# Custom paths
pmat maintain roadmap \
  --roadmap-path ./docs/ROADMAP.md \
  --tickets-dir ./docs/tickets
```

**Roadmap Health Report**:
```
📊 Roadmap Health Report

Sprint 20: UX Improvements
  Total: 6 tickets
  Complete: 6
  Status: ✅ COMPLETE

Overall Health: 100%
```

### Project Health Checks

```bash
# Quick health check (build only, <10s)
pmat maintain health --quick

# Default (build check only)
pmat maintain health

# Full health check
pmat maintain health --all

# Individual checks
pmat maintain health --check-build --check-tests

# Custom timeout
pmat maintain health --check-build --check-coverage
```

**Health Report**:
```
✅ Project Health Report

✓ Build: Project builds successfully
✓ Tests: All tests passing (606 tests)
✓ Coverage: 84.2%
⚠ Complexity: 3 functions >10
⏭ SATD: Not checked

📊 Summary:
   Total:   4
   Passed:  3
   Warned:  1
   Failed:  0

⚠ Project has warnings
```

### Health Check Options

**Performance**:
- `--quick`: Build only (~10s)
- Default: Build only (~14s)
- `--all`: All checks (~10 minutes)

**Individual Checks**:
- `--check-build`: Cargo check
- `--check-tests`: Run test suite
- `--check-coverage`: Coverage with llvm-cov
- `--check-complexity`: Complexity analysis
- `--check-satd`: SATD detection

---

## Quality Gates

### Configuration

Generate default quality gate configuration:

```bash
# Initialize config
pmat quality-gate init

# Show current config
pmat quality-gate show

# Validate config
pmat quality-gate validate
```

**Generated `.pmat-gates.toml`**:
```toml
[quality]
max_complexity = 10
min_coverage = 80.0
max_satd = 0

[checks]
build = true
test = true
coverage = true
complexity = true
satd = true

[timeouts]
build = 120
test = 600
coverage = 300
```

### Running Quality Gates

```bash
# Run all gates
pmat quality-gate

# With custom config
pmat quality-gate --config .pmat-gates.toml

# JSON output
pmat quality-gate --json

# Specific project
pmat quality-gate --project-dir /path/to/project
```

### Quality Profiles

**Standard**:
- Coverage: ≥60%
- Complexity: ≤15
- Basic testing

**Strict** (Default):
- Coverage: ≥80%
- Complexity: ≤10
- Comprehensive testing

**Extreme**:
- Coverage: ≥85%
- Complexity: ≤8
- Mutation testing
- Property-based testing

---

## Git Hooks

### Installation

```bash
# Install hooks
pmat hooks install

# Install with dry-run
pmat hooks install --dry-run

# Check hook status
pmat hooks status

# Uninstall hooks
pmat hooks uninstall
```

### Available Hooks

**Pre-commit**:
- Run quality gates
- Validate roadmap
- Check code formatting
- Lint code

**Post-commit**:
- Update roadmap status
- Generate health report
- Track ticket progress

### Hook Configuration

Hooks respect `.pmat-gates.toml` configuration and can be bypassed:

```bash
# Bypass hooks temporarily
git commit --no-verify -m "WIP: Quick save"

# Normal commit with hooks
git commit -m "feat: Add new feature"
```

---

## CLI Reference

### Global Flags

```bash
# Verbose output
pmat --verbose <command>

# Quiet mode (errors only)
pmat --quiet <command>

# Color control
pmat --color auto|always|never <command>

# Debug mode
pmat --debug <command>
```

### Scaffold Commands

```bash
# Agent scaffolding
pmat scaffold agent [OPTIONS]
  --name <NAME>              Agent name (required)
  --template <TEMPLATE>      Template type (required)
  --features <FEATURES>      Comma-separated features
  --quality <LEVEL>          Quality level (standard|strict|extreme)
  --output <PATH>            Output directory
  --force                    Overwrite existing
  --dry-run                  Preview only
  --interactive              Interactive mode

# WASM scaffolding
pmat scaffold wasm [OPTIONS]
  --name <NAME>              Project name (required)
  --framework <FRAMEWORK>    Framework (wasm-labs|pure-wasm)
  --features <FEATURES>      Comma-separated features
  --quality <LEVEL>          Quality level
  --output <PATH>            Output directory
  --force                    Overwrite existing
  --dry-run                  Preview only

# List templates
pmat scaffold list-templates
```

### Maintain Commands

```bash
# Roadmap management
pmat maintain roadmap [OPTIONS]
  --roadmap-path <PATH>      Path to ROADMAP.md
  --tickets-dir <PATH>       Path to tickets directory
  --validate                 Validate roadmap
  --health                   Show health report
  --fix                      Fix checkbox status
  --dry-run                  Preview changes
  --format <FORMAT>          Output format (table|json|yaml)

# Health checks
pmat maintain health [OPTIONS]
  --quick                    Quick mode (build only)
  --all                      Run all checks
  --check-build              Check build
  --check-tests              Run tests
  --check-coverage           Check coverage
  --check-complexity         Check complexity
  --check-satd               Check SATD
  --format <FORMAT>          Output format (table|json|yaml)
```

### Hooks Commands

```bash
# Hook management
pmat hooks <SUBCOMMAND>
  install                    Install git hooks
  uninstall                  Remove git hooks
  status                     Show hook status
  --dry-run                  Preview only
```

### Quality Gate Commands

```bash
# Quality gates
pmat quality-gate [OPTIONS]
  init                       Initialize config
  show                       Show current config
  validate                   Validate config
  --config <PATH>            Config file path
  --report                   Generate report
  --json                     JSON output
  --project-dir <PATH>       Project directory
```

---

## MCP Integration

The scaffolding and maintenance system is fully integrated with MCP (Model Context Protocol).

### MCP Server Mode

```bash
# Start MCP server
pmat agent mcp-server

# Use with Claude Code or other MCP clients
```

### Available MCP Tools

#### scaffold_agent

Create a new MCP agent.

**Input Schema**:
```json
{
  "name": "my_agent",
  "template": "basic",
  "features": ["logging", "metrics"],
  "quality": "strict",
  "output": "./my_agent"
}
```

**Example**:
```json
{
  "name": "execute_tool",
  "arguments": {
    "name": "scaffold_agent",
    "arguments": {
      "name": "my_agent",
      "template": "basic"
    }
  }
}
```

#### scaffold_wasm

Create a new WASM project.

**Input Schema**:
```json
{
  "name": "my_wasm",
  "framework": "wasm-labs",
  "features": ["logging"],
  "quality": "extreme"
}
```

#### maintain_roadmap

Manage project roadmap.

**Input Schema**:
```json
{
  "action": "health",
  "roadmap_path": "./ROADMAP.md",
  "tickets_dir": "./docs/tickets"
}
```

#### maintain_health

Check project health.

**Input Schema**:
```json
{
  "mode": "quick",
  "checks": ["build", "tests"]
}
```

### MCP Usage Example

From Claude Code or any MCP client:

```
User: "Create a new agent called data_processor with logging"

Claude: I'll create a new MCP agent for you.
[Calls scaffold_agent tool]

✓ Agent 'data_processor' created successfully
Location: ./data_processor
Next steps:
  cd data_processor
  cargo build
  cargo test
```

---

## Testing Guide

### TDD with assert_cmd

All CLI commands are tested with `assert_cmd` for comprehensive coverage.

#### Testing Scaffold Agent

```rust
use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn test_scaffold_agent_dry_run() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["scaffold", "agent",
                "--name", "test_agent",
                "--template", "basic",
                "--dry-run"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Dry run"))
        .stderr(predicate::str::contains("test_agent"));
}

#[test]
fn test_scaffold_agent_with_features() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["scaffold", "agent",
                "--name", "featured_agent",
                "--template", "basic",
                "--features", "logging,metrics"])
        .assert()
        .success();
}

#[test]
fn test_scaffold_agent_quality_levels() {
    for quality in &["standard", "strict", "extreme"] {
        Command::cargo_bin("pmat")
            .unwrap()
            .args(&["scaffold", "agent",
                    "--name", "quality_test",
                    "--template", "basic",
                    "--quality", quality,
                    "--dry-run"])
            .assert()
            .success();
    }
}
```

#### Testing WASM Scaffold

```rust
#[test]
fn test_scaffold_wasm_frameworks() {
    for framework in &["wasm-labs", "pure-wasm"] {
        Command::cargo_bin("pmat")
            .unwrap()
            .args(&["scaffold", "wasm",
                    "--name", "test_wasm",
                    "--framework", framework,
                    "--dry-run"])
            .assert()
            .success();
    }
}

#[test]
fn test_scaffold_wasm_invalid_framework() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["scaffold", "wasm",
                "--name", "test",
                "--framework", "invalid",
                "--dry-run"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Unknown WASM framework"))
        .stderr(predicate::str::contains("Suggestions"));
}
```

#### Testing Health Checks

```rust
#[test]
fn test_maintain_health_quick() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["maintain", "health", "--quick"])
        .assert()
        .success()
        .stderr(predicate::str::contains("Health Report"));
}

#[test]
fn test_maintain_health_individual_checks() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["maintain", "health", "--check-build"])
        .assert()
        .success();
}
```

#### Testing Error Messages

```rust
#[test]
fn test_error_messages_are_helpful() {
    Command::cargo_bin("pmat")
        .unwrap()
        .args(&["maintain", "roadmap"])
        .current_dir(std::env::temp_dir())
        .assert()
        .failure()
        .stderr(predicate::str::contains("ROADMAP.md"))
        .stderr(predicate::str::contains("Suggestions"));
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_scaffold_names_are_validated(
        name in "[a-z_][a-z0-9_]{0,20}"
    ) {
        Command::cargo_bin("pmat")
            .unwrap()
            .args(&["scaffold", "agent",
                    "--name", &name,
                    "--template", "basic",
                    "--dry-run"])
            .assert()
            .success();
    }
}
```

### Integration Test Suite

Run all 27 integration tests:

```bash
# Run all CLI integration tests
cargo test --release cli_integration_tests::

# Run specific test
cargo test --release cli_integration_tests::test_scaffold_agent_dry_run

# Run with output
cargo test --release cli_integration_tests:: -- --nocapture
```

---

## Troubleshooting

### Common Issues

#### Directory Already Exists

**Error**:
```
ERROR: Directory already exists
  Location: /path/to/my_agent

  Suggestions:
  - Use --force to overwrite existing directory
  - Choose a different output directory with --output
  - Remove the existing directory manually
```

**Solution**:
```bash
# Force overwrite
pmat scaffold agent --name my_agent --template basic --force

# Different location
pmat scaffold agent --name my_agent --template basic --output /different/path
```

#### ROADMAP.md Not Found

**Error**:
```
ERROR: Failed to read ROADMAP.md
  Location: /current/dir/ROADMAP.md
  Reason: File not found

  Suggestions:
  - Run 'pmat maintain roadmap' from project root
  - Ensure you're in the correct directory
  - Check if ROADMAP.md exists in your project
```

**Solution**:
```bash
# Run from project root
cd /path/to/project
pmat maintain roadmap

# Or specify path
pmat maintain roadmap --roadmap-path /path/to/ROADMAP.md
```

#### Health Check Timeout

**Error**: Health check takes too long.

**Solution**:
```bash
# Use quick mode
pmat maintain health --quick

# Or run specific checks only
pmat maintain health --check-build --check-tests
```

---

## Performance

### Benchmarks

| Operation | Time | Notes |
|-----------|------|-------|
| Agent scaffold (dry-run) | <1s | Preview generation |
| Agent scaffold (full) | ~5s | With git init |
| WASM scaffold (dry-run) | <1s | Preview generation |
| WASM scaffold (full) | ~10s | With build setup |
| Health check (quick) | <10s | Build only |
| Health check (default) | ~14s | Build check |
| Health check (all) | ~10min | Full suite |
| Roadmap health | <2s | Validation |

### Optimization Tips

1. **Use Quick Mode**: For fast feedback, use `--quick`
2. **Dry Run First**: Preview with `--dry-run` before creating
3. **Selective Checks**: Use individual flags for specific checks
4. **Quiet Mode**: Use `--quiet` for scripts and CI

---

## Best Practices

### Project Structure

Follow the generated structure:

```
my_project/
├── src/              # Source code
├── tests/            # Integration tests
├── docs/             # Documentation
│   ├── ROADMAP.md    # Sprint tracking
│   └── tickets/      # Ticket details
├── Cargo.toml        # Dependencies
├── README.md         # Project overview
└── .pmat-gates.toml  # Quality config
```

### Quality Standards

1. **Complexity**: Keep all functions <10 CC
2. **Coverage**: Maintain >80% test coverage
3. **SATD**: Zero tolerance for self-admitted technical debt
4. **Documentation**: Document all public APIs

### Git Workflow

1. **Commit Messages**: Use conventional commits
2. **Hooks**: Install git hooks for validation
3. **Roadmap**: Keep roadmap up to date
4. **Tickets**: Link all work to tickets

### Development Cycle

```bash
# 1. Create feature branch
git checkout -b feature/new-feature

# 2. Implement with TDD
cargo test --watch

# 3. Run quality gates
pmat quality-gate

# 4. Update roadmap
pmat maintain roadmap --fix

# 5. Commit with hooks
git commit -m "feat: Add new feature"

# 6. Health check
pmat maintain health --quick
```

---

## Additional Resources

- [Sprint 16 Summary](../sprints/SPRINT-16-SUMMARY.md) - Scaffolding Foundation
- [Sprint 17 Summary](../sprints/SPRINT-17-SUMMARY.md) - Maintenance Engine
- [Sprint 18 Summary](../sprints/SPRINT-18-SUMMARY.md) - Quality Gates
- [Sprint 19 Summary](../sprints/SPRINT-19-SUMMARY.md) - CLI Integration
- [Sprint 20 Summary](../sprints/SPRINT-20-SUMMARY.md) - UX Improvements
- [Dogfooding Results](../dogfooding/SPRINT-19-DOGFOODING-RESULTS.md)
- [Examples](../../examples/README.md)

---

## Support

For issues, questions, or contributions:

- GitHub Issues: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- Documentation: https://paiml.com/docs
- Examples: See `examples/` directory

---

**Version**: v2.139.0
**Last Updated**: 2025-10-06
**Status**: Production Ready
