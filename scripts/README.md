# Scripts Directory

This directory contains utility scripts for the MCP Agent Toolkit project.

## Installation

pmat is installed from crates.io — this is the single supported install method:

```bash
# Latest
cargo install pmat

# Specific version
cargo install pmat --version 3.18.0
```

## Testing

The installation scripts have comprehensive test coverage:

### Unit Tests (`install.test.ts`)

- Platform detection logic
- URL construction
- Version handling
- Error cases

### Integration Tests (`install.integration.test.ts`)

- Verifies GitHub releases exist
- Checks asset naming conventions
- Tests download URLs are accessible

Run tests with:

```bash
make test-scripts
```

## Platform Support

Both installers support the following Rust target triples:

- `x86_64-unknown-linux-gnu` (Linux x64)
- `aarch64-unknown-linux-gnu` (Linux ARM64)
- `x86_64-apple-darwin` (macOS Intel)
- `aarch64-apple-darwin` (macOS Apple Silicon)
- `x86_64-pc-windows-msvc` (Windows x64)

## Available Scripts

### mcp-install.ts

Installation and management script for the MCP server that provides smart
rebuild detection and idempotent installation.

**Usage:**

```bash
./scripts/mcp-install.ts              # Smart install (rebuilds only if source changed)
./scripts/mcp-install.ts --status     # Check installation and build status
./scripts/mcp-install.ts --check      # Check if rebuild is needed
./scripts/mcp-install.ts --reinstall  # Force complete reinstall
./scripts/mcp-install.ts --help       # Show help
```

**Note:** The standard `make install` command is now idempotent and will
automatically remove any existing installation before installing the new
version.

### validate-docs.ts

Validates documentation consistency and naming conventions across the project.

**Usage:**

```bash
./scripts/validate-docs.ts  # Run validation
# Or via make:
make validate-docs
```

### validate-github-actions-status.ts

Checks the status of GitHub Actions workflows and provides detailed failure
analysis.

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
