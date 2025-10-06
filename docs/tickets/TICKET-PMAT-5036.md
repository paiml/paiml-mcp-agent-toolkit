# TICKET-PMAT-5036: Create Example Scaffolded Projects

**Status**: GREEN
**Priority**: P1
**Complexity**: 2
**Estimated Time**: 30 minutes
**Dependencies**: TICKET-PMAT-5030, TICKET-PMAT-5031
**Sprint**: Sprint 19 - CLI Integration & Dogfooding

## Objective

Create example documentation and guides showing how to use the scaffolding commands to create agent and WASM projects. Since we can't actually run the commands without a built binary, we'll document the expected workflow and create README files that developers can follow.

## Success Criteria

- [ ] Example agent project documentation created
- [ ] Example WASM project documentation created
- [ ] Getting started guide for scaffolding
- [ ] Clear instructions for both frameworks
- [ ] Documentation shows expected project structure
- [ ] All quality gates pass (documentation quality)

## Current State

**Available Commands:**
- `pmat scaffold agent` - Create MCP agents with various templates
- `pmat scaffold wasm` - Create WASM projects (WasmLabs or PureWasm)

**What We'll Document:**
- How to scaffold new projects
- What gets generated
- Expected project structure
- Next steps after scaffolding
- Quality gates and testing

## Documentation Plan

### File 1: examples/README.md

Main examples directory README explaining all available examples.

### File 2: examples/agent-scaffolding.md

Step-by-step guide for creating agents with examples.

### File 3: examples/wasm-scaffolding.md

Step-by-step guide for creating WASM projects with examples.

### File 4: examples/scaffolding-quickstart.md

Quick start guide for developers who want to get started fast.

## Implementation

### examples/README.md

```markdown
# PMAT Examples

This directory contains examples and guides for using PMAT's scaffolding commands.

## Quick Links

- [Agent Scaffolding Guide](./agent-scaffolding.md) - Create MCP agents
- [WASM Scaffolding Guide](./wasm-scaffolding.md) - Create WASM projects
- [Quick Start Guide](./scaffolding-quickstart.md) - Get started in 5 minutes

## Available Scaffolding Commands

### 1. Agent Scaffolding

Create Model Context Protocol (MCP) agents with built-in quality gates.

```bash
pmat scaffold agent --name my-agent --template basic
```

**Features:**
- MCP server setup
- State machine patterns
- Quality gates pre-configured
- Testing infrastructure
- Multiple templates available

**Templates:**
- `basic` - Simple MCP server
- `stateful` - Server with state management
- `hybrid` - Deterministic core + LLM wrapper

### 2. WASM Scaffolding

Create WebAssembly projects optimized for local development.

```bash
pmat scaffold wasm --name my-wasm --framework wasm-labs
```

**Features:**
- WasmLabs or PureWasm frameworks
- Local development setup
- Quality gates (85%+ coverage, mutation testing)
- Makefile automation
- localhost:8000 testing

**Frameworks:**
- `wasm-labs` - Full-featured WASM development
- `pure-wasm` - Minimal WASM setup

## Project Structure

All scaffolded projects follow PMAT quality standards:
- ✅ Extreme TDD (85%+ coverage)
- ✅ Complexity <10
- ✅ Mutation testing
- ✅ Property-based testing
- ✅ Quality gate enforcement

## Next Steps

1. Choose your project type (agent or WASM)
2. Read the relevant guide
3. Run the scaffold command
4. Follow the generated README
5. Start building!

## Support

- [PMAT Documentation](../README.md)
- [Quality Gates](../docs/quality-gates.md)
- [Dogfooding Report](../docs/dogfooding/SPRINT-19-FINDINGS.md)
```

### examples/agent-scaffolding.md

```markdown
# Agent Scaffolding Guide

Create Model Context Protocol (MCP) agents with PMAT scaffolding.

## Quick Start

```bash
# Basic agent
pmat scaffold agent --name my-agent --template basic

# Stateful agent with features
pmat scaffold agent --name smart-agent \
  --template stateful \
  --features logging,metrics \
  --quality extreme

# Dry run to preview
pmat scaffold agent --name test-agent --template basic --dry-run
```

## Command Options

### Required Arguments

- `--name <NAME>`: Agent name (kebab-case recommended)
- `--template <TEMPLATE>`: Template type (basic, stateful, hybrid)

### Optional Arguments

- `--features <FEATURES>`: Comma-separated features (logging, metrics, tracing)
- `--quality <LEVEL>`: Quality level (standard, strict, extreme)
- `--output <PATH>`: Output directory (default: current directory)
- `--force`: Overwrite existing directory
- `--dry-run`: Preview without creating files

## Templates

### 1. Basic Agent

Simple MCP server for straightforward use cases.

**Generated Structure:**
```
my-agent/
├── src/
│   ├── main.rs              # Entry point
│   ├── server.rs            # MCP server implementation
│   ├── handlers.rs          # Request handlers
│   └── lib.rs               # Library exports
├── tests/
│   ├── integration.rs       # Integration tests
│   └── property.rs          # Property-based tests
├── Cargo.toml               # Dependencies and metadata
├── README.md                # Getting started guide
└── .pmat-gates.toml         # Quality gate configuration
```

**Next Steps:**
```bash
cd my-agent
cargo build
cargo test
pmat quality-gates
```

### 2. Stateful Agent

Agent with state management and persistence.

**Additional Features:**
- State machine patterns
- Persistence layer
- Event sourcing
- Transaction support

**Generated Structure:**
```
smart-agent/
├── src/
│   ├── main.rs
│   ├── server.rs
│   ├── state/
│   │   ├── machine.rs       # State machine
│   │   ├── persistence.rs   # State persistence
│   │   └── mod.rs
│   ├── handlers.rs
│   └── lib.rs
├── tests/
│   ├── integration.rs
│   ├── property.rs
│   └── state_tests.rs       # State machine tests
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

### 3. Hybrid Agent

Deterministic core with LLM wrapper for complex reasoning.

**Architecture:**
- Deterministic core (property-tested)
- LLM wrapper (probabilistic)
- Fallback to deterministic
- Confidence thresholds

**Generated Structure:**
```
hybrid-agent/
├── src/
│   ├── main.rs
│   ├── core/                # Deterministic core
│   │   ├── logic.rs
│   │   ├── verification.rs
│   │   └── mod.rs
│   ├── wrapper/             # LLM wrapper
│   │   ├── llm_client.rs
│   │   ├── fallback.rs
│   │   └── mod.rs
│   ├── server.rs
│   └── lib.rs
├── tests/
│   ├── core_property.rs     # Core property tests
│   ├── wrapper_tests.rs
│   └── integration.rs
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

## Features

### Logging

Adds structured logging with tracing.

```bash
pmat scaffold agent --name my-agent --template basic --features logging
```

**Includes:**
- `tracing` crate
- `tracing-subscriber` setup
- Log level configuration
- Span instrumentation

### Metrics

Adds metrics collection and reporting.

```bash
pmat scaffold agent --name my-agent --template basic --features metrics
```

**Includes:**
- `metrics` crate
- Prometheus exporter
- Counter, gauge, histogram macros
- /metrics endpoint

### Tracing

Adds distributed tracing support.

```bash
pmat scaffold agent --name my-agent --template basic --features tracing
```

**Includes:**
- OpenTelemetry integration
- Trace context propagation
- Jaeger exporter
- Span instrumentation

## Quality Levels

### Standard

Basic quality gates for rapid prototyping.

- Coverage: ≥60%
- Complexity: ≤15
- Basic tests

### Strict (Default)

Production-ready quality standards.

- Coverage: ≥80%
- Complexity: ≤10
- Integration tests
- Property tests

### Extreme

Maximum quality for critical systems.

- Coverage: ≥85%
- Complexity: ≤10
- Mutation testing
- Property-based testing
- Formal verification

## Workflow

1. **Scaffold**: `pmat scaffold agent --name my-agent --template basic`
2. **Build**: `cd my-agent && cargo build`
3. **Test**: `cargo test`
4. **Quality**: `pmat quality-gates`
5. **Develop**: Add your agent logic
6. **Iterate**: Test, refactor, improve

## Examples

### Simple Echo Agent

```bash
pmat scaffold agent --name echo-agent --template basic
cd echo-agent

# Edit src/handlers.rs to implement echo logic
# Run tests
cargo test

# Check quality gates
pmat quality-gates
```

### File System Agent

```bash
pmat scaffold agent \
  --name fs-agent \
  --template stateful \
  --features logging,metrics \
  --quality strict

cd fs-agent
# Implement file system operations in handlers
# State machine tracks file operations
```

### Code Assistant Agent

```bash
pmat scaffold agent \
  --name code-assistant \
  --template hybrid \
  --features logging,metrics,tracing \
  --quality extreme

cd code-assistant
# Deterministic core: syntax validation, formatting
# LLM wrapper: code generation, suggestions
```

## Troubleshooting

### Directory Already Exists

```bash
# Use --force to overwrite
pmat scaffold agent --name my-agent --template basic --force

# Or specify different output directory
pmat scaffold agent --name my-agent --template basic --output /tmp/agents
```

### Unknown Template

```bash
# List available templates
pmat scaffold list-templates

# Verify template name
pmat scaffold agent --name test --template TEMPLATE_NAME --dry-run
```

## Next Steps

- Read generated README.md in your project
- Implement agent handlers
- Run `pmat quality-gates` regularly
- Add tests as you develop
- Use `pmat hooks install` for pre-commit gates
```

### examples/wasm-scaffolding.md

```markdown
# WASM Scaffolding Guide

Create WebAssembly projects with PMAT scaffolding.

## Quick Start

```bash
# WasmLabs project
pmat scaffold wasm --name my-wasm --framework wasm-labs

# PureWasm project
pmat scaffold wasm --name minimal-wasm --framework pure-wasm

# With features and extreme quality
pmat scaffold wasm --name quality-wasm \
  --framework wasm-labs \
  --features logging,metrics \
  --quality extreme

# Dry run to preview
pmat scaffold wasm --name test --framework wasm-labs --dry-run
```

## Command Options

### Required Arguments

- `--name <NAME>`: Project name (kebab-case recommended)
- `--framework <FRAMEWORK>`: Framework (wasm-labs, pure-wasm)

### Optional Arguments

- `--features <FEATURES>`: Comma-separated features (logging, metrics, tracing)
- `--quality <LEVEL>`: Quality level (standard, strict, extreme)
- `--output <PATH>`: Output directory (default: current directory)
- `--force`: Overwrite existing directory
- `--dry-run`: Preview without creating files

## Frameworks

### WasmLabs Framework

Full-featured WASM development for local environments.

**Project Structure:**
```
my-wasm/
├── src/
│   ├── lib.rs               # Main WASM module
│   └── utils.rs             # Utility functions
├── tests/
│   ├── web.rs               # Browser tests
│   └── property.rs          # Property tests
├── www/
│   ├── index.html           # Test page
│   ├── index.js             # WASM loader
│   └── styles.css           # Styling
├── Cargo.toml               # Rust dependencies
├── Makefile                 # Build automation
├── README.md                # Getting started
└── .pmat-gates.toml         # Quality gates
```

**Features:**
- wasm-bindgen for JS interop
- wasm-pack for building
- Local development server
- Browser testing with wasm-pack test
- Extreme TDD (85%+ coverage)
- Mutation testing

**Build Commands:**
```bash
cd my-wasm

# Full build with quality gates
make wasm-full

# Serve locally
python3 -m http.server 8000
# Visit http://localhost:8000

# Run tests
make wasm-test

# Run quality gates
make wasm-quality
```

### PureWasm Framework

Minimal WASM setup for learning or simple projects.

**Project Structure:**
```
minimal-wasm/
├── src/
│   └── lib.rs               # Core WASM module
├── tests/
│   └── tests.rs             # Basic tests
├── Cargo.toml
├── README.md
└── .pmat-gates.toml
```

**Features:**
- Minimal dependencies
- Direct wasm32 target
- Fast builds
- Educational focus

**Build Commands:**
```bash
cd minimal-wasm

# Build WASM
cargo build --target wasm32-unknown-unknown --release

# Run tests
cargo test

# Quality check
pmat quality-gates
```

## Features

### Logging

Adds console logging for WASM.

```bash
pmat scaffold wasm --name my-wasm --framework wasm-labs --features logging
```

**Includes:**
- `console_log!` macro
- Error logging
- Debug output
- Browser console integration

### Metrics

Adds performance metrics collection.

```bash
pmat scaffold wasm --name my-wasm --framework wasm-labs --features metrics
```

**Includes:**
- Performance.now() bindings
- Timing instrumentation
- Memory tracking
- JS metrics export

### Tracing

Adds execution tracing for debugging.

```bash
pmat scaffold wasm --name my-wasm --framework wasm-labs --features tracing
```

**Includes:**
- Function call tracing
- Execution timeline
- Browser DevTools integration
- Flame graph support

## Quality Levels

### Standard

Basic quality for prototypes.

- Coverage: ≥60%
- Complexity: ≤15
- Basic wasm-pack tests

### Strict (Default)

Production-ready WASM.

- Coverage: ≥80%
- Complexity: ≤10
- Browser and headless tests
- Property tests

### Extreme

Mission-critical WASM.

- Coverage: ≥85%
- Complexity: ≤10
- Mutation testing
- Property-based testing
- Memory leak detection
- Performance benchmarks

## Local Development Workflow

### WasmLabs Workflow

1. **Scaffold Project**
```bash
pmat scaffold wasm --name my-app --framework wasm-labs
cd my-app
```

2. **Build WASM**
```bash
make wasm-full
```

3. **Start Dev Server**
```bash
python3 -m http.server 8000
```

4. **Open Browser**
```
http://localhost:8000
```

5. **Develop**
- Edit src/lib.rs
- Save changes
- Run `make wasm-full`
- Refresh browser

6. **Test**
```bash
make wasm-test
make wasm-quality
```

### PureWasm Workflow

1. **Scaffold Project**
```bash
pmat scaffold wasm --name minimal --framework pure-wasm
cd minimal
```

2. **Build**
```bash
cargo build --target wasm32-unknown-unknown --release
```

3. **Test**
```bash
cargo test
pmat quality-gates
```

## Example Projects

### Hello WASM

```bash
pmat scaffold wasm --name hello-wasm --framework wasm-labs
cd hello-wasm

# src/lib.rs will have:
# - greet() function
# - WASM bindings
# - Basic tests

# Build and test
make wasm-full
make wasm-test

# Serve
python3 -m http.server 8000
```

### Calculator WASM

```bash
pmat scaffold wasm \
  --name calc-wasm \
  --framework wasm-labs \
  --features logging \
  --quality strict

cd calc-wasm

# Implement calculator operations
# Add comprehensive tests
# Verify quality gates
make wasm-quality
```

### Game Engine WASM

```bash
pmat scaffold wasm \
  --name game-wasm \
  --framework wasm-labs \
  --features logging,metrics,tracing \
  --quality extreme

cd game-wasm

# High-performance game logic
# Extensive testing
# Mutation testing
# Performance benchmarks
```

## Troubleshooting

### wasm-pack Not Found

```bash
cargo install wasm-pack
```

### WASM Build Fails

```bash
# Ensure wasm32 target installed
rustup target add wasm32-unknown-unknown

# Try clean build
cargo clean
make wasm-full
```

### Tests Failing in Browser

```bash
# Run headless tests
wasm-pack test --headless --firefox

# Or use Chrome
wasm-pack test --headless --chrome
```

### localhost:8000 Not Working

```bash
# Ensure you're in project root
cd /path/to/my-wasm

# Try different port
python3 -m http.server 9000
```

## Deployment (Out of MVP Scope)

**Note**: Deployment is intentionally excluded from MVP scope. Focus is on:
- ✅ Perfect local development
- ✅ Extreme quality gates
- ✅ Team collaboration
- ❌ Not: S3, CloudFront, CI/CD, production infrastructure

For deployment, see future documentation after MVP completion.

## Next Steps

- Read generated README.md in your project
- Implement WASM functions
- Run `make wasm-full` regularly
- Test in browser at localhost:8000
- Use `pmat quality-gates` for quality checks
- Install hooks: `pmat hooks install`
```

### examples/scaffolding-quickstart.md

```markdown
# Scaffolding Quick Start

Get started with PMAT scaffolding in 5 minutes.

## Agent in 60 Seconds

```bash
# Create agent
pmat scaffold agent --name quick-agent --template basic

# Build and test
cd quick-agent
cargo build
cargo test

# Quality check
pmat quality-gates

# Done! 🎉
```

## WASM in 60 Seconds

```bash
# Create WASM project
pmat scaffold wasm --name quick-wasm --framework wasm-labs

# Build
cd quick-wasm
make wasm-full

# Serve
python3 -m http.server 8000

# Open browser: http://localhost:8000
# Done! 🎉
```

## Choose Your Adventure

### I want to build an MCP agent

```bash
pmat scaffold agent --name my-agent --template basic
```

→ [Full Agent Guide](./agent-scaffolding.md)

### I want to build a WASM app

```bash
pmat scaffold wasm --name my-wasm --framework wasm-labs
```

→ [Full WASM Guide](./wasm-scaffolding.md)

### I want to see what would be created

```bash
# Agent dry run
pmat scaffold agent --name test --template basic --dry-run

# WASM dry run
pmat scaffold wasm --name test --framework wasm-labs --dry-run
```

## Common Workflows

### Prototype Fast (Standard Quality)

```bash
pmat scaffold agent --name prototype --template basic --quality standard
cd prototype
cargo run
```

### Production Ready (Strict Quality)

```bash
pmat scaffold agent --name production --template stateful --quality strict
cd production
cargo build --release
pmat quality-gates
```

### Mission Critical (Extreme Quality)

```bash
pmat scaffold agent \
  --name critical \
  --template hybrid \
  --features logging,metrics,tracing \
  --quality extreme

cd critical
cargo test
pmat quality-gates
```

## Next Steps

1. **Scaffold your project** using commands above
2. **Read the generated README.md** in your project
3. **Start coding** in src/
4. **Run tests frequently** with `cargo test`
5. **Check quality gates** with `pmat quality-gates`
6. **Install hooks** with `pmat hooks install` for automatic checks

## Tips

- Use `--dry-run` to preview before creating
- Use `--force` to overwrite existing projects
- Start with `basic` template, upgrade later
- Run `pmat quality-gates` often
- Install hooks early: `pmat hooks install`
- Follow generated README.md for specifics

## Help

```bash
# Agent help
pmat scaffold agent --help

# WASM help
pmat scaffold wasm --help

# List templates
pmat scaffold list-templates
```

## Examples Directory

See full examples:
- [Agent Scaffolding Guide](./agent-scaffolding.md)
- [WASM Scaffolding Guide](./wasm-scaffolding.md)
- [Main Examples README](./README.md)
```

## Files to Create

### New Files
- `examples/README.md` - Main examples directory
- `examples/agent-scaffolding.md` - Agent guide
- `examples/wasm-scaffolding.md` - WASM guide
- `examples/scaffolding-quickstart.md` - Quick start

## Verification

- [ ] All files created
- [ ] Markdown properly formatted
- [ ] Code examples correct
- [ ] Links work
- [ ] Consistent terminology
- [ ] Clear instructions

## Risk Assessment

**Very Low Risk:**
- Documentation only
- No code changes
- No dependencies

## Notes

Since we can't actually run the scaffolding commands without a built binary, these examples show:
1. What commands to run
2. What gets generated
3. Expected project structure
4. Next steps workflow

Developers can follow these guides once the binary is available.

**TDD Cycle Duration**: Estimated 30 minutes for documentation creation
