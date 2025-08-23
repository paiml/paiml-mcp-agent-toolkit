# PMAT Documentation

Welcome to the PMAT (Pragmatic AI MCP Agent Toolkit) documentation.

## 📚 Documentation Structure

### Core Documentation
- **[SPECIFICATION.md](./SPECIFICATION.md)** - Complete system specification (source of truth)
- **[CLAUDE_CODE_AGENT.md](./CLAUDE_CODE_AGENT.md)** - Claude Code Agent Mode user guide (v2.10.0)
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
  - Individual feature guides for each major capability

#### User Guides
- **[guides/](./guides/)** - User and integration guides
  - [interfaces-overview.md](./guides/interfaces-overview.md) - CLI, MCP, HTTP interfaces
  - [refactor-auto-guide.md](./guides/refactor-auto-guide.md) - Automated refactoring guide
  - [github-actions-quality-gate.md](./guides/github-actions-quality-gate.md) - CI/CD integration

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
- **Homepage**: [paiml.com](https://paiml.com)

---

*Last Updated: 2025-08-21 | Version: 2.6.1*