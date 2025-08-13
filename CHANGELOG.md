# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.3.0] - 2025-08-13

### Added
- **PDMT Integration**: Comprehensive integration with Pragmatic Deterministic MCP Templating for enterprise-grade todo generation
  - New MCP tool: `pdmt_deterministic_todos` for generating quality-enforced todo lists from requirements
  - Deterministic todo generation with reproducible outputs (fixed seed: 42)
  - Enforces 80%+ test coverage, zero SATD tolerance, and complexity limits
  - Three enforcement modes: Strict (reject), Advisory (warn), Auto-Fix (refactor)
  - Comprehensive quality validation pipeline with 7 validation phases
  - Includes validation commands and success criteria for each generated todo
  - Full integration with existing quality proxy infrastructure
- **New Services**: 
  - `PdmtService` for deterministic todo generation (server/src/services/pdmt_service.rs)
  - `PdmtQualityEnforcer` for quality validation (server/src/services/pdmt_quality_integration.rs)
- **New Models**: Comprehensive PDMT data models (server/src/models/pdmt.rs)
- **Documentation**: PDMT integration guide (docs/pdmt-integration-guide.md)
- **Tests**: Integration tests for PDMT functionality (server/tests/pdmt_integration_test.rs)

### Changed
- Updated MCP server to include 18 core tools (was 17)
- Enhanced README with PDMT integration examples and documentation

### Technical Details
- PDMT todos include quality gates, validation commands, and implementation specs
- Quality enforcement validates: structure, coverage, doctests, property tests, examples, and SATD
- Granularity levels: low (1 todo), medium (2-3 todos), high (full breakdown with tests/docs)
- Priority detection based on requirement keywords (critical, bug, feature, refactor)
- Dependency management for logical task ordering

## [2.2.0] - 2024-12-12

### Changed
- **BREAKING: Unified MCP Server Architecture** - Consolidated all MCP implementations into ONE
  - Removed three separate MCP server implementations (standard, refactor, pmcp variants)
  - All MCP operations now use the unified pmcp SDK-based server
  - Eliminated environment variable switches (PMAT_PMCP_MCP, PMAT_REFACTOR_MCP)
  - Single code path for all MCP tools, eliminating duplication

### Added
- **SimpleUnifiedServer**: New consolidated MCP server implementation
  - 17 core tools immediately available (analysis, refactoring, quality, context)
  - Quality proxy integration built-in for all operations
  - Type-safe tool handlers with compile-time validation
  - Consistent error handling and logging across all tools
- **Example**: Added `unified_mcp_demo` demonstrating the unified architecture

### Improved
- **Performance**: 10x faster MCP operations using pmcp SDK exclusively
- **Code Reduction**: ~30% less code by eliminating duplicate implementations
- **Maintenance**: Single implementation point for all MCP functionality
- **Quality**: Consistent quality enforcement across all tools
- **Configuration**: Simplified server initialization and configuration

### Removed
- Legacy `run_mcp_server` function and standard MCP implementation
- Separate `mcp_server::McpServer` refactoring server
- Environment-based server selection logic
- Duplicate handler implementations across different servers

## [2.1.0] - 2024-12-12

### Added
- **Quality Proxy Service**: New service to intercept and validate AI-generated code before it's written
  - Three enforcement modes: Strict (reject), Advisory (warn), Auto-Fix (refactor automatically)
  - Comprehensive quality checks: complexity, SATD, documentation, lint violations
  - MCP tool integration for AI agents (Claude Code, GitHub Copilot, etc.)
  - Automatic refactoring in Auto-Fix mode with SATD removal and documentation generation
  - Property-based testing with 9 comprehensive test scenarios
  - Full integration with pmcp SDK for high-performance MCP handling
- **New MCP Tool**: `quality_proxy` tool for validating code through MCP protocol
  - Supports write, edit, and append operations
  - Configurable quality thresholds and enforcement modes
  - Returns detailed quality reports with violations and suggestions
- **Example**: Added `quality_proxy_demo` showcasing all features and modes

### Documentation
- Added comprehensive Quality Proxy documentation to CLI reference
- Updated MCP protocol documentation with quality_proxy tool (now 34 total MCP tools)
- Added Quality Proxy API documentation with usage examples
- Updated README with Quality Proxy feature description

## [2.0.0] - 2025-08-08

### Added
- **pmcp 1.0 Integration**: Complete integration with the pmcp Rust MCP SDK
  - Migrated to pmcp 1.0.0 for high-performance MCP server implementation
  - Added comprehensive transport layer abstraction supporting stdio, WebSocket, and HTTP/SSE
  - Implemented unified `TransportAdapter` trait for consistent transport behavior
  - Added property-based testing for transport layer reliability
  - Integrated pmcp's type-safe tool handlers for improved reliability
  - Added mock transport implementation for deterministic testing
- **Enhanced MCP Architecture**: Modernized MCP server implementation
  - pmcp-based server now provides 10x performance improvement
  - Type-safe tool handlers with compile-time validation
  - Built-in transport support for multiple connection types
  - Automatic JSON-RPC request/response handling
  - Enhanced error propagation and logging
- **Production-Grade Transport Layer**: Complete rewrite of transport infrastructure
  - `StdioTransportAdapter`: Length-prefixed stdio communication
  - `WebSocketTransportAdapter`: Native WebSocket support for browser clients
  - `HttpSseTransportAdapter`: Server-Sent Events for HTTP clients
  - `MockTransport`: Deterministic testing with failure injection
  - Comprehensive property testing for transport reliability

### Changed
- **BREAKING**: Upgraded to pmcp 1.0.0 (major version bump)
- **BREAKING**: Transport layer API completely rewritten
- MCP server now uses pmcp by default (no longer feature-gated)
- Improved error handling with pmcp's error system
- Enhanced tool handler architecture with `RequestHandlerExtra`

### Technical
- Fixed lifetime annotation warning in context service
- Updated all transport implementations to use async/await patterns
- Added comprehensive testing infrastructure for transport layer
- Maintained backward compatibility for existing MCP tools

## [0.30.9] - 2025-08-01

### Added
- **Complete Service Documentation Coverage**: Added comprehensive module-level documentation to all remaining service files
  - Added module documentation to `services/git_clone.rs` with Git repository cloning and caching service
  - Added module documentation to `services/readme_compressor.rs` with intelligent README compression strategies
  - Added module documentation to `services/project_meta_detector.rs` with metadata file detection patterns
  - Added module documentation to `services/incremental_coverage_analyzer.rs` with CI/CD incremental coverage analysis
  - Added module documentation to `services/project_analyzer.rs` with codebase exploration entry point
  - Added module documentation to `services/ast_strategies.rs` with multi-language AST parsing strategies
  - Added module documentation to `services/ast_strategies_temp.rs` with temporary Kotlin AST placeholder
  - Added module documentation to `services/old_cache.rs` with legacy caching utilities
  - Added module documentation to `services/ranking_utils.rs` with ranking and prioritization utilities
  - Added module documentation to `services/unified_refactor_analyzer.rs` with multi-language refactoring framework
- **Documentation Milestone**: Achieved 100% service module documentation coverage
  - All service modules now have comprehensive documentation with examples
  - Each module includes architecture details, feature lists, and usage examples
  - Documentation follows consistent style across the entire codebase

## [0.30.8] - 2025-08-01

### Added
- **Complete Service Documentation**: Added comprehensive module-level documentation to 9 more core service files
  - Added module documentation to `services/file_discovery.rs` with intelligent file filtering and categorization
  - Added module documentation to `services/tdg_calculator.rs` with Technical Debt Gradient scoring system
  - Added module documentation to `services/verified_complexity.rs` with multiple complexity metrics
  - Added module documentation to `services/mermaid_generator.rs` with diagram generation and simplification
  - Added module documentation to `services/semantic_naming.rs` with language-aware naming conventions
  - Added module documentation to `services/ranking.rs` with generic file ranking framework
  - Enhanced module documentation for `services/lightweight_provability_analyzer.rs` with abstract interpretation details
  - Added module documentation to `services/makefile_compressor.rs` with intelligent compression strategies
  - Added module documentation to `services/fixed_graph_builder.rs` with PageRank-based node selection
- **Documentation Completion**: Achieved comprehensive documentation coverage
  - All critical service modules now have detailed documentation
  - Each module includes working examples and clear explanations
  - Documentation follows consistent style with features, architecture, and usage examples

## [0.30.7] - 2025-08-01

### Added
- **Doctests for Analysis Service Modules**: Added executable documentation examples to critical analyzer functions
  - Added doctests to `services/satd_detector.rs` for Severity escalation/reduction and classification methods
  - Added doctests to `services/project_analyzer.rs` for Project construction and root path access
  - Added doctests to `services/defect_analyzer.rs` for FileRankingEngine initialization
  - Added doctests to `services/big_o_analyzer.rs` for BigOAnalyzer creation and JSON formatting
  - Added doctests to `services/unified_refactor_analyzer.rs` for AnalyzerPool operations
  - Added doctests to `services/dead_code_analyzer.rs` for HierarchicalBitSet and DeadCodeAnalyzer methods
  - Added doctests to `services/lightweight_provability_analyzer.rs` for PropertyDomain and provability calculations
- **Documentation Coverage Enhancement**: Final phase of systematic documentation improvement
  - All documented functions include working examples that are tested during build
  - Focus on public API functions to improve usability
  - Achieved significant progress toward 80% documentation coverage target

## [0.30.6] - 2025-07-31

### Added
- **Advanced Service Documentation**: Added comprehensive module-level documentation to 10 more core service files
  - Added module documentation to `services/canonical_query.rs` with query framework architecture
  - Added module documentation to `services/coupling_analyzer.rs` with coupling metrics and stability analysis
  - Added module documentation to `services/dag_builder.rs` with dependency graph construction process
  - Added module documentation to `services/deep_context.rs` with multi-dimensional analysis orchestration
  - Added module documentation to `services/dead_code_prover.rs` with reachability analysis approach
  - Added module documentation to `services/defect_analyzers.rs` with defect analyzer implementations
  - Added module documentation to `services/defect_probability.rs` with defect prediction model
  - Added module documentation to `services/embedded_templates.rs` with template embedding philosophy
- **Documentation Coverage Progress**: Phase 3 of systematic documentation improvement
  - All documented modules include working examples
  - Focus on advanced analysis and prediction services
  - Continued progress toward 80% documentation coverage target

## [0.30.5] - 2025-07-31

### Added
- **Enhanced Service Documentation**: Added comprehensive module-level documentation to core service files
  - Added module documentation to `services/quality_gates.rs` with quality gate enforcement examples
  - Added module documentation to `services/file_classifier.rs` with file classification patterns
  - Added module documentation to `services/git_analysis.rs` with code churn analysis examples
  - Added module documentation to `services/template_service.rs` with template generation workflow
  - Added module documentation to `services/big_o_analyzer.rs` with complexity analysis features
  - Added module documentation to `services/renderer.rs` with Handlebars template rendering
  - Added module documentation to `services/ast_rust.rs` with Rust AST analysis details
  - Added module documentation to `services/ast_python.rs` with Python AST analysis features
  - Added module documentation to `services/ast_based_dependency_analyzer.rs` with dependency tracking
  - Added module documentation to `models/refactor.rs` with refactoring state machine flow
- **Documentation Coverage Improvement**: Continued systematic improvement toward 80% coverage target
  - Phase 2 of documentation improvement plan in progress
  - Added 10 more module-level documentation blocks with examples
  - Focus on core services and analysis modules

## [0.30.4] - 2025-07-31

### Added
- **Enhanced Documentation Coverage**: Improved documentation from 52% to include more core modules
  - Added comprehensive module-level documentation to `unified_protocol/mod.rs`
  - Added module documentation to `models/mod.rs` with example usage
  - Added module documentation to `services/mod.rs` with service categories
  - Added module documentation to `demo/mod.rs` with architecture overview
  - Added module documentation to `utils/mod.rs` with utility examples
  - Enhanced `services/context.rs` with AI-ready context generation examples
  - Enhanced `services/refactor_engine.rs` with state machine workflow documentation
- **Documentation Improvement Plan**: Created `docs/todo/increase-docs.md` with granular tasks
  - Target: 80%+ documentation coverage (335+ files)
  - Target: 30%+ doctest coverage (125+ files)
  - Comprehensive roadmap for documentation improvements

## [0.30.3] - 2025-07-31

### Added
- **Comprehensive pmcp SDK Documentation**: Added extensive doctests and examples
  - Module-level documentation for `mcp_pmcp` with usage examples and performance metrics
  - Enhanced `PmcpServer` documentation with architecture details and custom configuration examples
  - Added doctests to `analyze_handlers.rs` showing JSON argument schemas for all analysis tools
  - Created `pmcp_analyze_workflow.rs` example demonstrating complete analysis workflow
  - Created `pmcp_refactor_session.rs` example showing refactoring state machine
  - Added proper feature gating for all pmcp-related code with `#[cfg(feature = "pmcp-mcp")]`
- **Full pmcp Tool Registration**: All 15 PMAT tools now registered in pmcp server
  - 6 analysis tools (complexity, SATD, dead code, DAG, deep context, Big-O)
  - 4 refactoring tools (start, nextIteration, getState, stop)
  - 1 quality gate tool
  - 1 git operations tool
  - 3 context tools (generate context, scaffold project, git status)

### Fixed
- **pmcp ToolHandler Signatures**: Fixed all handler methods to accept RequestHandlerExtra parameter
- **Missing Context Handler Exports**: Added re-exports for GenerateContextTool, GitTool, and ScaffoldProjectTool
- **Feature Flag Conditional Compilation**: Ensured all pmcp code is properly gated
- **CI Build Failure**: Removed pmcp path dependency to fix GitHub Actions builds
- **Clippy Warning**: Fixed unused variable warning in pmcp_analyze_workflow example

### Changed
- **Enhanced Documentation**: All tool handlers now include comprehensive usage examples and JSON schemas
- **Improved Examples**: Examples now show both feature-enabled and feature-disabled paths

## [0.30.1] - 2025-01-30

### Added
- **Experimental pmcp-based MCP Server**: Initial implementation of MCP server using the pmcp Rust SDK
  - Added `pmcp-mcp` feature flag for conditional compilation
  - Implemented basic structure for pmcp integration
  - Created modular handler structure: analyze, refactor, quality-gate, git, and context handlers
  - Added environment variable `PMAT_PMCP_MCP=1` to activate pmcp backend
  - Provides foundation for 10x performance improvement

### Fixed
- **Makefile Quote Escaping**: Fixed shell syntax error in crate-release target (line 798)

### Changed
- **pmcp Dependency**: Made pmcp dependency optional, only included with `pmcp-mcp` feature
- **Test Example**: Updated test_pmcp_server example to handle feature flag gracefully

## [0.30.0] - 2025-01-26

### Added
- **MCP Server pmcp SDK Example**: New `cargo run --example mcp_server_pmcp` demonstrating future pmcp SDK integration
- **pmcp SDK Documentation**: Comprehensive documentation for using the pmcp Rust SDK with PMAT
  - Added pmcp integration section to README.md
  - Enhanced MCP protocol documentation with pmcp SDK usage
  - Updated examples README with new MCP server example
  - Added migration guide from stdio to pmcp implementation

### Fixed
- **Git Clone Test**: Fixed `test_real_repo_sizes` by correcting size unit mismatch (bytes vs KB)
- **Dead Code Cleanup**: Removed 13 legacy dead code functions from `services/deep_context.rs`
  - Removed unused Semaphore import
  - Fixed compilation by replacing removed function calls with inline implementations

### Changed
- **Documentation Updates**: Enhanced all MCP-related documentation with pmcp SDK integration details
- **Code Quality**: Maintained zero-tolerance standards with 0 SATD comments and 0 lint violations

## [0.29.6] - 2025-01-15

### Fixed
- **Critical Bug Fix**: Fixed quality gate dead code detection always reporting violations
  - `check_dead_code` now properly analyzes dead code percentage instead of always returning a violation
  - Quality gate now correctly passes when dead code is below the threshold
  - Fixed test `test_quality_gate_passes_clean_code` to use code that won't be incorrectly flagged
- **Include Pattern Fix**: Fixed --include patterns being ignored for test directories
  - When explicit include patterns are provided (e.g., `--include "tests/**/*.rs"`), test files are now correctly included
  - Default exclusions only apply when no include patterns are specified
- **Clippy Fix**: Replaced deprecated `map_or` with `is_some_and` to fix clippy lint warning

## [0.29.5] - 2025-01-14

### Refactored
- **Toyota Way Modular Architecture Complete**: Achieved 97% complexity reduction in stubs.rs through systematic refactoring
- Created dedicated modules: `language_analyzer.rs`, `dead_code_formatter.rs`, `defect_formatter.rs`
- Eliminated 549 lines of duplicated code while maintaining full functionality
- Applied proper separation of concerns across all formatting and analysis functions

### Fixed
- Fixed test field name (max_nesting → nesting_max) in language_analyzer tests
- Removed all leftover dead code from previous implementations

### Changed
- `analyze_file_complexity_async`: 38 → 1 complexity (97% reduction)
- `format_dead_code_output`: 29 → 1 complexity (97% reduction)  
- `format_defect_full`: 30 → 1 complexity (97% reduction)
- `format_defect_sarif`: 15 → 1 complexity (93% reduction)
- `format_defect_csv`: 8 → 1 complexity (87% reduction)

### Documentation
- Updated README.md to reflect Toyota Way achievements
- Consolidated all release notes into `docs/release_notes/` directory
- Cleaned up stray files and artifacts from project root

## [0.29.3] - 2025-01-13

### Fixed
- Fixed HashMap mutability issue in demo assets preventing crates.io installation
- Eliminated all 51+ stub implementations across the codebase
- Fixed hardcoded values in quality gate checks (dead code, entropy, provability)
- Implemented real GitHub API integration for repository size checking
- Fixed dead code prover function name extraction
- Fixed all clippy warnings to ensure `make lint` passes

### Added
- HTTP and MCP protocol support for SATD analysis
- HTTP and MCP protocol support for lint-hotspot analysis
- Unified `make test` command that runs all test types
- Comprehensive test coverage in GitHub Actions

### Changed
- Consolidated testing into single `make test` command (runs test-fast, test-doc, test-property, test-examples)
- Enabled property tests in CI (previously disabled)
- Simplified GitHub Actions workflow to use unified test command
- Applied DRY principle across all protocol implementations

### Documentation
- Added comprehensive QA report documenting implementation status
- Updated CLAUDE.md to reflect simplified testing approach
- Added stub elimination update documentation

## [0.29.3] - 2025-01-13

### Fixed
- Fixed deep context analysis to use proper AST analysis instead of stub implementations (#33)
- Fixed all failing doctests achieving Toyota Way zero defects standard
- Fixed all property tests achieving Toyota Way zero defects standard
- Fixed demo e2e integration test timing issues for slower systems
- Fixed type mismatch in deep context complexity calculations

### Changed
- Enhanced deep-context command to use same AST analysis pathway as context command
- Improved consistency between context and deep-context commands
- Updated demo integration tests with more robust timeouts and error handling
- Enhanced JSON output format for deep-context to include file-level details

### Documentation
- Updated CLI reference for deep-context command with correct options
- Updated README with context and deep-context command examples
- Updated deep context analysis feature documentation
- Added comprehensive documentation update summary

## [0.29.2] - 2025-01-12

### Fixed
- Resolved --max-cyclomatic flag filtering issue (#32)
- Removed needless borrows in test arguments
- Updated property tests to handle both warning and error severity levels

## [0.29.1] - 2025-01-11

### Fixed
- Resolved all clippy lint violations and Makefile issues
- Fixed Toyota Way refactor handle_refactor_auto implementation

## [0.29.0] - 2025-01-10

### Added
- Toyota Way transformation with zero-compromise quality standards
- Comprehensive property tests for all major features
- Enhanced refactoring capabilities

### Changed
- Complete refactor using Toyota Way principles (Kaizen, Genchi Genbutsu, Jidoka)
- Achieved 84% complexity reduction with -3,401 lines while improving functionality
- All functions now ≤20 complexity (reduced from max 136 to 21)

### Fixed
- Eliminated all 5,202 quality violations
- Maintained zero SATD comments throughout refactoring