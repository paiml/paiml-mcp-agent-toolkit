# Submitting PMAT to Arch Linux AUR

This guide covers submitting PMAT v2.10.0 to the Arch User Repository (AUR).

## ✅ Prerequisites Met

- [x] **AUR Account**: SSH key registered with AUR
- [x] **Stable Release**: v2.10.0 tagged and released on GitHub  
- [x] **License**: MIT license (AUR compatible)
- [x] **Build System**: Standard Cargo build process
- [x] **SHA256**: Calculated and verified (`d8aa8ade82d3c877fd140327ee64c51d9a00d91b97c5f6195c54550ca1b8c4a0`)
- [x] **Package Files**: PKGBUILD and .SRCINFO ready
- [x] **Dependencies**: Only makedepends (rust, cargo)

## 🚀 Submission Process

### 1. Clone AUR repository
```bash
# Clone the AUR repository (will create new one if first submission)
git clone ssh://aur@aur.archlinux.org/pmat.git aur-pmat
cd aur-pmat

# If package doesn't exist yet, initialize empty repo
git init
git remote add origin ssh://aur@aur.archlinux.org/pmat.git
```

### 2. Copy package files
```bash
# Copy our prepared files
cp ../paiml-mcp-agent-toolkit/aur/PKGBUILD .
cp ../paiml-mcp-agent-toolkit/aur/.SRCINFO .

# Verify files are correct
cat PKGBUILD
cat .SRCINFO
```

### 3. Test package locally
```bash
# Validate PKGBUILD
namcap PKGBUILD

# Test build (optional but recommended)
makepkg -si

# Test installed binary
pmat --version
pmat agent --help

# Clean up test
sudo pacman -R pmat
```

### 4. Submit to AUR
```bash
# Add files
git add PKGBUILD .SRCINFO

# Commit with descriptive message
git commit -m "pmat: initial upload - v2.10.0

PMAT is a zero-config AI context generation and code quality toolkit 
with Claude Code Agent Mode for continuous quality monitoring.

Features:
- Claude Code Agent Mode with MCP protocol integration
- AI context generation optimized for LLM workflows  
- Code complexity analysis with Toyota Way standards
- Technical debt detection and quality gates
- Multi-language support (30+ languages)
- Production-ready systemd service

Homepage: https://github.com/paiml/paiml-mcp-agent-toolkit"

# Push to AUR
git push origin master
```

## 📋 Package Description

**PMAT (Pragmatic AI MCP Agent Toolkit)** - Zero-config AI context generation and code quality toolkit with Claude Code Agent Mode.

### Key Features:
- **Claude Code Agent Mode**: Full MCP protocol integration for AI agents
- **Context Generation**: Optimized for LLM workflows and AI development
- **Quality Analysis**: Code complexity, technical debt, and quality gates
- **Multi-Language**: 30+ languages via tree-sitter parsing
- **Production Ready**: systemd service, configuration templates, monitoring

### Technical Details:
- **Build**: Standard Cargo/Rust build system
- **Dependencies**: rust, cargo (makedepends only)
- **Architecture**: x86_64, aarch64
- **Install Size**: ~8MB binary + docs
- **Config**: `/etc/pmat/` templates for agent modes

## 🔍 AUR Guidelines Compliance

- **Naming**: `pmat` follows AUR naming conventions
- **Version**: Follows upstream semantic versioning (2.10.0)
- **License**: MIT (AUR-compatible open source license)
- **Dependencies**: Minimal (build-only rust/cargo)
- **Files**: Standard PKGBUILD structure with proper .SRCINFO
- **Source**: Official GitHub release tarball with verified SHA256

## 📦 Package Contents

After installation via `yay -S pmat` or `makepkg -si`:

```
/usr/bin/pmat                           # Main binary
/usr/share/licenses/pmat/LICENSE        # MIT license
/usr/share/doc/pmat/README.md          # Documentation  
/usr/share/doc/pmat/CLAUDE_CODE_AGENT.md # Agent mode guide
/etc/pmat/agent-development.toml       # Dev config template
/etc/pmat/agent-production.toml        # Prod config template  
/etc/pmat/agent-ci.toml                # CI config template
/usr/lib/systemd/system/pmat-agent.service # systemd service (optional)
```

## 🎯 Post-Submission

Once submitted to AUR:
- **Immediate**: Package available via AUR helpers (`yay -S pmat`)
- **Community**: Arch Linux users can install with minimal setup
- **Integration**: One-command Claude Code Agent Mode setup
- **Automation**: AUR updates follow upstream releases

## 🔧 Maintenance

### Updating Package
1. Update `pkgver` in PKGBUILD
2. Update `sha256sums` with new release checksum  
3. Regenerate .SRCINFO: `makepkg --printsrcinfo > .SRCINFO`
4. Test build and commit changes
5. Push update to AUR

### Version Update Command
```bash
# For next release (e.g., v2.11.0)
updpkgsums  # Updates checksums automatically
makepkg --printsrcinfo > .SRCINFO
git add PKGBUILD .SRCINFO
git commit -m "pmat: update to 2.11.0"
git push origin master
```

## 📞 Support

- **AUR Issues**: Comments on AUR package page
- **PMAT Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues  
- **Build Problems**: Check PKGBUILD or contact maintainer
- **AUR Guidelines**: https://wiki.archlinux.org/title/AUR_submission_guidelines