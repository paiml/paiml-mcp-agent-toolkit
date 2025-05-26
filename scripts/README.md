# Scripts Directory

This directory contains utility scripts for the MCP Agent Toolkit project.

## Available Scripts

### mcp-reinstall.ts
A comprehensive reinstall script for the MCP server that provides idempotent uninstall and reinstall functionality.

**Usage:**
```bash
./scripts/mcp-reinstall.ts              # Full uninstall and reinstall
./scripts/mcp-reinstall.ts --uninstall  # Uninstall only  
./scripts/mcp-reinstall.ts --install    # Install only
./scripts/mcp-reinstall.ts --status     # Check installation status
./scripts/mcp-reinstall.ts --help       # Show help
```

**Note:** The standard `make install` command is now idempotent and will automatically remove any existing installation before installing the new version.

### docker-setup.ts
Docker installation and permission management script.

**Usage:**
```bash
./scripts/docker-setup.ts --status   # Check Docker installation status
./scripts/docker-setup.ts --install  # Install Docker if not present
./scripts/docker-setup.ts --repair   # Fix Docker permission issues
```

## Installation Methods

The MCP server can be installed/reinstalled in several ways:

1. **Standard Installation (Recommended)**
   ```bash
   make install  # Idempotent - removes old version first
   ```

2. **Full Reinstall with Script**
   ```bash
   ./scripts/mcp-reinstall.ts
   ```

3. **Manual Uninstall/Install**
   ```bash
   make uninstall
   make install
   ```

All methods ensure a clean installation by removing any existing versions first.