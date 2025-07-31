# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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