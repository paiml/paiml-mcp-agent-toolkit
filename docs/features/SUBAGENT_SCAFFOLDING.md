# Claude Code Sub-Agent Scaffolding

**Status**: ✅ IMPLEMENTED (PMAT-7007)
**Version**: v2.144.0
**Category**: Code Quality Tools

## Overview

PMAT now provides comprehensive scaffolding for generating specialized Claude Code sub-agents. These sub-agents are designed to work seamlessly with Claude Code's MCP (Model Context Protocol) integration, providing focused expertise in specific code quality domains.

## What Are Sub-Agents?

Sub-agents are specialized AI assistants that:
- **Focus on specific domains** (complexity analysis, mutation testing, technical debt detection, etc.)
- **Integrate with PMAT MCP tools** via standardized tool mappings
- **Follow Claude Code conventions** for markdown-based definitions
- **Coordinate with each other** through well-defined communication protocols
- **Provide deterministic analysis** powered by PMAT's quality infrastructure

## MVP Sub-Agents (5 Available)

### 1. Complexity Analyst
- **Purpose**: Cyclomatic and cognitive complexity analysis
- **Tools**: `pmat__analyze_complexity`, `pmat__analyze_cognitive_complexity`
- **Use Cases**: Identify high-complexity hotspots, suggest refactorings
- **Invocation**: `@complexity-analyst analyze src/`

### 2. Mutation Tester
- **Purpose**: Mutation testing with ML-powered prediction
- **Tools**: `pmat__mutation_test`, `pmat__mutation_predict`, `pmat__equivalent_detector`
- **Use Cases**: Test quality assessment, equivalent mutant detection
- **Invocation**: `@mutation-tester test src/lib.rs`

### 3. SATD Detector
- **Purpose**: Technical debt tracking (TODO/FIXME/HACK comments)
- **Tools**: `pmat__analyze_satd`, `pmat__analyze_context`
- **Use Cases**: Identify and prioritize technical debt
- **Invocation**: `@satd-detector scan for old debt`

### 4. Dead Code Eliminator
- **Purpose**: Unused code identification with safety scoring
- **Tools**: `pmat__analyze_dead_code`, `pmat__analyze_imports`
- **Use Cases**: Safe code cleanup, bloat reduction
- **Invocation**: `@dead-code-eliminator analyze src/utils --private-only`

### 5. Documentation Enforcer
- **Purpose**: Generic description detection and quality enforcement
- **Tools**: `pmat__check_generic_docs`, `pmat__analyze_context`
- **Use Cases**: Ensure high-quality documentation, eliminate vague descriptions
- **Invocation**: `@documentation-enforcer analyze src/services`

## CLI Commands

### List Available Sub-Agents

```bash
# List MVP sub-agents (default)
pmat scaffold list-subagents

# List all sub-agents (including future phases)
pmat scaffold list-subagents --all
```

**Output**:
```
Available PMAT Sub-Agents:

  ✅ MVP complexity-analyst - Expert in cyclomatic and cognitive complexity analysis
    Tools: analyze_complexity, analyze_cognitive_complexity

  ✅ MVP mutation-tester - Mutation testing specialist with ML prediction
    Tools: mutation_test, mutation_predict, equivalent_detector

  ...

Total: 5 MVP, 0 Future
```

### Create a Specific Sub-Agent

```bash
# Create to current directory/.claude/subagents (default)
pmat scaffold create-subagent complexity-analyst

# Create to custom directory
pmat scaffold create-subagent mutation-tester -o /path/to/output
```

**Output**:
```
Creating sub-agent: complexity-analyst
  Description: Expert in cyclomatic and cognitive complexity analysis
  Output: .claude/subagents

✅ Sub-agent created successfully!
  File: .claude/subagents/complexity-analyst.md

Next steps:
  1. Review the generated sub-agent definition
  2. Customize if needed (prompts, tools, examples)
  3. Use with Claude Code: Place in .claude/subagents/
  4. Invoke: @complexity-analyst <command>
```

### Create All MVP Sub-Agents

```bash
# Create all 5 MVP sub-agents at once
pmat scaffold create-all-subagents

# Create to custom directory
pmat scaffold create-all-subagents -o /path/to/output
```

**Output**:
```
Creating all MVP sub-agents...
  Output: .claude/subagents

✅ Created 5 sub-agents successfully!

  ✅ complexity-analyst - complexity-analyst.md
  ✅ mutation-tester - mutation-tester.md
  ✅ satd-detector - satd-detector.md
  ✅ dead-code-eliminator - dead-code-eliminator.md
  ✅ documentation-enforcer - documentation-enforcer.md

Next steps:
  1. Review generated sub-agents in .claude/subagents
  2. Place them in your project's .claude/subagents/ directory
  3. Use with Claude Code: @<agent-name> <command>

Example usage:
  @complexity-analyst analyze src/
  @mutation-tester test src/lib.rs
  @satd-detector scan for old debt
```

### Validate a Sub-Agent Definition

```bash
pmat scaffold validate-subagent .claude/subagents/complexity-analyst.md
```

**Output (on success)**:
```
Validating sub-agent: .claude/subagents/complexity-analyst.md

✅ Validation passed!
  All required sections present
  Markdown format valid
  No issues found
```

**Output (on failure)**:
```
Validating sub-agent: .claude/subagents/custom-agent.md

❌ Validation failed!

Issues:
  ✗ Missing required section: ## Capabilities
  ✗ Missing required section: ## Tools Used
  ✗ File should start with # title

Warnings:
  ⚠ No MCP tools mentioned (expected for PMAT sub-agents)
  ⚠ Contains placeholder text (TODO/TBD)
```

### Show MCP Tool Mapping

```bash
# Show mapping for all sub-agents
pmat scaffold show-tool-mapping

# Show mapping for specific sub-agent
pmat scaffold show-tool-mapping --agent complexity-analyst
```

**Output**:
```
MCP Tool Mapping: complexity-analyst
  Description: Expert in cyclomatic and cognitive complexity analysis

Primary Tools:
  • pmat__analyze_complexity
  • pmat__analyze_cognitive_complexity
```

### Export Tool Mapping as JSON

```bash
pmat scaffold export-tool-mapping -o tool-mapping.json
```

**Output** (`tool-mapping.json`):
```json
{
  "complexity-analyst": [
    "pmat__analyze_complexity",
    "pmat__analyze_cognitive_complexity"
  ],
  "mutation-tester": [
    "pmat__mutation_test",
    "pmat__mutation_predict",
    "pmat__equivalent_detector"
  ],
  ...
}
```

## Integration with Claude Code

### Step 1: Generate Sub-Agents

```bash
# Generate all MVP sub-agents
cd your-project
pmat scaffold create-all-subagents -o .claude/subagents
```

### Step 2: Configure Claude Code

Ensure your project has `.claude/subagents/` directory:

```bash
ls -la .claude/subagents/
# complexity-analyst.md
# mutation-tester.md
# satd-detector.md
# dead-code-eliminator.md
# documentation-enforcer.md
```

### Step 3: Use Sub-Agents in Claude Code

In Claude Code, invoke sub-agents using `@` syntax:

```
@complexity-analyst analyze src/services/
```

Claude Code will:
1. Load the sub-agent definition from `.claude/subagents/complexity-analyst.md`
2. Invoke PMAT MCP tools (`pmat__analyze_complexity`, etc.)
3. Return analysis in the format specified by the sub-agent

### Example Workflows

#### Complexity Analysis Workflow
```
User: "Is this codebase complex?"
Claude Code: @complexity-analyst analyze src/

ComplexityAnalyst: "Let me analyze the codebase complexity...
[Calls pmat__analyze_complexity via MCP]

Found 12 high-complexity functions (CC >10):
1. src/parser.rs:parse_expression() - CC=18 (CRITICAL)
2. src/validator.rs:validate_schema() - CC=14 (HIGH)
...

Recommendations:
- Extract parse_expression() into smaller functions
- Simplify validate_schema() conditionals"
```

#### Mutation Testing Workflow
```
User: "How good are my tests?"
Claude Code: @mutation-tester assess src/lib.rs

MutationTester: "Running mutation analysis...
[Calls pmat__mutation_test via MCP]

Mutation Score: 82% (41/50 mutants killed)
Test Quality: GOOD

Survived Mutants (9):
1. src/lib.rs:45 - Removed `if` guard (not tested)
2. src/lib.rs:67 - Changed `>` to `>=` (boundary not tested)
...

Recommended Tests:
- Add test for guard condition at line 45
- Add boundary tests for comparison at line 67"
```

#### SATD Detection Workflow
```
User: "What technical debt do we have?"
Claude Code: @satd-detector scan

SATDDetector: "Scanning for technical debt...
[Calls pmat__analyze_satd via MCP]

Found 23 SATD markers:

CRITICAL (3):
- src/auth.rs:89 - TODO: Fix auth bypass vulnerability
- src/db.rs:123 - FIXME: SQL injection risk

HIGH (7):
- src/cache.rs:45 - TODO: Add TTL expiration
- src/api.rs:234 - HACK: Temporary workaround for API rate limit
...

Prioritization:
1. Fix security issues immediately (2 critical)
2. Address 7 high-priority items in Sprint 25
3. Track 13 low-priority for future cleanup"
```

## Sub-Agent Template Structure

Each sub-agent definition follows this standardized markdown format:

### Required Sections

1. **# Title** - Sub-agent name (e.g., "Complexity Analyst")
2. **## Description** - One-sentence purpose
3. **## Capabilities** - Bulleted list of what the agent can do
4. **## Tools Used** - MCP tools with `pmat__` prefix
5. **## Role Definition** - Detailed expertise and constraints
6. **## Communication Protocol** - How to interact with Claude Code and other sub-agents
7. **## Implementation Workflow** - Step-by-step process
8. **## Example Invocations** - Concrete usage examples (automatic, manual, coordination)
9. **## Quality Gates** - Success criteria and thresholds

### Example Template

```markdown
# Complexity Analyst

## Description
Expert in cyclomatic and cognitive complexity analysis, suggests refactorings.

## Capabilities
- Analyze cyclomatic complexity (CC) for functions
- Calculate cognitive complexity for human comprehension
- Identify high-complexity hotspots requiring refactoring
- Suggest specific refactoring strategies

## Tools Used
- `analyze_complexity` (MCP) - Primary complexity analysis
- `analyze_cognitive_complexity` (MCP) - Cognitive load metrics

## Role Definition
You are a complexity analysis expert specializing in identifying
and resolving code complexity issues. You understand:
- Cyclomatic complexity thresholds (CC >10 is high)
- Cognitive complexity patterns (nested conditions, callbacks)
- Refactoring strategies (extract method, simplify conditionals)

**Constraints**:
- Never suggest refactorings without measuring impact
- Always provide before/after complexity scores
- Respect PMAT quality gates (CC ≤8)

...
```

## Customization

### Adding Custom Sub-Agents

1. Create a new markdown file following the template structure:

```bash
cp .claude/subagents/complexity-analyst.md .claude/subagents/my-custom-agent.md
```

2. Edit the file to define your agent:
   - Change title and description
   - Define capabilities
   - Map to PMAT MCP tools (or custom tools)
   - Specify role and workflow

3. Validate the definition:

```bash
pmat scaffold validate-subagent .claude/subagents/my-custom-agent.md
```

4. Use with Claude Code:

```
@my-custom-agent <command>
```

### Extending Existing Sub-Agents

Edit the generated `.md` files to:
- **Add examples** in "Example Invocations" section
- **Customize constraints** in "Role Definition"
- **Adjust thresholds** in "Quality Gates"
- **Add coordination patterns** in "Communication Protocol"

## Future Sub-Agents (Planned)

- **Rust Quality Expert** (P1) - Borrow checker patterns, lifetime analysis
- **Python Quality Expert** (P1) - Type hints, async patterns
- **TypeScript Quality Expert** (P1) - Interface usage, generics
- **Test Coverage Analyst** (P1) - Coverage gaps, test suggestions
- **Refactoring Advisor** (P2) - Pattern matching, best practices
- **Quality Gate Orchestrator** (P2) - Multi-metric aggregation
- **WASM Deep Inspector** (P2) - WebAssembly bytecode analysis

## Best Practices

### 1. Use Sub-Agents for Focused Tasks

✅ **Good**:
```
@complexity-analyst analyze src/parser.rs
@mutation-tester test src/validator.rs
```

❌ **Bad**:
```
@complexity-analyst "analyze everything and generate tests and fix bugs"
```

### 2. Coordinate Multiple Sub-Agents

```
Step 1: @complexity-analyst find high-complexity functions
Step 2: @mutation-tester assess test coverage for those functions
Step 3: @refactoring-advisor suggest improvements
```

### 3. Validate Before Deployment

```bash
# Always validate custom sub-agents
pmat scaffold validate-subagent .claude/subagents/custom.md
```

### 4. Keep Definitions Updated

```bash
# Regenerate when PMAT updates
pmat scaffold create-all-subagents --force
```

## Troubleshooting

### Sub-Agent Not Found

**Error**: `No sub-agent named 'xyz' found`

**Solution**: Check available agents:
```bash
pmat scaffold list-subagents --all
```

### MCP Tool Not Available

**Error**: `Tool pmat__xyz not found`

**Solution**: Ensure PMAT MCP server is running:
```bash
# Start MCP server
pmat serve mcp --port 3000
```

### Validation Failures

**Error**: `Missing required section: ## Capabilities`

**Solution**: Use a template from existing agents:
```bash
pmat scaffold create-subagent complexity-analyst
# Copy and modify
```

## Related Documentation

- [PMAT MCP Integration](./MCP_INTEGRATION.md)
- [Quality Gates](./QUALITY_GATES.md)
- [Mutation Testing](./MUTATION_TESTING.md)
- [Complexity Analysis](./COMPLEXITY_ANALYSIS.md)

## Implementation Details

**Files**:
- `server/src/scaffold/agent/subagents.rs` - Core infrastructure
- `server/src/scaffold/agent/subagent_templates/*.md.tmpl` - Templates
- `server/src/cli/handlers/subagent_handlers.rs` - CLI handlers

**Ticket**: PMAT-7007
**Sprint**: 24
**Release**: v2.144.0
