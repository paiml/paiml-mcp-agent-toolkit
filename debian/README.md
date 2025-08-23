# PMAT Debian/Ubuntu Package

This directory contains the complete Debian package structure for PMAT v2.12.0.

## 📦 Package Structure

```
debian/
├── DEBIAN/
│   ├── control          # Package metadata and dependencies
│   ├── postinst         # Post-installation script with user guidance
│   ├── prerm            # Pre-removal script (service cleanup)
│   └── postrm           # Post-removal script (purge cleanup)
├── etc/pmat/            # Configuration templates
│   ├── agent-development.toml
│   ├── agent-production.toml
│   └── agent-ci.toml
├── lib/systemd/system/  # systemd service file
│   └── pmat-agent.service
├── usr/share/doc/pmat/  # Documentation
│   ├── copyright        # Debian copyright format
│   ├── README.Debian    # Debian-specific documentation
│   ├── changelog.Debian.gz # Package changelog
│   └── CLAUDE_CODE_AGENT.md # User guide
└── usr/share/man/man1/  # Manual page
    └── pmat.1.gz        # Compressed man page
```

## 🏗️ Building the Package

### Quick Build
```bash
cd debian
./build-deb.sh
```

### Manual Build  
```bash
# Set correct permissions
find . -type d -exec chmod 755 {} \;
find . -type f -exec chmod 644 {} \;
chmod +x DEBIAN/{postinst,prerm,postrm}

# Compress documentation
gzip -9 usr/share/doc/pmat/changelog.Debian
gzip -9 usr/share/man/man1/pmat.1

# Build package
cd .. && dpkg-deb --build debian pmat_2.12.0_amd64.deb
```

## 🧪 Testing

### Local Testing
```bash
./test-deb.sh
```

### Manual Testing
```bash
# Install package
sudo dpkg -i ../pmat_2.12.0_amd64.deb

# Fix dependencies if needed
sudo apt-get install -f

# Test functionality
pmat --version
systemctl status pmat-agent

# Remove package
sudo apt remove pmat

# Complete purge
sudo apt purge pmat
```

## 📋 Package Features

### Installation Strategy
The package provides a **framework installation** rather than binary installation:

1. **Service Setup**: Creates systemd service, user account, directories
2. **Configuration**: Installs templates and documentation  
3. **Integration**: Prepares system for PMAT usage
4. **Guidance**: Provides clear installation instructions for the actual binary

### Binary Installation Methods
Post-package installation, users choose their preferred method:

- **Cargo** (Recommended): `cargo install pmat`
- **npm**: `npm install -g pmat-agent` 
- **Docker**: `docker run paiml/pmat:latest`

### Service Integration
- **systemd service**: `/lib/systemd/system/pmat-agent.service`
- **Service user**: `pmat` system account
- **State directory**: `/var/lib/pmat-agent/`
- **Log directory**: `/var/log/pmat-agent/`

## 🎯 Installation Experience

### User Flow
1. **Install package**: `sudo apt install pmat` (once in repository)
2. **See guidance**: Package provides clear next-step instructions
3. **Install binary**: Choose preferred method (Cargo/npm/Docker)
4. **Configure**: Edit templates in `/etc/pmat/` if using service mode
5. **Use**: Run `pmat` commands or enable service

### Claude Code Integration
After binary installation:
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

## 📊 Package Details

- **Package Name**: `pmat`
- **Version**: `2.12.0`
- **Architecture**: `amd64` (expandable to `arm64`)
- **Section**: `devel` (development tools)
- **Priority**: `optional`
- **Installed Size**: ~8MB
- **Dependencies**: Minimal system libraries only
- **Suggests**: `cargo`, `nodejs`, `npm`, `git`

## 🚀 Distribution Options

### 1. Local Installation
```bash
sudo dpkg -i pmat_2.12.0_amd64.deb
```

### 2. APT Repository
Upload to private APT repository and install via:
```bash
sudo apt update && sudo apt install pmat
```

### 3. Ubuntu PPA
Submit to Personal Package Archive (see `SUBMIT_TO_UBUNTU_PPA.md`)

### 4. Debian Official Repository
Long-term goal: Submit to Debian official repositories

## 🔧 Maintenance

### Version Updates
1. Update `DEBIAN/control` version
2. Update `usr/share/doc/pmat/changelog.Debian`  
3. Rebuild with `./build-deb.sh`
4. Test with `./test-deb.sh`

### Configuration Updates  
- Templates in `etc/pmat/` can be updated
- Service file in `lib/systemd/system/` can be modified
- Documentation in `usr/share/doc/pmat/` can be enhanced

## 📞 Support

- **Package Issues**: Check `/usr/share/doc/pmat/README.Debian`
- **PMAT Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Debian Packaging**: https://www.debian.org/doc/manuals/maint-guide/

---

**Ready for distribution!** Complete Debian package with service integration, comprehensive documentation, and user-friendly installation experience.