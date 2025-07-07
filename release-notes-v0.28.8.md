# Release v0.28.8

## 🚀 New Features

### Technical Debt Gradient (TDG) Analysis - Full Interface Parity
- **Complete Implementation**: Replaced stub TDG implementation with full TDGCalculator integration
- **Three Analysis Modes**: 
  - Single file analysis for focused inspection
  - Multiple files mode for MCP tool composition 
  - Project-wide analysis with pattern filtering
- **All Output Formats**: Table, JSON, Markdown, and SARIF formats fully supported
- **MCP Tool Chaining**: TDG now supports the same interface as complexity analysis, enabling AI agents to chain analysis tools effectively

## 🐛 Bug Fixes

### Clippy Lint Issues Resolved
- Fixed `too_many_arguments` warnings by adding appropriate allows
- Changed `&PathBuf` to `&Path` parameters following clippy recommendations
- Fixed dead code warnings in MCP property tests
- Resolved all Rust linting issues - `make lint` now passes ✅

### Doctest Fixes
- Updated TDG handler doctests to match new 12-parameter signature
- Fixed Complexity command doctests to include file/files parameters
- Fixed comprehensive handler doctests for interface consistency
- Corrected Defect model doctests

## 📚 Documentation Updates

### Interface Parity Requirements
- Added comprehensive documentation for handler interface requirements
- All analysis handlers must support three modes: single file, multi-file MCP, and project-wide
- Updated example-create-prompt.md with interface parity checklist
- Added validation steps to ensure consistent handler interfaces

## 🔧 Technical Improvements

### Code Quality
- Zero tolerance for stub implementations - all features fully functional
- Consistent parameter patterns across all analysis handlers
- Improved error handling and progress indicators
- Enhanced MCP tool composition support

## 📦 Installation

```bash
# From crates.io
cargo install pmat

# From source
git clone https://github.com/paiml/paiml-mcp-agent-toolkit
cd paiml-mcp-agent-toolkit
cargo build --release
```

## 🔄 Breaking Changes

None - this release maintains backward compatibility while adding new functionality.

## 🙏 Acknowledgments

This release follows the Toyota Way principles:
- **Kaizen**: Continuous incremental improvement
- **Jidoka**: Quality built into every step
- **Zero Defects**: No compromises on quality

---

🤖 Generated with [Claude Code](https://claude.ai/code)

Co-Authored-By: Claude <noreply@anthropic.com>