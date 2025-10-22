---
name: Deep Context Generation with PMAT
description: |
  Generates comprehensive, LLM-optimized codebase context using PMAT
  (Pragmatic AI Labs MCP Agent Toolkit). Use this skill when:
  - Starting work on unfamiliar codebases
  - Onboarding to new projects or repositories
  - Need quick understanding of project architecture
  - Preparing for refactoring or feature implementation
  - Creating documentation or technical specifications
  Outputs highly compressed markdown (60-80% reduction) optimized for LLM consumption.
  Supports 25+ languages with architecture visualization, complexity heatmaps, and dependency graphs.
allowed-tools: Bash, Read, Write, Glob, Grep
---

# PMAT Deep Context Generation Skill

You are an expert at generating comprehensive codebase context using PMAT (Pragmatic AI Labs MCP Agent Toolkit).

## When to Activate

This skill should automatically activate when:
1. User asks for codebase overview, architecture, or "how does this work?"
2. Starting work on unfamiliar code ("I'm new to this project")
3. Need to understand project structure before making changes
4. Onboarding scenarios ("walk me through this codebase")
5. Creating technical documentation or specifications

## Core Command: `pmat context`

```bash
pmat context --output context.md --format llm-optimized
```

**What it does**:
- Scans entire codebase (respects .gitignore)
- Generates highly compressed markdown (60-80% smaller than raw code)
- Includes architecture diagrams (ASCII art tree structure)
- Provides complexity heatmaps and hotspot identification
- Extracts key abstractions (classes, functions, modules)
- Maps dependencies and import relationships

**Performance**:
- Small projects (<100 files): <500ms
- Medium projects (100-1K files): <2s
- Large projects (10K+ files): 5-15s

## Usage Workflow

### Step 1: Generate Full Context
When user needs comprehensive understanding:
```bash
pmat context --output deep_context.md --format llm-optimized
```

Then read the generated file:
```bash
cat deep_context.md
```

### Step 2: Analyze the Context
Look for these key sections in the generated context:

1. **Project Overview**
   - Language distribution
   - Total files/lines of code
   - Main entry points

2. **Architecture Tree**
   - Directory structure
   - Module organization
   - Key components

3. **Complexity Heatmap**
   - Files sorted by complexity score
   - Hotspots requiring attention
   - Maintainability indicators

4. **Dependency Graph**
   - Import relationships
   - Cross-module dependencies
   - External libraries

5. **Key Abstractions**
   - Classes and interfaces
   - Public APIs
   - Core data structures

### Step 3: Answer User Questions
Use the generated context to:
- Explain architecture decisions
- Identify where to make changes
- Suggest refactoring targets
- Map feature implementation paths

## Example Usage

### Example 1: Quick Codebase Tour
```bash
# User: "Can you give me a quick tour of this codebase?"

# Step 1: Generate context
pmat context --output tour.md --format llm-optimized

# Step 2: Read and analyze
cat tour.md

# Step 3: Provide structured response
# "This is a Rust-based CLI tool with the following structure:
#
# **Core Architecture** (from tour.md lines 20-45):
# - `src/main.rs` - Entry point (85 LOC)
# - `src/cli/` - Command-line interface (12 files, 1,234 LOC)
# - `src/services/` - Business logic (45 files, 8,976 LOC)
# - `src/tdg/` - Technical Debt Grading system (23 files)
#
# **Key Components**:
# 1. Deep Context Analyzer (complexity: 45)
# 2. Quality Gate System (complexity: 32)
# 3. MCP Server (Model Context Protocol)
#
# **Complexity Hotspots** (from tour.md lines 120-135):
# - `services/deep_context.rs` - Highest complexity (342)
# - `tdg/analyzer.rs` - Medium complexity (128)
# ..."
```

### Example 2: Find Implementation Location
```bash
# User: "Where should I add a new language analyzer?"

# Step 1: Generate context focused on language analyzers
pmat context --output lang_context.md --format llm-optimized

# Step 2: Grep for existing language analyzers
grep -n "LanguageAnalyzer" lang_context.md

# Step 3: Provide specific guidance
# "Based on the codebase context, add your new language analyzer here:
#
# **Location**: `src/services/languages/`
# **Pattern** (from lang_context.md):
# - Each language has its own file (rust.rs, python.rs, etc.)
# - Implement the `LanguageAnalyzer` trait
# - Register in `src/services/language_analyzer.rs:150`
#
# **Example structure**:
# ```rust
# // src/services/languages/kotlin.rs
# pub struct KotlinAnalyzer;
#
# impl LanguageAnalyzer for KotlinAnalyzer {
#     fn analyze(&self, content: &str) -> Result<Analysis> {
#         // Your implementation
#     }
# }
# ```"
```

### Example 3: Refactoring Guidance
```bash
# User: "I need to refactor the authentication module. Where should I start?"

# Step 1: Generate full context
pmat context --output refactor_context.md --format llm-optimized

# Step 2: Extract auth-related sections
grep -A 10 -B 2 "auth" refactor_context.md > auth_subset.md

# Step 3: Analyze complexity and dependencies
# "Based on the deep context analysis:
#
# **Current Auth Architecture** (from refactor_context.md):
# - `src/auth/` module (5 files, 892 LOC)
# - Complexity: 78 (HIGH - refactoring recommended)
# - Dependencies: 12 modules depend on this
#
# **Refactoring Strategy**:
# 1. Start with `auth/token.rs` (complexity: 45, lowest coupling)
# 2. Extract interface: `AuthProvider` trait
# 3. Implement: `JwtAuthProvider`, `OAuth2AuthProvider`
# 4. Update dependents (listed in refactor_context.md lines 234-256)
#
# **Risk Assessment**:
# - Medium risk: 12 dependent modules
# - Mitigation: Create adapter pattern for backward compatibility"
```

## Advanced Features

### Incremental Context Updates
For large projects, generate context incrementally:
```bash
# Generate context for specific directory
pmat context --path src/services --output services_context.md --format llm-optimized

# Combine with existing full context when needed
```

### Filter by Language
Focus on specific language files:
```bash
# Context for Rust files only
pmat context --language rust --output rust_context.md --format llm-optimized
```

### Include/Exclude Patterns
Customize what gets analyzed:
```bash
# Exclude test files from context
pmat context --exclude "**/*test.rs" --output prod_context.md --format llm-optimized
```

## Integration with Other Skills

**Combine with pmat-quality**:
1. Generate context first (architecture understanding)
2. Run quality analysis (identify issues)
3. Provide comprehensive improvement plan

**Combine with pmat-refactor**:
1. Generate context (current state)
2. Run refactor analysis (opportunities)
3. Context helps prioritize refactoring targets

## Output Format Details

The generated `context.md` file includes:

### Section 1: Metadata
```markdown
# Project Context: <project_name>
Generated: 2025-10-22
Tool: PMAT v2.170.0
Total Files: 234
Total Lines: 45,678
Languages: Rust (85%), JavaScript (10%), Other (5%)
```

### Section 2: Architecture Tree
```markdown
## Architecture
├── src/
│   ├── main.rs (85 LOC, complexity: 12)
│   ├── cli/ (12 files, 1,234 LOC)
│   ├── services/ (45 files, 8,976 LOC)
│   │   ├── deep_context.rs (complexity: 342)
│   │   └── quality_gates.rs (complexity: 128)
│   └── tdg/ (23 files, 3,456 LOC)
```

### Section 3: Complexity Heatmap
```markdown
## Complexity Hotspots
1. services/deep_context.rs - Score: 342 (REFACTOR URGENTLY)
2. tdg/analyzer.rs - Score: 128 (REVIEW)
3. cli/handlers.rs - Score: 95 (ACCEPTABLE)
```

### Section 4: Key Abstractions
```markdown
## Core Types
- `DeepContextAnalyzer`: Main analysis engine
- `QualityGate`: Quality enforcement system
- `TdgScore`: Technical debt grading
```

### Section 5: Dependency Graph
```markdown
## Dependencies
services/deep_context.rs imports from:
- services/file_discovery.rs
- services/language_analyzer.rs
- tdg/analyzer.rs
```

## Performance Optimization Tips

**For Very Large Codebases** (100K+ files):
1. Generate context for subdirectories separately
2. Cache generated context files
3. Use `--exclude` to skip vendor/node_modules
4. Generate once, reference multiple times in conversation

**Context File Size**:
- Typically 60-80% smaller than raw code
- Example: 10MB of source code → 2-3MB context file
- Still very large - use grep/search to extract relevant sections

## Caching Strategy

PMAT caches context generation:
- Cache location: `.pmat-cache/context/`
- Invalidation: Automatic on file changes
- Manual refresh: `pmat context --refresh`

## Error Handling

**If context generation fails**:
1. Check disk space (context files can be large)
2. Verify pmat version: `pmat --version` (requires v2.170.0+)
3. Try smaller scope: `pmat context --path src/ --output src_context.md`
4. Check for binary files causing issues (PMAT should skip these)

## Best Practices

1. **Generate Once Per Session**: Cache and reuse context files
2. **Start Broad, Then Narrow**: Full context first, then grep for specifics
3. **Update on Major Changes**: Regenerate context after significant refactoring
4. **Combine Tools**: Use context + quality + refactor for comprehensive analysis
5. **Share Context**: Include context.md in issue reports for better bug triage

## Scientific Foundation

Context generation implements:
- **Halstead Metrics**: Program vocabulary and volume
- **Information Retrieval**: TF-IDF for key term extraction
- **Graph Theory**: Dependency network analysis
- **Compression Algorithms**: Semantic deduplication (60-80% reduction)

## Limitations

- **Very Large Files**: Files >10K LOC may be summarized rather than fully included
- **Binary Files**: Skipped (images, compiled code, etc.)
- **Generated Code**: Included but may inflate context size
- **Minified Code**: Analyzed but contributes little semantic value

## When NOT to Use This Skill

- **Syntax Errors**: PMAT requires parseable code
- **Empty Repositories**: Need at least some files to analyze
- **Real-time Editing**: Context reflects files on disk, not in-memory buffers
- **Partial Clones**: Works best with full repository checkout

## Version Requirements

- **Minimum**: PMAT v2.170.0
- **Recommended**: Latest version for best compression
- **Check version**: `pmat --version`

---

**Remember**: Always generate deep context BEFORE diving into code changes. Understanding the architecture prevents costly mistakes and identifies the right place for modifications.
