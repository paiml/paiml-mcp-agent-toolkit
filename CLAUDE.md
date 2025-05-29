check# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Context

**IMPORTANT**: Always check `docs/bugs/` directory for active bugs before making changes. Archived bugs are in `docs/bugs/archived/`. Current active bugs may affect your work.

**This is a frequently accessed project** - assume familiarity with the codebase structure, development patterns, and ongoing work. This is the MCP Agent Toolkit project that provides template generation services for project scaffolding.

## Project Overview

MCP Agent Toolkit is a production-grade MCP (Model Context Protocol) server that provides:
1. **Template Generation** - Project scaffolding for Makefile, README.md, and .gitignore files
2. **AST-Based Code Analysis** - Full AST parsing and analysis for Rust, TypeScript/JavaScript, and Python
3. **Code Complexity Metrics** - Cyclomatic complexity, cognitive complexity, nesting depth analysis
4. **Code Churn Tracking** - Git-based code change analysis and hotspot detection

The system is built in Rust as a stateless binary with all capabilities compiled in - no external dependencies required.

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
1. **Run commands from the repository root** - Never use `cd` patterns
2. **Use `make server-*` targets** instead of `cd server && make`
3. **Use specific Ubuntu versions** - Never use `ubuntu-latest`
4. **Use `--manifest-path`** for direct cargo commands when needed

#### Required Ubuntu Versions

**NEVER use `ubuntu-latest` or `ubuntu-20.04`** - always pin to specific versions for reproducibility:

```yaml
# ✅ CORRECT - Use specific versions:
jobs:
  release:
    runs-on: ubuntu-22.04  # For all workflows - standard version
  
  ci:
    runs-on: ubuntu-22.04  # For general CI/development workflows
    
  # For future-proofing, consider ubuntu-24.04 for new workflows
  future_workflow:
    runs-on: ubuntu-24.04  # When available and tested

# ❌ WRONG - Never use these:
jobs:
  bad_example_1:
    runs-on: ubuntu-latest  # This can break builds unexpectedly
  
  bad_example_2:
    runs-on: ubuntu-20.04  # RETIRED on 2025-04-15, will cause CI failures
```

**Version Guidelines:**
- **`ubuntu-22.04`**: Use for ALL workflows (releases, CI, testing, development)
- **`ubuntu-24.04`**: Consider for new workflows when stability is confirmed
- **NEVER `ubuntu-20.04`**: Retired on 2025-04-15, will cause workflow failures
- **NEVER `ubuntu-latest`**: Can change unexpectedly and break reproducible builds
- **Rationale**: Pinned versions ensure reproducible builds and prevent surprise breakage from OS updates or platform retirement

#### Command Patterns

```yaml
# ✅ CORRECT:
- name: Run tests
  run: make server-test

- name: Build binary
  run: make server-build-binary

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

## Dogfooding During Development

**MANDATORY: Every development session MUST include dogfooding!** This means:

1. **At Session Start**: Run `make dogfood` to analyze the current state
2. **During Development**: Use our own tools for analysis and generation
3. **Before Session End**: Run `make dogfood` again to update documentation with changes

**ALWAYS use this project's own tools while developing it!** This ensures we catch issues early and understand the developer experience.

### Continuous Code Quality Monitoring

1. **Check complexity before commits**:
   ```bash
   make server-build-binary
   ./target/release/paiml-mcp analyze complexity --toolchain rust --max-cyclomatic 15
   ```

2. **Monitor code churn weekly**:
   ```bash
   ./target/release/paiml-mcp analyze churn --period 7 --format markdown
   ```

3. **Use in CI/CD**:
   ```yaml
   - name: Check Code Complexity
     run: |
       make server-build-binary
       ./target/release/paiml-mcp analyze complexity --format sarif > complexity.sarif
   ```

### Available MCP Tools

The server exposes these tools via MCP protocol:

1. **`generate_template`** - Generate project files
   ```json
   {
     "resource_uri": "template://makefile/rust/cli",
     "parameters": {
       "project_name": "my-project",
       "has_tests": true
     }
   }
   ```

2. **`analyze_complexity`** - Analyze code complexity
   ```json
   {
     "project_path": "/path/to/project",
     "toolchain": "rust|deno|python-uv",
     "format": "summary|full|json|sarif",
     "max_cyclomatic": 20,
     "max_cognitive": 30
   }
   ```

3. **`analyze_code_churn`** - Analyze git history
   ```json
   {
     "project_path": "/path/to/project",
     "period_days": 30,
     "format": "summary|json|markdown|csv"
   }
   ```

### AST Analysis Capabilities

The project provides deep AST analysis for:

**Rust**:
- Functions (including async detection)
- Structs/Enums with field/variant counts
- Traits and implementations
- Module structure
- Visibility modifiers

**TypeScript/JavaScript**:
- Functions/Methods
- Classes with member counts
- Interfaces
- Import/Export analysis
- Async/Generator detection

**Python**:
- Functions/Methods (including async)
- Classes with attribute counts
- Decorators
- Import analysis
- Type annotations

### Complexity Metrics Explained

1. **Cyclomatic Complexity**: Number of independent paths through code
   - Threshold: 10 (warning), 20 (error)
   - Measures: if/else, loops, match/switch statements

2. **Cognitive Complexity**: How hard code is to understand
   - Threshold: 15 (warning), 30 (error)
   - Measures: nesting, breaks in linear flow, recursion

3. **Nesting Depth**: Maximum level of nested blocks
   - Threshold: 4 (warning), 6 (error)

### Example: Analyzing This Project

```bash
# Full analysis of the server codebase
./target/release/paiml-mcp analyze complexity \
  --toolchain rust \
  --format full \
  --include "server/src/**/*.rs"

# Check for complex functions
./target/release/paiml-mcp analyze complexity \
  --toolchain rust \
  --max-cyclomatic 10 \
  --max-cognitive 15

# Track hotspots over the last month
./target/release/paiml-mcp analyze churn \
  --period 30 \
  --format markdown > HOTSPOTS.md
```

### Dogfooding Workflow (REQUIRED)

**Every development session MUST follow this workflow:**

1. **Session Start Dogfooding**:
   ```bash
   # Analyze current state and update documentation
   make dogfood
   ```

2. **During Development**:
   ```bash
   # After significant changes, analyze complexity
   make server-build-binary
   ./target/release/paiml-mcp-agent-toolkit analyze complexity --toolchain rust
   
   # Check for code churn hotspots
   ./target/release/paiml-mcp-agent-toolkit analyze churn --period-days 7
   
   # Generate dependency graphs
   ./target/release/paiml-mcp-agent-toolkit analyze dag --show-complexity
   ```

3. **Session End Dogfooding** (MANDATORY):
   ```bash
   # Update README with fresh metrics
   make dogfood
   
   # Verify artifacts were created
   ls -la artifacts/dogfooding/
   ```

**Why Dogfooding Matters:**
- Ensures our tools work correctly on real projects
- Keeps documentation up-to-date with actual metrics
- Identifies issues before users encounter them
- Demonstrates real-world usage patterns

## Using MCP Agent Toolkit Features

### Template Generation

When users ask about generating project files:

1. **Detect Project Type**: Look for language-specific files (Cargo.toml, package.json, etc.)
2. **Generate Templates**: Use the appropriate template URI
3. **Common Requests**:
   - "Generate a Makefile for my Rust project"
   - "Create a .gitignore for Rust development"
   - "Set up build automation"

### Code Analysis

When users ask about code quality or complexity:

1. **Run Complexity Analysis**: Use `analyze_complexity` with appropriate thresholds
2. **Check Code Churn**: Use `analyze_code_churn` to find frequently changed files
3. **Common Requests**:
   - "What are the most complex functions in my codebase?"
   - "Show me code hotspots from the last month"
   - "Check if any functions exceed complexity thresholds"

### Integration Examples

```typescript
// Generate a Makefile
await mcp.call("generate_template", {
  resource_uri: "template://makefile/rust/cli",
  parameters: {
    project_name: "my-project",
    has_tests: true,
    has_benchmarks: false
  }
});

// Analyze complexity
const complexity = await mcp.call("analyze_complexity", {
  project_path: process.cwd(),
  toolchain: "rust",
  format: "json",
  max_cyclomatic: 15
});

// Check code churn
const churn = await mcp.call("analyze_code_churn", {
  project_path: process.cwd(),
  period_days: 30,
  format: "json"
});
```