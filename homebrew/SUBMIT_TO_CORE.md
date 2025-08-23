# Submitting PMAT to homebrew-core

This guide covers submitting the PMAT formula to the official Homebrew repository.

## ✅ Prerequisites Met

- [x] **Stable Release**: v2.10.0 tagged and released on GitHub
- [x] **License**: MIT license (Homebrew compatible) 
- [x] **Build System**: Standard Cargo build process
- [x] **Tests**: Comprehensive test suite included
- [x] **SHA256**: Calculated and verified
- [x] **Dependencies**: Only build dependency (Rust)
- [x] **Documentation**: Complete with homepage and description

## 🚀 Submission Process

### 1. Fork homebrew-core
```bash
# Fork the repository on GitHub
open https://github.com/Homebrew/homebrew-core

# Clone your fork
git clone https://github.com/YOUR_USERNAME/homebrew-core.git
cd homebrew-core
```

### 2. Create the formula
```bash
# Create formula in correct location
mkdir -p Formula
cp /path/to/pmat.rb Formula/pmat.rb

# Verify formula meets standards
brew audit --strict Formula/pmat.rb
```

### 3. Test the formula
```bash
# Test installation
brew install --build-from-source Formula/pmat.rb

# Test functionality
pmat --version
pmat agent --help

# Test uninstall
brew uninstall pmat
```

### 4. Submit pull request
```bash
# Create branch
git checkout -b add-pmat-2.10.0

# Commit formula
git add Formula/pmat.rb
git commit -m "pmat 2.10.0 (new formula)

PMAT is a zero-config AI context generation and code quality toolkit 
with Claude Code Agent Mode for continuous quality monitoring."

# Push and create PR
git push origin add-pmat-2.10.0
```

## 📋 PR Description Template

```markdown
## PMAT 2.10.0 (new formula)

PMAT (Pragmatic AI MCP Agent Toolkit) is a zero-configuration AI context generation system with Claude Code Agent Mode for continuous quality monitoring.

### Features:
- Claude Code Agent Mode with MCP protocol integration  
- AI context generation optimized for LLM workflows
- Code complexity analysis with Toyota Way standards
- Technical debt detection and quality gates
- Multi-language support (30+ languages via tree-sitter)
- Production-ready systemd service deployment

### Formula Details:
- **Build system**: Cargo (standard Rust)
- **Dependencies**: rust (build only)
- **License**: MIT
- **Tests**: Version check and basic functionality
- **Platforms**: macOS and Linux

### Verification:
- [x] `brew audit --strict Formula/pmat.rb` passes
- [x] `brew install --build-from-source Formula/pmat.rb` succeeds
- [x] `brew test pmat` passes
- [x] Binary works correctly: `pmat --version` shows 2.10.0

### Links:
- Homepage: https://github.com/paiml/paiml-mcp-agent-toolkit
- Documentation: https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/CLAUDE_CODE_AGENT.md
- npm package: https://www.npmjs.com/package/pmat-agent
- Docker: https://hub.docker.com/r/paiml/pmat
```

## 🔍 Review Criteria

Homebrew maintainers will check:

1. **Notability**: PMAT provides unique AI/code quality functionality
2. **Stability**: v2.10.0 is a stable release with comprehensive testing
3. **Build**: Standard Cargo build process (well-supported)
4. **Tests**: Formula includes appropriate test cases
5. **Naming**: "pmat" follows Homebrew naming conventions
6. **Dependencies**: Minimal (only build-time Rust dependency)

## ⚡ Expected Timeline

- **Submission**: Immediate (ready now)
- **Initial Review**: 1-7 days
- **Feedback/Iterations**: 1-2 weeks
- **Approval**: 2-4 weeks total

## 🎯 Success Metrics

Once approved:
- `brew install pmat` available globally
- 5M+ macOS/Linux developers can access PMAT
- Claude Code integration becomes one-command setup
- Enhanced credibility through official Homebrew inclusion

## 📞 Support

- **Formula Issues**: GitHub Issues on homebrew-core PR
- **PMAT Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Homebrew Guidelines**: https://docs.brew.sh/Formula-Cookbook