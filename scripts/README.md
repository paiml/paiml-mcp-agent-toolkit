# Scripts Directory

This directory contains utility scripts for the MCP Agent Toolkit project.

## Available Scripts

### mcp-install.ts / mcp-reinstall.ts / mcp-install-deterministic.ts
Installation and management scripts for the MCP server that provide smart rebuild detection and idempotent installation.

**Usage:**
```bash
./scripts/mcp-install.ts              # Smart install (rebuilds only if source changed)
./scripts/mcp-install.ts --status     # Check installation and build status
./scripts/mcp-install.ts --check      # Check if rebuild is needed
./scripts/mcp-install.ts --reinstall  # Force complete reinstall
./scripts/mcp-install.ts --help       # Show help
```

**Note:** The standard `make install` command is now idempotent and will automatically remove any existing installation before installing the new version.

### validate-docs.ts
Validates documentation consistency and naming conventions across the project.

**Usage:**
```bash
./scripts/validate-docs.ts  # Run validation
# Or via make:
make validate-docs
```

### validate-github-actions-status.ts
Checks the status of GitHub Actions workflows and provides detailed failure analysis.

**Usage:**
```bash
./scripts/validate-github-actions-status.ts  # Check CI status
# Or via make:
make ci-status

# With GitHub token for higher rate limits:
GITHUB_TOKEN=your_token make ci-status
```

**Features:**
- Shows latest workflow run status for each workflow
- Provides detailed failure analysis for failed jobs
- Lists specific steps that failed
- Offers common fixes for typical failures
- Exits with non-zero code if any workflows are failing

### docker-setup.ts
Docker installation and permission management script (Docker is optional for this project).

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