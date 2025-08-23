# PMAT Chocolatey Package for Windows

This directory contains the Chocolatey package for installing PMAT on Windows systems.

## Installation

### Via Chocolatey (Once Published)
```powershell
# Install PMAT
choco install pmat

# Upgrade PMAT
choco upgrade pmat

# Uninstall PMAT
choco uninstall pmat
```

### Current Status
The package is ready for submission to the Chocolatey Community Repository. See `SUBMIT_TO_CHOCOLATEY.md` for submission details.

## Installation Strategy

The Chocolatey package provides multiple installation paths:

### 1. Automatic Installation (Preferred)
If Rust/Cargo is available, the package automatically installs from crates.io:
```powershell
cargo install pmat --version 2.10.0 --force
```

### 2. Alternative Methods
If Cargo is not available, the package provides guided installation via:
- **Node.js/npm**: `npm install -g pmat-agent`
- **Installation helpers**: Automated scripts in Program Files
- **Manual download**: Links to GitHub releases

### 3. Helper Scripts
The package installs helper scripts in `%ProgramFiles%\PMAT\`:
- `install-via-cargo.bat`: Automated Rust installation
- `install-via-npm.bat`: Automated npm installation

## Claude Code Integration

After binary installation, configure Claude Code:

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

## Package Contents

- **pmat.nuspec**: Package metadata and dependencies
- **tools/chocolateyinstall.ps1**: Installation script with multiple fallbacks
- **tools/chocolateyuninstall.ps1**: Clean removal script
- **legal/**: LICENSE.txt and VERIFICATION.txt
- **build-package.ps1**: Automated build and submission script
- **test-package.ps1**: Comprehensive testing script

## Testing

### Local Testing
```powershell
# Build and test package
.\build-package.ps1 -Test

# Test installation (requires admin)
.\test-package.ps1

# Build without testing
.\build-package.ps1
```

### Automated Testing
The package includes comprehensive tests:
- Package creation validation
- Installation process testing  
- Binary availability checking
- Clean uninstallation verification

## Submission

### Prerequisites
- Chocolatey account at [community.chocolatey.org](https://community.chocolatey.org)
- API key from your account profile
- Local testing completed successfully

### Submit Package
```powershell
# Automated submission
.\build-package.ps1 -Submit -ApiKey "YOUR_API_KEY"

# Manual submission
choco pack pmat.nuspec
choco push pmat.2.10.0.nupkg --source https://push.chocolatey.org/
```

## Features

### Multi-Tier Installation
1. **Cargo** (if available) → Direct crates.io installation
2. **npm** (if available) → Node.js ecosystem installation  
3. **Helper Scripts** → Guided manual installation
4. **Future**: Pre-built Windows binaries

### User Experience
- **Progress Feedback**: Clear installation progress and status
- **Error Handling**: Graceful fallbacks for missing dependencies
- **Helpful Messages**: Guidance for each installation method
- **Clean Removal**: Complete uninstallation with PATH cleanup

## Support

- **Chocolatey Issues**: Package comments after publication
- **PMAT Issues**: [GitHub Issues](https://github.com/paiml/paiml-mcp-agent-toolkit/issues)
- **Chocolatey Guidelines**: [Documentation](https://docs.chocolatey.org/en-us/community-repository/)

---

**Maintainer**: Pragmatic AI Labs  
**License**: MIT  
**Homepage**: https://github.com/paiml/paiml-mcp-agent-toolkit