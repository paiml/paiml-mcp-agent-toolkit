# Release Notes

## v0.26.2 (2025-07-02)

### New Features

#### 🚀 WebAssembly and AssemblyScript Support
- Added comprehensive WebAssembly analysis capabilities
- Two new analysis commands:
  - `pmat analyze assemblyscript` - Analyze AssemblyScript source code
  - `pmat analyze webassembly` - Analyze WebAssembly binary and text formats
- Features include:
  - **Complexity Analysis**: WASM-specific metrics including gas estimation
  - **Security Validation**: Basic security checks for WebAssembly modules
  - **Memory Analysis**: Memory pool management and optimization hints
  - **Parallel Processing**: Efficient analysis of multiple WASM files
  - **Multiple Output Formats**: JSON, SARIF, Markdown, and summary formats
- Supports file extensions: `.wasm` (binary), `.wat` (text), `.ts` (AssemblyScript)

#### 🔧 Language Support Expansion
- Added Language enum variants: `AssemblyScript = 14`, `WebAssembly = 15`
- Integrated with unified AST system for consistent analysis
- Memory-safe parsing with timeout protection and file size limits

### Improvements
- Enhanced language detection to support WebAssembly file types
- Added WASM-specific complexity metrics (cyclomatic, cognitive, memory pressure)
- Implemented iterative parsing to prevent stack overflow on large files

### Bug Fixes
- Fixed compilation issues with unused imports and variables
- Resolved pattern matching exhaustiveness in test files
- Fixed field assignment clippy warnings

## v0.26.1 (2025-07-02)

### New Features

#### 🎯 Property-Based Testing Integration
- Added property-based testing support to `pmat refactor auto`
- Automatically generates property tests that verify behavior preservation during refactoring
- Integrates with QuickCheck and proptest frameworks
- Ensures refactored code maintains semantic equivalence

#### 🚀 Refactor Auto UX Improvements
- Enhanced `pmat refactor auto` output with clear instructions
- Now displays actionable next steps after generating refactoring requests
- Explicitly states 80% test coverage requirement
- Improved user guidance for successful refactoring workflow

#### 🧹 AI-Assisted Documentation Cleanup (`pmat refactor docs`)
- New command for maintaining Zero Tolerance Quality Standards in documentation
- Identifies and removes temporary files, outdated status reports, and build artifacts
- Features:
  - Pattern-based detection with customizable patterns
  - Interactive mode for reviewing each file before removal
  - Dry run mode to preview changes
  - Automatic backup before removing files
  - Age-based filtering (only remove files older than X days)
  - Preservation patterns to protect important files
- Default patterns include:
  - Temporary scripts: `fix-*`, `test-*`, `temp-*`, `tmp-*`
  - Status files: `*_STATUS.md`, `*_PROGRESS.md`, `*_COMPLETE.md`
  - Build artifacts: `*.mmd`, `optimization_state.json`, `complexity_report.json`

#### 🔥 EXTREME Quality Lint Hotspot Analysis
- `pmat analyze lint-hotspot` now uses EXTREME quality standards by default
- Automatically includes:
  - `-D warnings` - Zero tolerance for all warnings
  - `-D clippy::pedantic` - Strictest built-in lint group
  - `-D clippy::nursery` - Experimental lints
  - `-D clippy::cargo` - Cargo.toml manifest lints
  - `--all-targets` - Lints library, binaries, tests, and examples

### Improvements

#### 🏗️ Major Complexity Reduction Refactoring
- Refactored high-complexity functions achieving 95%+ complexity reduction
- `handle_analyze_dead_code`: Cognitive complexity 244 → 10 (96% reduction)
- `format_quality_gate_output`: Complexity 136 → 5 (96.3% reduction)
- `handle_analyze_churn`: Complexity 97 → 5 (94.8% reduction)
- Established refactoring pattern: main function delegates to focused helper functions
- All refactored code includes comprehensive test suites with >80% coverage

#### 📚 Documentation Organization
- Reorganized entire documentation structure following Zero Tolerance Quality Standards
- Implemented strict directory structure rules:
  - No loose markdown files in `docs/` root
  - Bug tracking in `docs/bugs/active/` and `docs/bugs/archived/`
  - Specifications in `docs/todo/active/` and `docs/todo/archive/`
  - Removed duplicate directories (kaizen, system-status)
- Added comprehensive documentation-organization-spec.md

### Bug Fixes
- Fixed compilation errors in refactor handlers
- Fixed test failures related to AST parsing
- Updated binary size limits to accommodate new features

### Technical Debt
- Achieved Zero SATD (Self-Admitted Technical Debt)
- All functions now below complexity threshold of 20
- Removed all temporary and work-in-progress files

## v0.26.0 (2025-01-20)

### New Features
- Complete Kotlin language support with memory-safe parsing
- 11 additional analysis features including graph metrics, name similarity, and symbol table
- ML-based defect prediction
- Lightweight formal verification (provability analysis)
- Incremental coverage tracking with git diff integration

### Known Limitations
- `pmat context` command may timeout on very large codebases (>1000 files) with AST analysis enabled
  - Workaround: Use specific paths or exclude patterns

## Previous Releases
See git history for details on earlier releases.