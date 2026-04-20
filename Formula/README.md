# Homebrew Formula for PMAT

This directory contains the Homebrew formula for installing PMAT on macOS and Linux.

## Installation

### From Homebrew Core (Future)
```bash
# Once accepted into homebrew-core
brew install pmat
```

### From Tap (Current)
```bash
# Add our tap
brew tap paiml/pmat https://github.com/paiml/paiml-mcp-agent-toolkit

# Install PMAT
brew install pmat

# Verify installation
pmat --version
```

## Usage After Installation

```bash
# Start Claude Code Agent Mode
pmat agent mcp-server

# Analyze code quality
pmat quality-gate --strict

# Generate AI context
pmat context
```

## Submitting to Homebrew Core

To submit this formula to the official Homebrew repository:

1. **Fork homebrew-core**: https://github.com/Homebrew/homebrew-core
2. **Calculate SHA256**: 
   ```bash
   curl -L https://github.com/paiml/paiml-mcp-agent-toolkit/archive/v2.12.0.tar.gz | shasum -a 256
   ```
3. **Update formula**: Replace SHA256_PLACEHOLDER with actual hash
4. **Test formula**:
   ```bash
   brew install --build-from-source pmat.rb
   brew test pmat
   ```
5. **Submit PR**: Create pull request to homebrew-core

## Formula Requirements Met

✅ **Stable URL**: GitHub release tarball  
✅ **License**: MIT specified  
✅ **Dependencies**: Only Rust build dependency  
✅ **Tests**: Version check and basic functionality  
✅ **Build System**: Standard Cargo build  
✅ **Documentation**: Clear description and homepage