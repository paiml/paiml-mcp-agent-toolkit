# PAIML MCP Agent Toolkit (pmat)

[![Crates.io](https://img.shields.io/crates/v/pmat.svg)](https://crates.io/crates/pmat)
[![npm](https://img.shields.io/npm/v/pmat-agent.svg)](https://www.npmjs.com/package/pmat-agent)
[![Docker](https://img.shields.io/docker/v/paiml/pmat?label=docker)](https://hub.docker.com/r/paiml/pmat)
[![Homebrew](https://img.shields.io/badge/homebrew-ready-blue)](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/homebrew)
[![AUR](https://img.shields.io/badge/AUR-ready-orange)](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/aur)
[![Chocolatey](https://img.shields.io/badge/chocolatey-ready-blue)](https://github.com/paiml/paiml-mcp-agent-toolkit/tree/master/chocolatey)

[![Documentation](https://docs.rs/pmat/badge.svg)](https://docs.rs/pmat)
[![CI/CD](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/main.yml)
[![Quality Gate](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/quality-gate-test.yml/badge.svg?branch=master)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/quality-gate-test.yml)
[![Multi-Ecosystem Release](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/multi-ecosystem-release.yml/badge.svg)](https://github.com/paiml/paiml-mcp-agent-toolkit/actions/workflows/multi-ecosystem-release.yml)

[![MCP Compatible](https://img.shields.io/badge/MCP-Compatible-green)](https://modelcontextprotocol.io)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Downloads](https://img.shields.io/crates/d/pmat)](https://crates.io/crates/pmat)
[![Rust 1.80+](https://img.shields.io/badge/rust-1.80+-orange.svg)](https://www.rust-lang.org)

**Zero-configuration AI context generation system** with extreme quality enforcement and Toyota Way standards. Analyze any codebase instantly through CLI, MCP, or HTTP interfaces. Built by [Pragmatic AI Labs](https://paiml.com).

> **🔧 v2.14.0 Release**: **Technical Debt Elimination via TDD!** Major fixes using Test-Driven Development:
> - **✅ Language Detection Fixed**: Functions now properly detected (was 0, now detects all)
> - **🚫 Zero Stub Implementations**: All stub code eliminated with real implementations
> - **📉 Complexity Reduced**: Ruchy parser from 89 to ≤4 cyclomatic complexity (95% reduction)
> - **🧪 TDD Coverage**: 80%+ test coverage on critical language detection paths
> - **🏭 Toyota Way Applied**: ONE implementation principle, zero defect tolerance

> **🎯 v2.13.0**: **Technical Debt Grading (TDG) System!** Complete code quality scoring with 6 orthogonal metrics:
> - **📊 Comprehensive Scoring**: Structural complexity, semantic complexity, code duplication, coupling analysis
> - **📚 Documentation Coverage**: Language-specific documentation pattern detection and scoring
> - **🎨 Consistency Analysis**: Naming conventions, indentation patterns, and code style consistency
> - **🏆 Grade Classification**: A+ through F grading system with detailed component breakdowns
> - **🌍 Multi-Language Support**: 10+ languages including Rust, Python, JavaScript, TypeScript, Go, Java, C/C++
> - **🛠️ CLI & MCP Integration**: `pmat analyze tdg` command and MCP tools for programmatic access
> - **📈 Project Analysis**: Directory-level analysis with language distribution and aggregated scoring

> **🚀 v2.10.0**: **Claude Code Agent Mode - "Always Working" Achievement!** Transform PMAT into a persistent background quality agent:
> - **🤖 Claude Code Integration**: Native MCP server for seamless Claude Code integration
> - **💾 Persistent State**: Monitoring state maintained across restarts with auto-save
> - **⚙️ Production Ready**: Environment-specific configs for dev, prod, and CI/CD
> - **📊 Real-time Monitoring**: Continuous quality tracking with file system watching
> - **🏗️ Service Architecture**: Systemd deployment with health checks and auto-restart

> **🎯 v2.9.0**: **Universal Demo "Just Works" Achievement!** Complete AI-powered repository intelligence with multi-language analysis:
> - **🤖 AI-Powered Recommendations**: Framework-aware repository recommendations with complexity-based learning tiers
> - **🌍 Multi-Language Intelligence**: Advanced polyglot analysis with cross-language dependency detection
> - **🏛️ Architecture Pattern Recognition**: Microservices, Layered, Event-driven pattern detection with confidence scoring
> - **📚 Repository Showcase Gallery**: Curated collection of 8+ repositories across languages and complexity levels
> - **⚡ Universal Demo**: Any GitHub repository URL → Complete analysis with AI recommendations
> - **🌐 Enhanced Web Demo**: Interactive visualizations with 3 new API endpoints (/api/recommendations, /api/polyglot, /api/showcase)
> - **Toyota Way Excellence**: Zero compilation defects maintained throughout development

## 🚀 Quick Start

### Installation

Choose your preferred installation method - PMAT is available across all major package ecosystems:

#### 🦀 Rust (Recommended)
```bash
cargo install pmat
```

#### 📦 Package Managers
```bash
# macOS/Linux - Homebrew
brew install pmat

# Windows - Chocolatey  
choco install pmat

# Ubuntu/Debian - APT
sudo apt install pmat                    # (via PPA - coming soon)

# Arch Linux - AUR
yay -S pmat

# Node.js - npm (global)
npm install -g pmat-agent
```

#### 🐳 Docker
```bash
# Latest version
docker run --rm -v $(pwd):/workspace paiml/pmat:latest pmat --version

# Interactive analysis
docker run --rm -v $(pwd):/workspace -w /workspace paiml/pmat:latest pmat context
```

#### 🔧 From Source
```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit
make build
```

#### 📥 Direct Download
```bash
# Linux/macOS Quick Install
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh

# Windows PowerShell
# Download from: https://github.com/paiml/paiml-mcp-agent-toolkit/releases
```

### Basic Usage
```bash
# Analyze current directory
pmat context

# Technical Debt Grading (TDG) - NEW!
pmat analyze tdg --path . --include-components

# Get complexity metrics
pmat analyze complexity --top-files 10

# Find technical debt
pmat analyze satd

# Analysis with timeout control - NEW! 🔧
pmat analyze complexity --timeout 30    # 30-second timeout
pmat analyze dead-code --timeout 60     # 60-second timeout  
pmat analyze satd --timeout 45          # 45-second timeout

# Run quality gates
pmat quality-gate --strict

# Start MCP server
pmat mcp
```

### Universal Demo - "Just Works" Analysis
```bash
# Analyze any GitHub repository with AI recommendations
cargo run --example analyze_github_repo -- --url https://github.com/rust-lang/rust-clippy

# Compare multiple repositories across languages
cargo run --example compare_repos

# Run quality gates on GitHub repositories
cargo run --example quality_gate_github -- https://github.com/owner/repo

# Start interactive web demo
pmat demo --serve
# Then visit http://localhost:8080 for:
# • AI-powered repository recommendations
# • Multi-language project intelligence
# • Repository showcase gallery
# • Interactive analysis visualizations
```

### Toyota Way Development (NEW)
```bash
# Setup quality enforcement (one-time)
make setup-quality

# Start development with quality checks
make dev

# Create quality-enforced commit
make commit

# Verify sprint quality
make sprint-close
```

## 🎯 Core Capabilities

### Analysis Engine
- **Technical Debt Grading (TDG)**: 6-metric orthogonal code quality scoring with A+ through F grading
- **Complexity Analysis**: McCabe cyclomatic & cognitive complexity with AST precision
- **Dead Code Detection**: Graph-based reachability analysis across 30+ languages
- **SATD Detection**: Self-admitted technical debt with severity classification
- **Documentation Coverage**: Language-specific pattern detection with scoring algorithms
- **Consistency Analysis**: Naming conventions and code style consistency measurement
- **Deep Context Generation**: Multi-dimensional analysis optimized for AI agents

### 🤖 AI-Powered Intelligence (NEW)
- **Smart Recommendations**: Framework-aware repository suggestions with complexity matching
- **Polyglot Analysis**: Cross-language dependency detection and architecture pattern recognition
- **Repository Showcase**: Curated gallery with learning pathways from beginner to expert
- **Integration Points**: Risk assessment of multi-language project coupling with mitigation strategies

### Quality Systems
- **Quality Gates**: Zero-tolerance enforcement (complexity ≤20, SATD=0, coverage >80%)
- **Quality Proxy**: AI code interceptor with 7-stage validation pipeline
- **PDMT Integration**: Deterministic todo generation with embedded quality requirements
- **Refactoring Engine**: State machine-based code transformation with ACID snapshots

### Integration Protocols
- **MCP Protocol**: 18 tools via unified pmcp SDK 1.2.0 server (includes TDG analysis tools)
- **HTTP API**: RESTful with Server-Sent Events streaming
- **CLI Interface**: 47 commands with POSIX-compliant exit semantics

## 📖 Documentation

### Core Documentation
- **[Complete Specification](docs/SPECIFICATION.md)** - Unified source of truth (36 sections)
- **[TDG Guide](docs/TDG_GUIDE.md)** - **NEW!** Technical Debt Grading system documentation
- **[API Reference](docs/api-guide.md)** - Service APIs and integration patterns
- **[CLI Reference](docs/cli-reference.md)** - Complete command documentation

### Quality & Development
- **[Toyota Way Guide](docs/execution/quality-gates.md)** - Development workflow and standards
- **[Sprint Management](docs/execution/roadmap.md)** - Task tracking and execution DAG
- **[Quality Gates](docs/quality-gates-proxy-detailed.md)** - Enforcement mechanisms

### Integration Guides
- **[MCP Integration](docs/mcp-methods.md)** - Model Context Protocol setup
- **[PDMT Guide](docs/pdmt-integration-guide.md)** - Deterministic todo generation
- **[CI/CD Integration](docs/integrations/ci-cd-integration.md)** - Pipeline integration

## 🏗️ Architecture

PMAT implements Toyota Production System principles through rigorous static analysis:

- **Kaizen (改善)**: Iterative file-by-file improvement with measurable ΔQ metrics
- **Genchi Genbutsu (現地現物)**: Direct AST traversal, no heuristics
- **Jidoka (自働化)**: Automated quality gates with fail-fast semantics
- **Zero SATD Policy**: Compile-time enforcement of zero technical debt

### Service Architecture
```rust
// Unified service layer with dependency injection
pub trait Service: Send + Sync {
    type Input: Serialize + DeserializeOwned;
    type Output: Serialize + DeserializeOwned;
    
    async fn process(&self, input: Self::Input) -> Result<Self::Output, Self::Error>;
}

// All protocols use unified request/response
#[derive(Serialize, Deserialize)]
pub struct UnifiedRequest {
    pub operation: Operation,
    pub params: Value,
    pub context: RequestContext,
}
```

### Performance Characteristics
- **Startup**: 4ms hot, 127ms cold (mmap'd grammar cache)
- **Analysis**: 487K LOC/s single-thread, 3.9M LOC/s multi-core
- **Memory**: 47MB base + 312KB per KLOC
- **SIMD**: 43% vectorized paths, 2.7x AVX2 speedup

## 🛠️ Development

### Requirements
- Rust 1.80.0+ 
- Git (for repository analysis)

### Build from Source
```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit

# Setup Toyota Way quality enforcement
make setup-quality

# Build and test
make build
make validate

# Run examples
make examples
```

### Library Usage
```toml
[dependencies]
pmat = "2.35.0"
```

```rust
use pmat::services::code_analysis::CodeAnalysisService;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let service = CodeAnalysisService::new();
    
    // Generate AI-optimized context
    let context = service.generate_context(".", None).await?;
    
    // Analyze complexity with Toyota Way standards
    let complexity = service.analyze_complexity(".", Some(10)).await?;
    
    Ok(())
}
```

## 🔍 Language Support

- **Rust**: Full cargo integration with syn AST
- **TypeScript/JavaScript**: SWC-based parsing
- **Python**: RustPython AST analysis  
- **C/C++**: Tree-sitter with goto tracking
- **Ruchy**: v1.5.0 support with advanced analysis
  - Full AST parsing with 35+ token types
  - Halstead metrics (volume, difficulty, effort, time, bugs)
  - Dead code detection (unused functions/variables)  
  - Type inference for literals and binary operations
  - Actor message flow analysis with deadlock detection
  - Enhanced pattern matching complexity scoring
  - Import/export dependency tracking
- **Kotlin**: Tree-sitter based analysis
- **30+ Languages**: Via tree-sitter grammar support

## 🤖 MCP Integration

PMAT provides 18 MCP tools via unified pmcp SDK server:

```bash
# Start MCP server (auto-detects transport)
pmat mcp

# Test with Claude Code
cargo run --example mcp_server_pmcp
cargo run --example test_pmcp_server
```

### Available Tools
- `analyze_tdg` - **NEW!** Technical Debt Grading with 6-metric scoring
- `analyze_tdg_compare` - **NEW!** Compare TDG scores between files/projects
- `analyze_complexity` - Complexity metrics
- `analyze_satd` - Technical debt detection  
- `analyze_dead_code` - Unused code analysis
- `quality_gate` - Comprehensive quality validation
- `refactor_start` - Begin refactoring workflow
- `pdmt_deterministic_todos` - Generate quality todos
- `github_create_issue` - Create GitHub issues
- **NEW**: AI recommendation tools for intelligent repository analysis
- And 9 more...

## 🤖 Claude Code Agent Mode (NEW v2.10.0)

Transform PMAT into a persistent background quality agent that continuously monitors your codebase:

### Quick Start with Claude Code

```bash
# Start agent as MCP server for Claude Code
pmat agent mcp-server

# Configure in Claude Code settings.json:
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "env": {}
    }
  }
}
```

### Background Daemon Mode

```bash
# Start monitoring a project
pmat agent start --project-path /path/to/project

# Check monitoring status
pmat agent status

# Stop monitoring
pmat agent stop
```

### Key Features
- **Real-time Monitoring**: File system watching with instant quality feedback
- **Persistent State**: Maintains metrics across restarts with auto-save
- **Toyota Way Compliance**: Enforces ≤20 complexity with zero SATD tolerance
- **Analysis Timeouts**: Configurable timeouts prevent infinite hangs (NEW! 🔧)
- **Production Ready**: Systemd service with health checks and auto-restart
- **MCP Native**: Seamless Claude Code integration via stdio transport

### Available Agent Tools
- `start_quality_monitoring` - Begin monitoring a project
- `stop_quality_monitoring` - Stop monitoring
- `get_quality_status` - Current quality metrics
- `run_quality_gates` - Execute quality checks
- `analyze_complexity` - Complexity analysis
- `health_check` - Agent health status

See [Claude Code Agent Guide](docs/CLAUDE_CODE_AGENT.md) for detailed setup and deployment instructions.

### 🌐 Web Demo API Endpoints (NEW)
```bash
# AI-powered repository recommendations
GET /api/recommendations

# Multi-language project intelligence
GET /api/polyglot

# Repository showcase gallery
GET /api/showcase

# Core analysis APIs
GET /api/summary
GET /api/metrics
GET /api/hotspots
GET /api/dag
```

## 📊 Quality Standards

PMAT enforces extreme quality standards:

- **Complexity**: ≤20 cyclomatic, ≤15 cognitive
- **Technical Debt**: 0 SATD comments allowed
- **Test Coverage**: >80% with property-based testing
- **Code Quality**: 0 lint warnings, 0 dead code
- **Documentation**: Synchronized with every commit

### Quality Gates
```bash
# Run comprehensive quality analysis
pmat quality-gate --strict

# CI/CD integration
pmat analyze complexity --fail-on-violation
pmat analyze satd --fail-on-violation
pmat quality-gate --strict --fail-on-violation
```

## 🚀 Contributing

PMAT follows Toyota Way development principles:

1. **Setup quality enforcement**: `make setup-quality`
2. **Start development**: `make dev` 
3. **Make changes** with documentation updates
4. **Quality-enforced commit**: `make commit`
5. **Sprint verification**: `make sprint-close`

All contributions must meet:
- Zero SATD comments
- Complexity ≤20 per function
- Full test coverage
- Documentation updates

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

## 📋 License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

Built with ❤️ by [Pragmatic AI Labs](https://paiml.com)