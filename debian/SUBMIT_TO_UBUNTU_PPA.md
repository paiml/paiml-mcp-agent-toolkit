# Submitting PMAT to Ubuntu PPA

This guide covers submitting PMAT to an Ubuntu Personal Package Archive (PPA) for easy installation on Ubuntu systems.

## ✅ Prerequisites Met

- [x] **Debian Package**: Complete .deb package with all required files
- [x] **GPG Key**: Required for signing packages
- [x] **Launchpad Account**: Ubuntu PPA hosting requires Launchpad account
- [x] **Source Package**: Need source package (not just binary .deb)
- [x] **Ubuntu-Specific**: Package compatible with Ubuntu repositories
- [x] **Legal Compliance**: MIT license compatible with Ubuntu

## 🚀 PPA Submission Process

### 1. Set Up Launchpad Account

```bash
# Create account at https://launchpad.net
# Add GPG key to your Launchpad profile
# Import Ubuntu Code of Conduct

# Generate GPG key if needed
gpg --gen-key
gpg --send-keys YOUR-GPG-KEY-ID
```

### 2. Create Source Package

```bash
# Install Ubuntu packaging tools
sudo apt install devscripts build-essential dh-make

# Create source package structure
cd /path/to/pmat-source
dh_make -e hello@paiml.com -f ../pmat_2.10.0.orig.tar.gz

# Build source package
debuild -S -sa
```

### 3. Create PPA

```bash
# Create PPA on Launchpad
# Go to: https://launchpad.net/~your-username
# Click "Create a new PPA"
# Name: pmat
# Description: PMAT - Pragmatic AI MCP Agent Toolkit
```

### 4. Upload to PPA

```bash
# Upload source package to PPA
dput ppa:your-username/pmat pmat_2.10.0-1_source.changes

# Check upload status
# Visit: https://launchpad.net/~your-username/+archive/ubuntu/pmat
```

## 📋 Package Requirements for PPA

### Source Package Structure
```
pmat-2.10.0/
├── debian/
│   ├── control           # Package metadata
│   ├── rules             # Build rules
│   ├── copyright         # License information
│   ├── changelog         # Version history
│   ├── postinst          # Post-installation script
│   ├── prerm             # Pre-removal script
│   ├── postrm            # Post-removal script
│   └── source/format     # Source format specification
├── Cargo.toml            # Rust project file
├── src/                  # Source code
└── ...                   # Other project files
```

### Additional Required Files

1. **debian/rules** (make-like build script):
```makefile
#!/usr/bin/make -f

%:
	dh $@

override_dh_auto_build:
	cargo build --release

override_dh_auto_install:
	cargo install --path . --root debian/pmat/usr --force
```

2. **debian/source/format**:
```
3.0 (quilt)
```

3. **debian/compat**:
```
13
```

## 🔍 Ubuntu-Specific Considerations

### Package Naming
- Must follow Ubuntu naming conventions
- No conflicts with existing packages
- Version format: `2.10.0-1ubuntu1`

### Dependencies
- Only Ubuntu-available dependencies
- Conservative version requirements
- Handle missing optional dependencies gracefully

### Architecture Support
- amd64 (required)
- arm64 (recommended)
- armhf (optional)

### Ubuntu Versions
Target supported Ubuntu releases:
- Ubuntu 24.04 LTS (Noble Numbat)
- Ubuntu 22.04 LTS (Jammy Jellyfish)  
- Ubuntu 20.04 LTS (Focal Fossa)

## ⚡ Alternative: Snap Package

Consider creating a Snap package for broader Ubuntu support:

```bash
# Install snapcraft
sudo apt install snapcraft

# Create snapcraft.yaml
cat > snap/snapcraft.yaml << 'EOF'
name: pmat
version: '2.10.0'
summary: Pragmatic AI MCP Agent Toolkit
description: |
  Zero-config AI context generation and code quality toolkit
  with Claude Code Agent Mode integration.

base: core22
confinement: strict
grade: stable

parts:
  pmat:
    plugin: rust
    source: .
    build-packages:
      - pkg-config
      - libssl-dev

apps:
  pmat:
    command: bin/pmat
    plugs:
      - home
      - network
      - removable-media
EOF

# Build snap
snapcraft

# Test snap
sudo snap install pmat_2.10.0_amd64.snap --devmode

# Publish to Snap Store
snapcraft upload pmat_2.10.0_amd64.snap
```

## 🎯 Benefits of PPA Distribution

### For Users
- **Easy Installation**: `sudo add-apt-repository ppa:username/pmat && sudo apt install pmat`
- **Automatic Updates**: Updates through normal apt upgrade process
- **Dependency Management**: APT handles all dependencies automatically
- **Ubuntu Integration**: Native package manager integration

### For Project
- **Wide Reach**: Access to millions of Ubuntu users
- **Credibility**: Official Ubuntu ecosystem presence
- **Maintenance**: Centralized update distribution
- **Feedback**: User feedback through Ubuntu channels

## 📊 Expected Timeline

- **Account Setup**: 1-2 days (GPG verification)
- **Package Preparation**: 2-3 days (source package creation)
- **Initial Upload**: Few hours (automated processing)
- **Build Process**: 30-60 minutes per architecture
- **Publication**: Immediate once builds complete
- **User Access**: Immediate after publication

## 🛠️ Maintenance Workflow

### Regular Updates
```bash
# Update version in debian/changelog
dch -v 2.11.0-1 "New upstream release"

# Build and upload
debuild -S -sa
dput ppa:username/pmat pmat_2.11.0-1_source.changes
```

### Emergency Fixes
```bash
# Create patch version  
dch -v 2.10.0-2 "Fix critical security issue"
debuild -S -sa
dput ppa:username/pmat pmat_2.10.0-2_source.changes
```

## 🔧 Testing and Validation

### Local Testing
```bash
# Test package builds
pbuilder-dist focal build pmat_2.10.0-1.dsc
pbuilder-dist jammy build pmat_2.10.0-1.dsc
pbuilder-dist noble build pmat_2.10.0-1.dsc
```

### PPA Testing
```bash
# Add test PPA
sudo add-apt-repository ppa:username/pmat-testing
sudo apt update
sudo apt install pmat

# Verify functionality
pmat --version
pmat agent mcp-server --help
```

## 📞 Support and Resources

- **Ubuntu Packaging**: https://packaging.ubuntu.com/
- **Launchpad Help**: https://help.launchpad.net/
- **PPA Guidelines**: https://help.launchpad.net/Packaging/PPA
- **Ubuntu Policies**: https://wiki.ubuntu.com/UbuntuDevelopment

---

**Status**: Ready for PPA submission once source package is prepared and Launchpad account is configured with GPG key.