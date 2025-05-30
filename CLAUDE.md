# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Important Context

**IMPORTANT**: Always check `docs/bugs/` directory for active bugs before making changes. Archived bugs are in `docs/bugs/archived/`. Current active bugs may affect your work.

**This is a frequently accessed project** - assume familiarity with the codebase structure, development patterns, and ongoing work. This is the MCP Agent Toolkit project that provides template generation services for project scaffolding.

**MANDATORY DOGFOODING**: This project MUST use its own tools continuously throughout development. Every coding session MUST demonstrate eating our own dogfood by using the toolkit's analysis capabilities to guide decisions.

## Project Overview

MCP Agent Toolkit is a production-grade MCP (Model Context Protocol) server that provides:
1. **Template Generation** - Project scaffolding for Makefile, README.md, and .gitignore files
2. **AST-Based Code Analysis** - Full AST parsing and analysis for Rust, TypeScript/JavaScript, and Python
3. **Code Complexity Metrics** - Cyclomatic complexity, cognitive complexity, nesting depth analysis
4. **Code Churn Tracking** - Git-based code change analysis and hotspot detection
5. **Dependency Graph Generation** - Visual code structure analysis with Mermaid

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

## Deep Dogfooding Integration (MANDATORY)

**EVERY development session MUST extensively use our own tools!** This is not optional - it's how we ensure quality and understand the developer experience.

### Session Start Ritual (REQUIRED)

```bash
# 1. Analyze current project state
make dogfood

# 2. Check complexity metrics before starting
./target/release/paiml-mcp-agent-toolkit analyze complexity --toolchain rust --format full

# 3. Identify code hotspots from last week
./target/release/paiml-mcp-agent-toolkit analyze churn --period 7 --format markdown

# 4. Generate dependency graph to understand structure
./target/release/paiml-mcp-agent-toolkit analyze dag --show-complexity -o current-dag.mmd

# 5. Generate context for AI assistance
./target/release/paiml-mcp-agent-toolkit context rust --format markdown -o context.md
```

### During Development (CONTINUOUS)

#### After Writing New Functions
```bash
# Check complexity of new code immediately
./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --toolchain rust \
  --include "**/*.rs" \
  --max-cyclomatic 10 \
  --max-cognitive 15

# If complexity is too high, refactor immediately
```

#### Before Major Changes
```bash
# Generate "before" snapshot
./target/release/paiml-mcp-agent-toolkit analyze dag \
  --dag-type full-dependency \
  --show-complexity \
  -o before-change.mmd

# Make changes...

# Generate "after" snapshot
./target/release/paiml-mcp-agent-toolkit analyze dag \
  --dag-type full-dependency \
  --show-complexity \
  -o after-change.mmd

# Compare visually to ensure architectural integrity
```

#### Using MCP Mode in Claude Code
```
# Ask Claude to analyze using MCP tools:
"What are the complexity hotspots in this codebase?"
"Show me the code churn analysis for the last month"
"Generate a dependency graph for the services module"
"Which files should I refactor based on churn and complexity?"
```

### Session End Ritual (MANDATORY)

```bash
# 1. Final complexity check
./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --format sarif > complexity-report.sarif

# 2. Update churn metrics
./target/release/paiml-mcp-agent-toolkit analyze churn \
  --period 30 \
  --format json > churn-metrics.json

# 3. Regenerate all documentation
make dogfood

# 4. Verify all artifacts were created
ls -la artifacts/dogfooding/
```

### CI/CD Dogfooding

```yaml
# In GitHub Actions workflows:
- name: Dogfood Analysis
  run: |
    make server-build-binary
    ./target/release/paiml-mcp-agent-toolkit analyze complexity \
      --format sarif > complexity.sarif
    ./target/release/paiml-mcp-agent-toolkit analyze churn \
      --period 30 --format json > churn.json
    
- name: Upload Analysis Results
  uses: github/codeql-action/upload-sarif@v2
  with:
    sarif_file: complexity.sarif
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

4. **`analyze_dag`** - Generate dependency graphs
   ```json
   {
     "project_path": "/path/to/project",
     "dag_type": "call-graph|import-graph|inheritance|full-dependency",
     "filter_external": true,
     "show_complexity": true,
     "format": "mermaid"
   }
   ```

5. **`generate_context`** - Generate project context with AST
   ```json
   {
     "toolchain": "rust|deno|python-uv",
     "project_path": "/path/to/project",
     "format": "markdown|json"
   }
   ```

### Dogfooding Decision Matrix

| Scenario | Tool to Use | Command Example |
|----------|-------------|-----------------|
| Starting new feature | `analyze dag` + `context` | `./target/release/paiml-mcp-agent-toolkit analyze dag --dag-type call-graph` |
| Code review prep | `analyze complexity` + `churn` | `make dogfood` |
| Refactoring decision | `analyze churn` | `./target/release/paiml-mcp-agent-toolkit analyze churn --period 90` |
| Architecture review | `analyze dag --show-complexity` | Full dependency analysis |
| Performance investigation | `analyze complexity --format sarif` | IDE integration |
| Before commits | `analyze complexity` | Check threshold violations |
| Weekly planning | `analyze churn --period 7` | Identify unstable areas |

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

### Real-World Dogfooding Examples

#### Example 1: Refactoring High-Complexity Functions
```bash
# Find complex functions
./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --toolchain rust \
  --max-cyclomatic 10 \
  --format full | grep "ERROR"

# Generate context for specific file
./target/release/paiml-mcp-agent-toolkit context rust \
  --include "**/handlers/tools.rs"

# After refactoring, verify improvement
./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --toolchain rust \
  --include "**/handlers/tools.rs"
```

#### Example 2: Architecture Planning
```bash
# Analyze current architecture
./target/release/paiml-mcp-agent-toolkit analyze dag \
  --dag-type full-dependency \
  --filter-external \
  --show-complexity \
  -o architecture.mmd

# Identify circular dependencies
./target/release/paiml-mcp-agent-toolkit analyze dag \
  --dag-type import-graph | grep -A5 -B5 "circular"

# Check for god objects
./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --format json | jq '.files | map(select(.metrics.functions > 20))'
```

#### Example 3: Code Review Automation
```bash
#!/bin/bash
# pre-commit hook using our tools

# Check complexity thresholds
if ! ./target/release/paiml-mcp-agent-toolkit analyze complexity \
  --max-cyclomatic 15 --max-cognitive 20 --format json | \
  jq -e '.summary.total_errors == 0'; then
  echo "❌ Complexity threshold exceeded. Please refactor."
  exit 1
fi

# Check for high-churn files being modified
CHANGED_FILES=$(git diff --cached --name-only)
HIGH_CHURN=$(./target/release/paiml-mcp-agent-toolkit analyze churn \
  --period 30 --format json | jq -r '.hotspots[].file')

for file in $CHANGED_FILES; do
  if echo "$HIGH_CHURN" | grep -q "$file"; then
    echo "⚠️  Warning: $file is a high-churn file. Extra review recommended."
  fi
done
```

### Performance Monitoring with Dogfooding

```bash
# Benchmark before optimization
hyperfine --warmup 3 \
  './target/release/paiml-mcp-agent-toolkit analyze complexity --toolchain rust'

# Profile with perf
perf record --call-graph=dwarf \
  ./target/release/paiml-mcp-agent-toolkit analyze dag --dag-type full-dependency

# Check binary size impact
ls -lh target/release/paiml-mcp-agent-toolkit
```

### Integration Testing with Our Own Tools

```bash
# Test template generation
./target/release/paiml-mcp-agent-toolkit scaffold rust \
  --templates makefile,readme,gitignore \
  -p project_name=test-project

# Analyze the generated project
cd test-project
../target/release/paiml-mcp-agent-toolkit analyze complexity --toolchain rust
../target/release/paiml-mcp-agent-toolkit context rust
```

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

// Generate dependency graph
const dag = await mcp.call("analyze_dag", {
  project_path: process.cwd(),
  dag_type: "full-dependency",
  show_complexity: true,
  format: "mermaid"
});
```

## Why Dogfooding Matters

1. **Quality Assurance**: We find bugs before users do
2. **UX Understanding**: We experience the same friction as users
3. **Feature Validation**: We know if features actually solve problems
4. **Performance Reality**: We feel the real-world performance
5. **Documentation Accuracy**: Our docs reflect actual usage patterns

## Dogfooding Metrics to Track

```bash
# Weekly metrics collection
./scripts/collect-dogfood-metrics.ts

# Metrics tracked:
# - How often each tool is used internally
# - Which features are most valuable
# - Performance in real-world scenarios
# - Pain points and friction
# - Feature requests from our own usage
```

Remember: **If we don't use our own tools constantly, we can't expect others to find them valuable.**