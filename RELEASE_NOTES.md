# Release Notes

## v0.26.1 (Unreleased)

### New Features

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