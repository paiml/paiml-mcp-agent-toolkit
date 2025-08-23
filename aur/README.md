# PMAT Arch Linux AUR Package

This directory contains the AUR (Arch User Repository) package for installing PMAT on Arch Linux and derivatives.

## Installation

### Via AUR Helper (Recommended)
```bash
# Using yay
yay -S pmat

# Using paru
paru -S pmat

# Using pamac
pamac install pmat
```

### Manual Installation
```bash
# Clone AUR repository
git clone https://aur.archlinux.org/pmat.git
cd pmat

# Build and install
makepkg -si
```

## Package Contents

After installation:
- **Binary**: Available after separate cargo/npm installation
- **systemd Service**: `/usr/lib/systemd/system/pmat-agent.service`
- **Configuration**: `/etc/pmat/` (templates for agent modes)
- **Documentation**: `/usr/share/doc/pmat/`
- **Man Page**: `man pmat`

## Claude Code Integration

After installing the binary via cargo or npm:

```json
{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["agent", "mcp-server"],
      "env": {}
    }
  }
}
```

## Service Mode

For continuous monitoring:
```bash
# Enable and start service
sudo systemctl enable pmat-agent
sudo systemctl start pmat-agent

# Check status
sudo systemctl status pmat-agent
```

## Binary Installation

The AUR package provides system integration. Install the actual binary via:

```bash
# Option 1: Rust/Cargo (recommended)
sudo pacman -S rust
cargo install pmat

# Option 2: npm (Node.js)
sudo pacman -S nodejs npm
npm install -g pmat-agent

# Option 3: Docker
sudo pacman -S docker
docker run --rm paiml/pmat:latest pmat --version
```

## Files

- **PKGBUILD**: Arch Linux package build script
- **SUBMIT_TO_AUR.md**: Submission guide for maintainers
- **test-package.sh**: Local testing script
- **submit-to-aur.sh**: Automated submission script

## Support

- **AUR Issues**: Comments on [AUR package page](https://aur.archlinux.org/packages/pmat)
- **PMAT Issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- **Documentation**: [User Guide](https://github.com/paiml/paiml-mcp-agent-toolkit/blob/master/docs/CLAUDE_CODE_AGENT.md)

---

**Maintainer**: Pragmatic AI Labs  
**License**: MIT  
**Homepage**: https://github.com/paiml/paiml-mcp-agent-toolkit