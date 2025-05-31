# Archive Directory

This directory contains deprecated and historical scripts that have been
archived for reference.

## Deprecated Scripts

### dogfood-readme-deprecated.ts

- **Status**: Deprecated as of 2025-05-30
- **Replaced by**: The binary's built-in dogfooding via `make dogfood`
- **Reason**: The Deno script approach has been superseded by using our own MCP
  Agent Toolkit binary directly
- **Migration**: Use `make dogfood` instead, which uses the binary's analysis
  capabilities and properly manages README section replacement

The binary approach is preferred because:

- Uses our own tools for genuine dogfooding
- Zero external dependencies (no Deno required)
- Better section replacement logic with HTML comment markers
- Consistent with our "eating our own dogfood" philosophy

## Archived Historical Scripts

### generate-from-project-state.ts

- **Status**: Archived as of 2025-05-30
- **Purpose**: Generate project artifacts from central project state JSON
- **Reason**: Not actively used but could be useful for future badge/metadata
  generation

### cleanup-releases.ts

- **Status**: Archived as of 2025-05-30
- **Purpose**: Diagnose and clean up GitHub release issues
- **Reason**: Useful maintenance utility for debugging release problems when
  they occur

### verify-demo-binary-size.ts

- **Status**: Archived as of 2025-05-30
- **Purpose**: Verify binary size with demo mode
- **Reason**: Could be useful for performance monitoring but not actively used

### cleanup-test-artifacts.ts

- **Status**: Archived as of 2025-05-30
- **Purpose**: Clean up test artifacts from lib directory
- **Reason**: Utility script not actively used in current workflow

## Dead Scripts (Removed)

### dead-scripts/ directory

Contains scripts that were completely obsolete and had no current value:

#### download-mermaid.ts

- **Purpose**: Downloads Mermaid.js v10.6.1 for demo mode
- **Reason for removal**: Demo mode uses embedded assets now, script is outdated

#### mcp-install-deterministic.ts

- **Purpose**: Alternative "deterministic" MCP install script
- **Reason for removal**: Duplicate of `mcp-install.ts` with no active usage

#### docker-setup.ts

- **Purpose**: Docker installation and setup utility
- **Reason for removal**: Duplicate exists in `server/scripts/docker-setup.ts`
