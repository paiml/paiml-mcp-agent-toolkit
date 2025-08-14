# Release Notes Index

This directory contains all release notes for the PAIML MCP Agent Toolkit (pmat).

## Latest Releases

### [v0.29.4 - Toyota Way Modular Architecture Complete](RELEASE_NOTES_v0.29.4.md)
**January 14, 2025**
- 97% complexity reduction through modular refactoring
- Created dedicated modules: language_analyzer.rs, dead_code_formatter.rs, defect_formatter.rs
- Eliminated 549 lines of duplicated code
- Maintained zero tolerance quality standards

### [v0.29.2 - Critical HashMap Fix](RELEASE_NOTES_v0.29.2.md)
**January 13, 2025**
- Fixed critical HashMap mutability issue preventing crates.io installation
- Improved installation reliability

### [v0.29.1 - Quality Improvements](RELEASE_NOTES_v0.29.1.md)
**January 11, 2025**
- Quality gate enhancements
- Documentation updates

## Previous v0.28.x Series

### [v0.28.14](RELEASE_NOTES_v0.28.14.md) - Final v0.28 Release
### [v0.28.13](RELEASE_NOTES_v0.28.13.md) - Bug Fixes
### [v0.28.12](RELEASE_NOTES_v0.28.12.md) - Performance Improvements
### [v0.28.9](RELEASE_NOTES_v0.28.9.md) - Enhanced Analysis
### [v0.28.8](release-notes-v0.28.8.md) - Feature Updates
### [v0.28.7](release-notes-v0.28.7.md) - Stability Improvements
### [v0.28.6](RELEASE_NOTES_v0.28.6.md) - Core Features
### [v0.28.4](RELEASE_NOTES_v0.28.4.md) - Foundation Release
### [v0.28.2](RELEASE_NOTES_v0.28.2.md) - Early Release
### [v0.28.1](RELEASE_NOTES_v0.28.1.md) - Initial v0.28

## Release Summaries

- [Overall Release Summary](RELEASE_SUMMARY.md) - Comprehensive overview
- [v0.28.4 Summary](RELEASE_SUMMARY_v0.28.4.md) - Specific release highlights

## Installation

Latest stable version:
```bash
cargo install pmat
```

Development version:
```bash
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit
cargo build --release
```

## Quality Standards

All releases follow Toyota Way principles:
- Zero tolerance for technical debt
- Comprehensive property testing
- Zero failing doctests
- Zero SATD comments
- Proper separation of concerns