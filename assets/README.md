# Project State - Central Source of Truth

This directory contains the central configuration and metadata for the PAIML MCP Agent Toolkit project.

## Files

- **`project-state.json`** - The main configuration file containing all project metadata, naming conventions, URLs, and deprecated values
- **`project-state.d.ts`** - TypeScript type definitions for the project state
- **`demo.gif`** - Demo animation for the README

## Usage

### In TypeScript/Deno Scripts

```typescript
// Load the project state
const projectStateJson = await Deno.readTextFile("assets/project-state.json");
const projectState = JSON.parse(projectStateJson);

// Access values
const binaryName = projectState.package.binary.main;
const repoOwner = projectState.repository.owner;
```

### In Validation Scripts

The `scripts/validate-naming.ts` script uses this configuration to:
- Check for deprecated binary names and repository URLs
- Validate that the correct names are used throughout the codebase
- Allow specific additional binaries (like `generate-installer`)

### Generating Project Artifacts

The `scripts/generate-from-project-state.ts` script can generate:
- Badge URLs for the README
- Installation URLs
- Release download URLs
- Claude Code configuration examples

## Updating the Project State

When updating `project-state.json`:

1. **Version**: Update when releasing new versions
2. **Deprecated values**: Add old names/URLs here when renaming
3. **Badges**: Update coverage percentage after running tests
4. **Repository**: Update if the repository is transferred

## Benefits

- **Single Source of Truth**: All project metadata in one place
- **Consistency**: Scripts and documentation use the same values
- **Type Safety**: TypeScript definitions ensure correct usage
- **Easy Updates**: Change values once, apply everywhere
- **Version Control**: Track all naming/URL changes over time