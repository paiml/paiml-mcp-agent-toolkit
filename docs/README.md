# PMAT Documentation

Welcome to the PMAT (Pragmatic AI MCP Agent Toolkit) documentation.

## 📚 Documentation Structure

### Core Documentation
- **[SPECIFICATION.md](./SPECIFICATION.md)** - Complete system specification (source of truth)
- **[CLAUDE_CODE_AGENT.md](./CLAUDE_CODE_AGENT.md)** - Claude Code Agent Mode user guide (v2.12.0)
- **[DISTRIBUTION_STATUS.md](./DISTRIBUTION_STATUS.md)** - Multi-ecosystem distribution status and automation
- **[DOCUMENTATION_STRUCTURE.md](./DOCUMENTATION_STRUCTURE.md)** - Documentation organization guide

### Active Documentation

#### Architecture & Design
- **[architecture/](./architecture/)** - System architecture and design decisions
  - [ARCHITECTURE.md](./architecture/ARCHITECTURE.md) - High-level architecture overview
  - [decisions/](./architecture/decisions/) - Architecture Decision Records (ADRs)

#### Development
- **[execution/](./execution/)** - Sprint planning and execution
  - [roadmap.md](./execution/roadmap.md) - Development roadmap with task tracking
  - [quality-gates.md](./execution/quality-gates.md) - Quality enforcement standards
  - [velocity.json](./execution/velocity.json) - Sprint velocity metrics

#### Features
- **[features/](./features/)** - Feature documentation
  - [README.md](./features/README.md) - Feature overview
  - **[SCAFFOLDING-AND-MAINTENANCE.md](./features/SCAFFOLDING-AND-MAINTENANCE.md)** - **🆕 v2.139.0** Project Scaffolding & Maintenance System (Complete Guide)
  - [claude-agent-sdk-guide.md](./claude-agent-sdk-guide.md) - Claude Agent SDK Integration Guide
  - [deep-wasm-usage.md](./deep-wasm-usage.md) - Deep WASM Pipeline Inspection (Phases 1-2.7 Complete)
  - [mutation-testing.md](./mutation-testing.md) - Mutation Testing with ML Prediction
  - Individual feature guides for each major capability

#### User Guides
- **[guides/](./guides/)** - User and integration guides
  - [interfaces-overview.md](./guides/interfaces-overview.md) - CLI, MCP, HTTP interfaces
  - [refactor-auto-guide.md](./guides/refactor-auto-guide.md) - Automated refactoring guide
  - [github-actions-quality-gate.md](./guides/github-actions-quality-gate.md) - CI/CD integration

#### Dogfooding
- **[dogfooding/](./dogfooding/)** - Real-world validation of PMAT features
  - [README.md](./dogfooding/README.md) - Dogfooding overview and command reference
  - [v2.139.0-INTEGRATION-SHOWCASE.md](./dogfooding/v2.139.0-INTEGRATION-SHOWCASE.md) - **🆕** Complete v2.139.0 integration showcase
  - [SPRINT-19-DOGFOODING-RESULTS.md](./dogfooding/SPRINT-19-DOGFOODING-RESULTS.md) - Sprint 19 findings

#### Operations
- **[operations/](./operations/)** - Operational documentation
  - [configuration.md](./operations/configuration.md) - Configuration guide
  - [error-handling.md](./operations/error-handling.md) - Error handling patterns
  - [telemetry.md](./operations/telemetry.md) - Monitoring and telemetry

#### Quality & Testing
- **[quality/](./quality/)** - Quality standards and metrics
  - [standards.md](./quality/standards.md) - Code quality standards
- **[testing/](./testing/)** - Testing documentation
  - [property-based.md](./testing/property-based.md) - Property-based testing guide
  - [integration.md](./testing/integration.md) - Integration testing
  - [performance.md](./testing/performance.md) - Performance testing

#### Specifications
- **[specifications/](./specifications/)** - Feature specifications
  - [roadmap-todo-quality-gate-spec.md](./specifications/roadmap-todo-quality-gate-spec.md) - Roadmap management spec
  - [publish-mcp-registry.md](./specifications/publish-mcp-registry.md) - MCP Registry publishing specification

### Release Information
- **[release-process.md](./release-process.md)** - Release workflow and procedures
- **[release_notes/](./release_notes/)** - Recent release notes (v2.x+)
- **[/CHANGELOG.md](../CHANGELOG.md)** - Complete version history

### Development Planning
- **[todo/](./todo/)** - Future development specifications
  - Active specifications for upcoming features
  - [archive/](./todo/archive/) - Completed or deprecated specs

### Reference
- **[cli-reference.md](./cli-reference.md)** - CLI command reference
- **[bugs/](./bugs/)** - Known issues and bug reports
  - [archived/](./bugs/archived/) - Resolved issues

## 🗄️ Archived Documentation

Historical and deprecated documentation has been moved to the archive:
- **[archive/](./archive/)** - Archived documentation
  - [ARCHIVE_INDEX.md](./archive/ARCHIVE_INDEX.md) - Archive navigation guide
  - [pre-v2.0/](./archive/pre-v2.0/) - Pre-2.0 version documentation
  - Historical release notes, implementation docs, and deprecated features

## 🚀 Quick Start

1. **New Users**: Start with [SPECIFICATION.md](./SPECIFICATION.md) for system overview
2. **Developers**: Check [execution/roadmap.md](./execution/roadmap.md) for current tasks
3. **Contributors**: Review [quality/standards.md](./quality/standards.md) for quality requirements
4. **Integrators**: See [guides/interfaces-overview.md](./guides/interfaces-overview.md) for API details

## 📖 Documentation Standards

All documentation follows these principles:
- **Single Source of Truth**: SPECIFICATION.md is the authoritative reference
- **Version Synchronized**: Documentation updates required with code changes
- **Quality Enforced**: Pre-commit hooks ensure documentation quality
- **Toyota Way Aligned**: Continuous improvement (Kaizen) approach

## 🔗 External Resources

- **Repository**: [github.com/paiml/paiml-mcp-agent-toolkit](https://github.com/paiml/paiml-mcp-agent-toolkit)
- **Crates.io**: [crates.io/crates/pmat](https://crates.io/crates/pmat)
- **MCP Registry**: [registry.modelcontextprotocol.io (io.github.paiml/pmat-agent)](https://registry.modelcontextprotocol.io/v0/servers?search=pmat)
- **Homepage**: [paiml.com](https://paiml.com)

## 🎯 Featured Capabilities

### Deep WASM Pipeline Inspection
Complete Rust/Ruchy → WebAssembly analysis pipeline with bidirectional tracing. **[Full Guide →](./deep-wasm-usage.md)**

- ✅ **Phase 1-2.7 Complete**: WASM binary parsing, DWARF correlation, mutation testing, unified parser
- 🔬 WASM binary parser with zero-copy analysis (wasmparser)
- 🗺️ DWARF v5 bidirectional source mapping (gimli)
- 🧬 WASM mutation testing with 3 operators (180 tests passing)
- ⚡ Unified WASM parser with 40-50% performance improvement
- 🎭 Ruchy language support for actor systems
- 📋 Phase 3 scoped: Runtime analysis, profiling, security scanning

### Mutation Testing Engine
Empirical mutation testing with actual test execution and optional ML prediction. **[Full Guide →](./mutation-testing.md)**

- ⚡ **20× faster than cargo-mutants** with smart test filtering (v2.135.0)
- ✨ **Properly formatted output** using prettyplease (v2.136.0)
- 🔧 **CRITICAL BUG FIXED**: File corruption issue resolved (Issue #64, v2.136.0)
- 🧪 Real test execution with `cargo test --lib` on each mutant
- 🔀 6 mutation operators (AOR, ROR, COR, UOR, CRR, SDL)
- 🌐 Multi-language support (Rust, WASM/WAT)
- 🧠 Optional ML prediction with decision tree classifier (75-95% accuracy)
- 🚀 Distributed execution with work-stealing queue
- 📈 CI/CD learning and auto-training (50 sample threshold)

### TypeScript/JavaScript Mutation Testing ✨ NEW (v2.144.0)
**Production-ready AST-based mutation testing for TypeScript and JavaScript.** **[Full Guide →](./features/TYPESCRIPT-MUTATION-TESTING.md)**

- 🎯 **80%+ mutation scores achievable** - Quantify test suite quality
- ⚡ **Fast generation** - 67 mutants in 14ms using tree-sitter AST
- 🔍 **Real test execution** - Works with jest, vitest, mocha
- 🧬 **5 mutation operators** - Arithmetic, equality, optional chaining, nullish coalescing, async/await
- 📊 **Identifies test gaps** - Surviving mutants show actual weaknesses
- 🔄 **Full automation** - Source → mutants → tests → score
- 🏗️ **Language-agnostic architecture** - Reusable for Python, Go, C++

### Python Mutation Testing ✨ NEW (v2.152.0)
**Production-ready AST-based mutation testing for Python 3.6+.** **[Full Guide →](./features/PYTHON-MUTATION-TESTING.md)**

- 🎯 **80%+ mutation scores achievable** - Validate test suite quality
- ⚡ **Ultra-fast generation** - 56 mutants in 5ms using tree-sitter AST
- 🔍 **Real test execution** - Works with pytest and unittest
- 🧬 **5 mutation operators** - Binary (AOR), relational (ROR), logical (LOR), identity (is/is not), membership (in/not in)
- 📊 **Identifies test gaps** - Surviving mutants reveal actual weaknesses
- 🔄 **Full automation** - Source → mutants → tests → score
- 🏗️ **Language-agnostic architecture** - Shared with TypeScript implementation

---

*Last Updated: 2025-10-08 | Version: 2.152.0*