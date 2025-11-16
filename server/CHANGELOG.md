# Changelog

All notable changes to PMAT (Pragmatic AI Labs MCP Agent Toolkit) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Rust Project Score v1.1 - Evidence-based 106-point scoring system
  - 6 category analyzers (Code Quality, Testing, Documentation, Performance, Dependencies, Rust Tooling)
  - CLI command: `pmat rust-project-score`
  - Dual-mode operation: fast mode (default) and --full mode
  - Output formats: text, json, markdown, yaml
  - Actionable recommendations based on analysis
- cargo-deny configuration (deny.toml) for dependency policy enforcement
- Comprehensive rustfmt formatting across codebase

### Changed
- Applied rustfmt to 145 files for consistent code formatting

## [2.98.3] - 2024

### Added
- Multi-language support improvements
- Bug fixes for language detection

## [2.98.2] - 2024

### Added
- Enhanced language analysis capabilities

## [2.98.1] - 2024

### Added
- Quality gate improvements
- MCP tools enhancements

## [2.98.0] - 2024

### Added
- Advanced analysis features
- Quality tracking capabilities

## [2.97.0] - 2024

### Added
- Organizational intelligence integration
- Red team demo capabilities

---

**Note**: This CHANGELOG was initiated on 2025-11-16 during Rust Project Score v1.1 implementation.
Prior version history has been reconstructed from git tags. For detailed commit history,
see: https://github.com/paiml/paiml-mcp-agent-toolkit/commits/master
