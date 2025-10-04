# MCP Registry Publishing Specification

## Overview

This document describes the automated publishing mechanism for the PMAT Agent Toolkit to the Model Context Protocol (MCP) registry. The server is published as `io.github.paiml/pmat-agent` and is available at https://registry.modelcontextprotocol.io.

## Published Server Information

- **Name**: `io.github.paiml/pmat-agent`
- **Display Name**: PMAT Agent Toolkit
- **Registry URL**: https://registry.modelcontextprotocol.io/v0/servers?search=pmat
- **Status**: Active
- **Deployment Type**: Cargo (Rust crate)
- **Package Name**: `pmat`
- **Binary Name**: `pmat-agent`

## Server Configuration

The MCP server is configured via `server.json` in the repository root:

```json
{
  "$schema": "https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json",
  "name": "io.github.paiml/pmat-agent",
  "displayName": "PMAT Agent Toolkit",
  "description": "Zero-config AI context generation and code quality toolkit with Claude Code Agent Mode support",
  "version": "2.121.0",
  "homepage": "https://github.com/paiml/paiml-mcp-agent-toolkit",
  "sourceUrl": "https://github.com/paiml/paiml-mcp-agent-toolkit",
  "author": {
    "name": "Pragmatic AI Labs",
    "email": "hello@paiml.com",
    "url": "https://github.com/paiml"
  },
  "license": "MIT",
  "readme": "https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/README.md",
  "categories": ["development-tools", "code-analysis"],
  "tags": ["code-quality", "static-analysis", "complexity", "technical-debt", "ai-context", "claude-code", "agent"],
  "deployment": {
    "type": "package",
    "package": {
      "type": "cargo",
      "name": "pmat",
      "binaryName": "pmat-agent"
    }
  }
}
```

## Automated Publishing Workflow

### Trigger

Publishing is automatically triggered on GitHub release creation:

```bash
gh release create v2.122.0 --generate-notes
```

### Workflow Steps

The MCP publishing workflow (`.github/workflows/multi-ecosystem-release.yml`) includes a dedicated `publish-mcp` job:

1. **Prepare**: Extract version and metadata
2. **Wait for crates.io**: Allow 60 seconds for the crate to be indexed on crates.io
3. **Install MCP Publisher**: Download and extract the `mcp-publisher` CLI tool (v1.2.3)
4. **Authenticate**: Use GitHub OIDC to authenticate with the MCP registry
5. **Publish**: Execute `mcp-publisher publish` to publish the server

### Authentication

The workflow uses GitHub OIDC (OpenID Connect) for authentication:

```yaml
permissions:
  id-token: write  # Required for OIDC authentication
  contents: read
```

This eliminates the need for additional secrets or tokens - authentication is handled automatically via GitHub's identity.

### Workflow Configuration

```yaml
publish-mcp:
  needs: prepare
  runs-on: ubuntu-latest
  permissions:
    id-token: write
    contents: read
  steps:
  - name: Checkout
    uses: actions/checkout@v4

  - name: Wait for crates.io availability
    if: ${{ !github.event.inputs.dry_run }}
    run: |
      echo "Waiting 60 seconds for crates.io to index the package..."
      sleep 60

  - name: Install MCP Publisher
    if: ${{ !github.event.inputs.dry_run }}
    run: |
      curl -L "https://github.com/modelcontextprotocol/registry/releases/download/v1.2.3/mcp-publisher_1.2.3_$(uname -s | tr '[:upper:]' '[:lower:]')_$(uname -m | sed 's/x86_64/amd64/;s/aarch64/arm64/').tar.gz" | tar xz

  - name: Login to MCP Registry
    if: ${{ !github.event.inputs.dry_run }}
    run: ./mcp-publisher login github-oidc

  - name: Publish to MCP Registry
    if: ${{ !github.event.inputs.dry_run }}
    run: ./mcp-publisher publish
```

## Prerequisites

1. **Crates.io Publication**: The Rust crate must be published to crates.io first (handled by `publish-crates.yml` workflow)
2. **CARGO_REGISTRY_TOKEN**: Secret must be set for crates.io publishing
3. **GitHub Repository**: Must be under the `paiml` organization to match the `io.github.paiml` namespace
4. **server.json**: Must be present in repository root with valid configuration

## Deployment Type: Cargo

The MCP server uses cargo (Rust crate) deployment, which means:

- The MCP registry validates the package exists on crates.io
- Users install via: `cargo install pmat`
- The binary `pmat-agent` is executed for MCP server functionality
- No npm or other package manager is required for MCP functionality

## Manual Publishing

If needed, you can manually publish using the MCP Publisher CLI:

```bash
# Install MCP Publisher
curl -L "https://github.com/modelcontextprotocol/registry/releases/download/v1.2.3/mcp-publisher_1.2.3_linux_amd64.tar.gz" | tar xz

# Login (opens browser for GitHub OAuth)
./mcp-publisher login github

# Publish
./mcp-publisher publish
```

## Verification

After publishing, verify the server is in the registry:

```bash
curl -s "https://registry.modelcontextprotocol.io/v0/servers?search=pmat" | jq '.'
```

Expected response:
```json
{
  "servers": [
    {
      "server": {
        "name": "io.github.paiml/pmat-agent",
        "version": "2.121.0",
        "status": "active"
      }
    }
  ]
}
```

## Troubleshooting

### Common Issues

1. **"invalid repository URL" error**
   - Solution: Use empty repository URLs in server.json (`"url": "", "source": ""`)

2. **Package not found on crates.io**
   - Solution: Ensure crates.io publish completed and allow time for indexing (60s wait included)

3. **Authentication failure**
   - Solution: Verify `id-token: write` permission is set in workflow

4. **Schema validation errors**
   - Solution: Validate server.json against schema:
     ```bash
     curl -sS https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json -o /tmp/server-schema.json
     npx ajv-cli validate --strict=false -s /tmp/server-schema.json -d server.json
     ```

## Version Management

- The version in `server.json` should match the crate version
- Update `server.json` version when bumping crate version
- The workflow uses the version from git tags for releases

## Resources

- **MCP Registry**: https://registry.modelcontextprotocol.io
- **Publishing Guide**: https://github.com/modelcontextprotocol/registry/blob/main/docs/guides/publishing/publish-server.md
- **Schema**: https://static.modelcontextprotocol.io/schemas/2025-09-29/server.schema.json
- **Publisher CLI**: https://github.com/modelcontextprotocol/registry/releases

## Integration with Multi-Ecosystem Release

The MCP publishing is integrated into the multi-ecosystem release workflow, which also handles:

- npm package publishing
- Homebrew formula updates
- AUR package updates
- Chocolatey package updates
- Debian package updates

All ecosystem updates are triggered by a single GitHub release, ensuring consistency across all distribution channels.
