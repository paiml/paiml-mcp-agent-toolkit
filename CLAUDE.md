# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Context

**This is a frequently accessed project** - assume familiarity with the codebase structure, development patterns, and ongoing work. This is the MCP Agent Toolkit project that provides template generation services for project scaffolding.

## Project Overview

MCP Agent Toolkit is a production-grade MCP (Model Context Protocol) template server implementing project scaffolding for three core file types: Makefile, README.md, and .gitignore. The system is built in Rust with embedded templates for instant, stateless template generation - no external dependencies required.

## Architecture

**Server Component**: Stateless Rust binary with embedded templates
- Standalone Rust binary with all templates compiled in
- Zero runtime dependencies - no database or cloud storage needed
- JSON-RPC 2.0 compliant MCP protocol implementation

**Client Component**: Claude Code integration via STDIO MCP transport
- Project analysis engine with parallel file system scanning
- Toolchain detection for Rust CLI, Deno/TypeScript, and Python UV
- Optimized MCP transport with connection pooling

## Development Guidelines

### Scripting Language Choice

**Use Deno/TypeScript for all scripting** instead of Bash:
- Deno provides strong typing and compile-time checks
- Better error handling and debugging capabilities
- Cross-platform compatibility without shell-specific issues
- Consistent tooling with potential TypeScript client code

Example: Test scripts, build automation, and utility scripts should be written in TypeScript and executed with Deno.

## Server Architecture

The server is designed as a stateless MCP server:

**Standalone Binary** - Single executable with embedded templates
- No runtime dependencies
- Fast startup and execution
- Easy distribution
- All templates compiled into the binary

## Workspace Structure

⚠️ **CRITICAL**: This is a Cargo workspace project with the root Makefile as the primary control point.

**Always use the root Makefile for:**
- All CI/CD operations
- Cross-project commands
- Development workflows (format, lint, test, build)
- Installation and deployment

**Workspace layout:**
```
paiml-mcp-agent-toolkit/          # Root workspace
├── Makefile                      # PRIMARY Makefile - use this!
├── Cargo.toml                    # Workspace definition
├── server/                       # Server project (workspace member)
│   ├── Makefile                 # Project-specific targets only
│   └── Cargo.toml               # Server package
└── installer-macro/              # Macro crate (workspace member)
    └── Cargo.toml               # Macro package
```

## Common Commands

### ⚠️ IMPORTANT: Use root-level commands for 80% of operations!

```bash
# From the ROOT directory (preferred):
make server-build           # Build server binary
make server-test            # Run server tests
make server-lint            # Lint server code
make server-run-mcp         # Run MCP server
make validate               # Run all validation checks
make install                # Install the binary

# DO NOT use these patterns in CI/CD:
# ❌ cd server && make test
# ❌ cd server && cargo build

# ✅ Instead use:
# make server-test
# make server-build-binary
```

### When to use project-specific Makefiles

Only use `cd server && make ...` when:
- You're actively developing within that directory
- You need project-specific targets not exposed at root
- You're debugging specific to that project

### CI/CD Guidelines

All GitHub Actions workflows MUST:
1. Run commands from the repository root
2. Use `make server-*` targets instead of `cd server && make`
3. Use `--manifest-path` for direct cargo commands when needed

Example:
```yaml
# ✅ CORRECT:
- name: Run tests
  run: make server-test

# ❌ WRONG:
- name: Run tests
  run: |
    cd server
    make test
```

## Template URI Schema

The system uses URIs following this pattern:
```
template://makefile/{toolchain}/cli
template://readme/{toolchain}/cli
template://gitignore/{toolchain}/cli
```

Example URIs:
- `template://makefile/rust/cli`
- `template://makefile/deno/cli`
- `template://makefile/python-uv/cli`

## Supported Toolchains

1. **Rust CLI** (cargo + clippy + rustfmt)
   - Variant: cli
   - Target architectures: x86_64-unknown-linux-gnu

2. **Deno/TypeScript CLI** (deno native tooling)
   - Variant: cli
   - Permissions model integrated

3. **Python UV CLI** (uv + ruff + mypy)
   - Variant: cli
   - Python 3.12+ optimized

## Performance Targets

- Startup time: <10ms (no cold starts)
- Template generation: <5ms (in-memory)
- Client analysis: <500ms for full project scan
- Memory usage: <20MB resident

## Development Priorities

1. ~~Embed templates directly in binary (stateless design)~~
2. ~~Build template rendering engine with Handlebars~~
3. Create client-side project analysis engine
4. ~~Implement MCP STDIO transport layer~~
5. Deploy MVP with three template types per toolchain

## Git Commit Policy

**NEVER commit changes unless explicitly asked by the user.** The user will commit when they are ready. This ensures:
- User maintains control over git history
- Changes can be reviewed before committing
- Commit messages can be customized
- Work can be staged incrementally

## Using MCP Agent Toolkit for Project Scaffolding

When users ask about generating project files (Makefile, README, .gitignore), you should:

1. **Detect Project Type**: Look for language-specific files (Cargo.toml, package.json, etc.)
2. **Use MCP Server**: The MCP Agent Toolkit server provides templates for:
   - Makefiles with language-specific build commands
   - README files with project structure
   - .gitignore files with appropriate patterns

3. **Common User Requests**:
   - "Generate a Makefile for my Rust project"
   - "Create a .gitignore for Rust development"
   - "Set up build automation"

4. **Template Parameters**: Each template accepts specific parameters:
   - Always provide `project_name`
   - Ask for clarification on optional parameters if needed
   - Use sensible defaults when appropriate

5. **Example Workflow**:
   ```typescript
   // When user asks for a Makefile
   await mcp.call("generate_template", {
     resource_uri: "template://makefile/rust/cli",
     parameters: {
       project_name: "detected_from_cargo_toml",
       has_tests: true,
       has_benchmarks: false
     }
   });
   ```