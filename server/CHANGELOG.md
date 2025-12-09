# Changelog

All notable changes to PMAT (Pragmatic AI Labs MCP Agent Toolkit) will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [2.211.0] - 2024-12-09

### Added
- **Terminal Graph Visualization** (trueno-viz integration)
  - `pmat tdg --viz` flag for rendering dependency graphs in terminal
  - Force-directed layout (Fruchterman-Reingold algorithm)
  - PageRank-based criticality scoring for function importance
  - ANSI TrueColor rendering (16.7M colors)
  - Multiple themes: default, high-contrast, light, colorblind-safe (Okabe-Ito palette)
  - Accessibility-focused dual encoding (shape + color) for WCAG 2.1 compliance
  - New example: `cargo run --example viz_demo --features viz`
- CLI flags: `--viz` and `--viz-theme <THEME>` for TDG command

### Performance
- O(1) function lookups via CSR-backed TdgGraph
- Semantic zooming limits display to top N nodes by PageRank criticality

## [2.210.0] - 2024-12-08

### Added
- Oracle PDCA Loop & Rich Reporting (v2.210.0 release)
- OIP first-class plugin integration
- Demo and book quality scoring specification

## [2.209.0] - 2024

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

### Performance
- **Kaizen optimization rounds 1-3**: 996x performance improvement for rust-project-score
  - Round 1: Implemented ScoringMode enum architecture
  - Round 2: Fixed TestingScorer + RustToolingScorer subprocess bottleneck (229s → 63s, 72.5% faster)
  - Round 3: Fixed CodeQualityScorer subprocess bottleneck (63s → 0.23s, 99.9% total improvement)
  - Eliminated 212 of 213 subprocess poll syscalls (99.5% reduction)
  - Final result: **3m 49s → 230ms** for comprehensive project analysis

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
