# Release Notes for v0.15.0

## 🎯 Major Feature Release: Dead Code Analysis with Cross-Reference Tracking

This release introduces comprehensive dead code detection with advanced reachability analysis, completing the implementation of unfinished TODO items and bringing the dead code analyzer to full production readiness.

## ✨ New Features

### Dead Code Analysis (`analyze dead-code`)
- **NEW**: Cross-reference tracking with multi-level reachability analysis
- **NEW**: Entry point detection for main functions, public APIs, and exported items
- **NEW**: Dynamic dispatch resolution for virtual method calls and trait implementations
- **NEW**: Hierarchical bitset with SIMD-optimized reachability tracking using RoaringBitmap
- **NEW**: Confidence scoring (High/Medium/Low) for detected dead code accuracy
- **NEW**: File ranking system with composite scoring and `--top-files` flag support
- **NEW**: Support for functions, classes, variables, and unreachable code blocks
- **NEW**: Multiple output formats: JSON, SARIF, Markdown, and summary table

### MCP Integration
- **NEW**: `analyze_dead_code` MCP tool with full parameter support
- **NEW**: Consistent interface across CLI, MCP, and HTTP endpoints
- **NEW**: Comprehensive test coverage with 18 dead code analyzer tests
- **UPDATED**: MCP tools list now includes dead code analysis (11 total tools)

### Core Implementation Completed
- **FIXED**: Implemented missing SIMD slice access method that was returning `unimplemented!()`
- **FIXED**: Built complete reference graph generation from AST and dependency graphs
- **FIXED**: Added proper entry point detection with intelligent heuristics
- **FIXED**: Implemented dynamic dispatch resolution for trait/interface calls
- **FIXED**: Added comprehensive error handling with saturating arithmetic
- **FIXED**: Complete test coverage for all dead code models and algorithms

## 🔧 Technical Implementation

### Dead Code Analyzer (`server/src/services/dead_code_analyzer.rs`)
- Implemented `HierarchicalBitSet` with RoaringBitmap backend for efficient reachability tracking
- Added `CrossLangReferenceGraph` for cross-language dependency analysis
- Created `VTableResolver` for dynamic dispatch resolution
- Built comprehensive `DeadCodeAnalyzer` with four-phase analysis pipeline:
  1. Reference graph building from AST/dependency data
  2. Dynamic dispatch resolution for virtual calls
  3. Vectorized reachability marking with SIMD optimization
  4. Dead code classification by type (functions, classes, variables, unreachable blocks)

### Dead Code Models (`server/src/models/dead_code.rs`)
- Implemented `FileDeadCodeMetrics` with weighted scoring algorithm
- Added `ConfidenceLevel` enum with Copy, PartialEq, Eq traits
- Created `DeadCodeRankingResult` for comprehensive analysis results
- Built `DeadCodeSummary` with aggregated statistics across files
- Added `DeadCodeAnalysisConfig` for customizable analysis behavior

### Cross-Interface Support
- **CLI**: Full `analyze dead-code` command with format options and file ranking
- **MCP**: JSON-RPC compatible `analyze_dead_code` tool with all parameters
- **HTTP**: RESTful API endpoints supporting GET and POST methods

## 📊 Usage Examples

```bash
# CLI Usage
paiml-mcp-agent-toolkit analyze dead-code --top-files 10 --format json
paiml-mcp-agent-toolkit analyze dead-code --include-tests --format sarif
paiml-mcp-agent-toolkit analyze dead-code --min-dead-lines 5 --format markdown

# MCP Tool Call
{"method": "analyze_dead_code", "params": {"project_path": "./", "top_files": 10, "format": "json"}}

# HTTP API
GET /api/v1/analyze/dead-code?top_files=10&format=json
POST /api/v1/analyze/dead-code {"top_files": 10, "include_tests": false, "min_dead_lines": 10}
```

## 🧪 Test Coverage Improvements

### Dead Code Analysis Tests (18 new tests)
- **NEW**: `HierarchicalBitSet` functionality tests
- **NEW**: `DeadCodeAnalyzer` workflow tests with entry points and references
- **NEW**: `VTableResolver` dynamic dispatch tests
- **NEW**: `CrossLangReferenceGraph` edge creation and lookup tests
- **NEW**: `CoverageData` integration tests
- **NEW**: Complete model tests for all dead code data structures
- **NEW**: Async ranking analysis tests with real project paths
- **FIXED**: All tests passing with comprehensive edge case coverage

### E2E Test Updates
- **UPDATED**: MCP protocol test to expect 11 tools (was 10)
- **VERIFIED**: All interface consistency checks passing
- **MAINTAINED**: Zero test failures across all suites

## 🎯 Quality Improvements

### Code Quality
- **FIXED**: Integer overflow issues using `saturating_sub()` for safe arithmetic
- **FIXED**: Missing derive traits (Copy, PartialEq, Eq) on core enums
- **FIXED**: Unused variable warnings with proper underscore prefixing
- **FIXED**: Compilation errors related to field access and method resolution
- **ACHIEVED**: Zero lint warnings and 100% successful compilation

### Documentation Updates
- **UPDATED**: README.md with comprehensive dead code analysis section
- **ADDED**: MCP tools table entry for `analyze_dead_code`
- **ENHANCED**: CLI usage examples with dead code analysis commands
- **IMPROVED**: Feature descriptions with technical implementation details

## 🚀 Performance Characteristics

- **Startup**: <10ms for dead code analysis initialization
- **Analysis**: SIMD-optimized reachability tracking with RoaringBitmap
- **Memory**: Efficient hierarchical bitset representation
- **Scaling**: Vectorized algorithms for large codebases (>1000 files)
- **Caching**: Persistent analysis results with intelligent cache invalidation

## 📈 Looking Forward

The dead code analyzer now provides production-ready static analysis capabilities with:
- Multi-language support (Rust, TypeScript, Python)
- Cross-reference accuracy through reachability analysis
- Confidence scoring to reduce false positives
- Integration with existing complexity and churn analysis tools
- Full triple-interface support (CLI, MCP, HTTP)

---

# Release Notes for v0.12.2

## 🎯 Feature Release: Advanced File Ranking System

This release introduces a comprehensive file ranking system with `--top-files` parameter, providing intelligent complexity analysis across all interfaces.

## ✨ New Features

### File Ranking System
- **NEW**: `--top-files` parameter for complexity analysis across CLI, MCP, and HTTP interfaces
- **NEW**: Composite complexity scoring algorithm (40% cyclomatic, 40% cognitive, 20% function count)
- **NEW**: Intelligent file ranking with parallel processing and caching using rayon
- **NEW**: Table and JSON output formats with detailed complexity metrics
- **NEW**: Cross-interface consistency ensuring identical results across CLI, MCP, and HTTP

### Enhanced Analysis Capabilities
- **IMPROVED**: Complexity analysis now includes file ranking and prioritization
- **NEW**: `FileRanker` trait system for extensible ranking algorithms
- **NEW**: `RankingEngine` with performance optimization and result caching
- **NEW**: Actionable complexity insights showing top files needing attention

### Interface Improvements
- **CLI**: Added `--top-files` parameter to `analyze complexity` command
- **MCP**: Extended `analyze_complexity` tool with `top_files` parameter support
- **HTTP**: Added support for `top_files` in both POST (JSON body) and GET (query params) requests

## 🔧 Implementation Details

### Core Ranking System (`server/src/services/ranking.rs`)
- Implemented `FileRanker` trait for pluggable ranking algorithms
- Added `ComplexityRanker` with composite scoring methodology
- Created `RankingEngine` with parallel processing capabilities
- Integrated caching system for performance optimization

### Cross-Interface Support
- **CLI Interface**: Full integration with existing complexity analysis workflow
- **MCP Interface**: JSON-RPC compatible parameter handling and response formatting
- **HTTP Interface**: RESTful API support with query parameter and JSON body options

## 📊 Dogfooding Results

Using our own `--top-files` system on this codebase:
- **146 files analyzed** with 15,239 total functions
- **158 hours estimated technical debt** identified
- **Top 5 complexity hotspots** ranked and prioritized:
  1. `./server/src/services/context.rs` (Score: 30.9) - 32 max cyclomatic complexity
  2. `./server/tests/documentation_examples.rs` (Score: 25.3) - 23 max cyclomatic complexity
  3. `./server/src/services/mermaid_generator.rs` (Score: 24.6) - 25 max cyclomatic complexity
  4. `./server/src/cli/mod.rs` (Score: 24.1) - 24 max cyclomatic complexity
  5. `./server/src/services/embedded_templates.rs` (Score: 23.3) - 22 max cyclomatic complexity

## 🚀 Usage Examples

```bash
# CLI Usage
paiml-mcp-agent-toolkit analyze complexity --top-files 5 --format json

# MCP Tool Call
{"method": "analyze_complexity", "params": {"top_files": 5, "format": "json"}}

# HTTP API
GET /api/v1/analyze/complexity?top_files=5&format=json
POST /api/v1/analyze/complexity {"top_files": 5, "format": "json"}
```

## 📚 Documentation Updates
- Updated README.md with latest complexity metrics using our own ranking system
- Enhanced CLI examples to showcase `--top-files` functionality
- Updated rust-docs with current performance benchmarks
- Comprehensive enhancement documentation in `docs/enhancement-top-files-flag.md`

---

# Release Notes for v0.12.0

## 🎯 Major Release: Unified Protocol Architecture

This is a significant release introducing a unified protocol architecture that consolidates CLI, HTTP, and MCP interfaces into a single, cohesive system.

## 🏗️ Architecture Improvements

### Unified Protocol System
- **NEW**: Unified protocol architecture supporting CLI, HTTP, and MCP interfaces
- **NEW**: Protocol adapters with type-safe request/response handling
- **NEW**: Centralized routing with Axum-based unified service
- **NEW**: Cross-protocol metrics collection and observability

### Enhanced MCP Integration
- **IMPROVED**: Full MCP 2.0 JSON-RPC compliance with 10+ tools
- **NEW**: Auto-detection of MCP mode via stdin/environment
- **NEW**: Comprehensive tool schema definitions
- **IMPROVED**: Error handling with proper JSON-RPC error codes

## ✨ New Features

### Analysis Capabilities
- **NEW**: `analyze_system_architecture` - High-level architectural analysis
- **NEW**: `analyze_defect_probability` - Predict defect-prone code areas
- **NEW**: Canonical query system for structured code analysis
- **IMPROVED**: Enhanced dependency graph generation with complexity metrics

### Web Interface
- **NEW**: Interactive demo server with real-time analysis
- **IMPROVED**: Web-based project visualization and metrics
- **NEW**: HTTP API endpoints for all analysis functions

## 🔧 Code Quality & Performance

### Zero Warnings Achieved
- **FIXED**: All 14 clippy lint warnings resolved
- **FIXED**: Test compilation errors and missing struct fields
- **FIXED**: Axum route syntax updated from `:param` to `{param}`
- **IMPROVED**: Clean compilation with zero warnings

### Performance Enhancements
- **IMPROVED**: Sub-10ms startup time maintained
- **IMPROVED**: Template generation <5ms
- **IMPROVED**: Enhanced caching system with persistent storage
- **METRICS**: 343 passing tests with 85%+ coverage

## 📚 Documentation & Developer Experience

### Comprehensive Documentation Updates
- **UPDATED**: README.md completely modernized for unified architecture
- **NEW**: Detailed MCP tools table with 10+ available tools
- **NEW**: Modern Mermaid architecture diagrams
- **UPDATED**: Performance metrics and test coverage information
- **IMPROVED**: CLI command examples and parameter documentation

### Project Organization
- **REORGANIZED**: Script directory with archived deprecated scripts
- **CLEANED**: Removed outdated coverage files and temporary artifacts
- **IMPROVED**: Clear project structure and component organization

## 🛠️ Technical Improvements

### Code Structure
- **NEW**: `src/unified_protocol/` module with adapters and service
- **NEW**: `src/services/canonical_query.rs` for structured analysis
- **NEW**: `src/services/defect_probability.rs` for quality prediction
- **IMPROVED**: Modular service architecture with dependency injection

### Testing & Quality
- **IMPROVED**: Comprehensive E2E test suite
- **FIXED**: Demo integration tests with proper struct initialization
- **ENHANCED**: Documentation synchronization tests
- **MAINTAINED**: Zero technical debt with comprehensive testing

## 📊 Metrics & Performance

- **Tests**: 343 passing tests (improved from 313)
- **Coverage**: 85%+ test coverage (improved from 81%)
- **Performance**: <10ms startup, <5ms template rendering
- **Architecture**: Zero external dependencies, single binary design
- **Quality**: Zero lint warnings, comprehensive error handling

## 🔄 Migration & Compatibility

### Backward Compatibility
- **MAINTAINED**: All existing CLI commands work unchanged
- **MAINTAINED**: All MCP tools retain same interface
- **MAINTAINED**: Template URIs and generation logic unchanged

### New Capabilities
- **AVAILABLE**: HTTP API endpoints for programmatic access
- **AVAILABLE**: Enhanced analysis tools for code quality
- **AVAILABLE**: Interactive web demo for showcasing capabilities

## 🚀 Getting Started

```bash
# Install latest version
curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh

# Quick CLI usage
paiml-mcp-agent-toolkit demo
paiml-mcp-agent-toolkit scaffold rust --templates makefile,readme,gitignore

# MCP integration with Claude Code
claude mcp add paiml-toolkit ~/.local/bin/paiml-mcp-agent-toolkit
```

## 📈 What's Next

- Enhanced dependency graph algorithms
- Additional analysis patterns and metrics
- Extended template library
- Performance optimizations for large codebases

---

*This release represents a major step forward in providing a unified, production-ready toolkit for AI-assisted development workflows.*

## Previous Releases

### v0.10.x Series
- **v0.10.1**: Release process improvements and bug fixes
- **v0.10.0**: Initial unified protocol groundwork and enhanced demo validation

### v0.9.x Series
- Enhanced demo functionality and validation
- Mermaid visualization improvements
- Bug fixes and performance optimizations

### v0.4.7 
- Added comprehensive MCP documentation synchronization tests
- Fixed Rust clippy warnings and improved code organization
- Documented all 10 MCP tools with examples
- Integrated documentation sync tests into build process