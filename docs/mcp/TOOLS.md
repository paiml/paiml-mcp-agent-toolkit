# PMAT MCP Tools Catalog

**Protocol Version**: MCP v2024-11-05
**Last Updated**: October 19, 2025
**Total Tools**: 19

## Table of Contents

1. [Overview](#overview)
2. [Documentation Quality Tools](#documentation-quality-tools)
3. [Code Quality Tools](#code-quality-tools)
4. [Agent-Based Analysis Tools](#agent-based-analysis-tools)
5. [Deep WASM Analysis Tools](#deep-wasm-analysis-tools)
6. [Semantic Search Tools](#semantic-search-tools)
7. [Testing Tools](#testing-tools)

---

## Overview

PMAT (Polyglot Multi-Language Analysis Toolkit) exposes its code analysis capabilities via MCP (Model Context Protocol), enabling AI agents to:

- Validate documentation accuracy against actual codebase
- Analyze code quality and technical debt
- Perform deep WebAssembly analysis
- Search code semantically
- Run mutation tests
- Orchestrate complex analysis workflows

**Protocol Compliance**: All tools follow MCP v2024-11-05 specification.

---

## Documentation Quality Tools

### 1. `validate_documentation`

**Category**: Documentation Quality (Sprint 40a)
**Source**: `server/src/mcp_integration/hallucination_detection_tools.rs:21`

Validates documentation files against the actual codebase to detect hallucinations, contradictions, and broken references.

**Scientific Foundation**:
- Semantic Entropy (Farquhar et al., Nature 2024)
- MIND framework (IJCAI 2025)
- Unified Detection Framework (Complex & Intelligent Systems 2025)

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "documentation_path": {
      "type": "string",
      "description": "Path to documentation file (README.md, CLAUDE.md, etc.)"
    },
    "deep_context_path": {
      "type": "string",
      "description": "Path to deep context file (generated via `pmat context`)"
    },
    "similarity_threshold": {
      "type": "number",
      "default": 0.7,
      "description": "Similarity threshold for claim validation (0.0-1.0)"
    },
    "fail_on_error": {
      "type": "boolean",
      "default": false,
      "description": "Whether to fail on unverified/contradicted claims"
    }
  },
  "required": ["documentation_path", "deep_context_path"]
}
```

**Output Format**:
```json
{
  "status": "completed",
  "summary": {
    "total_claims": 42,
    "verified": 38,
    "unverified": 2,
    "contradictions": 1,
    "not_found": 1,
    "pass": true
  },
  "results": [
    {
      "claim": "PMAT can analyze TypeScript complexity",
      "status": "Verified",
      "confidence": 0.94,
      "evidence": ["server/src/cli/language_analyzer.rs:150"]
    }
  ]
}
```

**Example Usage**:
```javascript
const result = await mcpClient.callTool("validate_documentation", {
  documentation_path: "README.md",
  deep_context_path: "deep_context.md",
  similarity_threshold: 0.7,
  fail_on_error: true
});
```

**Use Cases**:
- Pre-commit hooks to verify documentation accuracy
- CI/CD validation gates
- Documentation quality audits
- Preventing documentation drift

---

### 2. `check_claim`

**Category**: Documentation Quality (Sprint 40a)
**Source**: `server/src/mcp_integration/hallucination_detection_tools.rs:197`

Validates a single claim against the codebase to determine if it's accurate, unverified, or a contradiction.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "claim": {
      "type": "string",
      "description": "The claim to validate"
    },
    "deep_context_path": {
      "type": "string",
      "description": "Path to deep context file"
    },
    "similarity_threshold": {
      "type": "number",
      "default": 0.7
    }
  },
  "required": ["claim", "deep_context_path"]
}
```

**Output Format**:
```json
{
  "status": "Verified",
  "confidence": 0.94,
  "evidence": ["server/src/cli/language_analyzer.rs:150"],
  "claim": "PMAT can analyze TypeScript complexity"
}
```

**Example Usage**:
```javascript
const result = await mcpClient.callTool("check_claim", {
  claim: "PMAT can analyze TypeScript complexity",
  deep_context_path: "deep_context.md",
  similarity_threshold: 0.7
});
```

**Use Cases**:
- Interactive documentation writing
- Real-time claim verification
- Documentation review workflows

---

## Code Quality Tools

### 3. `analyze_technical_debt`

**Category**: Code Quality (Sprint 40c)
**Source**: `server/src/mcp_integration/tdg_tools.rs:14`

Analyzes technical debt gradient (TDG) for files or projects, providing comprehensive quality metrics and grades (A+ to F).

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to file or directory to analyze"
    },
    "analysis_type": {
      "type": "string",
      "enum": ["file", "project", "auto"],
      "default": "auto",
      "description": "Type of analysis (auto-detects by default)"
    },
    "include_penalties": {
      "type": "boolean",
      "default": true,
      "description": "Include detailed penalty breakdown"
    }
  },
  "required": ["path"]
}
```

**Output Format (File Analysis)**:
```json
{
  "status": "completed",
  "analysis_type": "file",
  "path": "src/main.rs",
  "score": {
    "total": 78.5,
    "grade": "B",
    "confidence": 0.92,
    "language": "Rust",
    "structural_complexity": 45.2,
    "semantic_complexity": 32.1,
    "duplication_ratio": 0.05,
    "coupling_score": 0.15,
    "doc_coverage": 0.85,
    "consistency_score": 0.91,
    "entropy_score": 0.12
  },
  "penalties": [
    {
      "source_metric": "CyclomaticComplexity",
      "amount": 5.2,
      "issue": "Function has high cyclomatic complexity (15)"
    }
  ]
}
```

**Output Format (Project Analysis)**:
```json
{
  "status": "completed",
  "analysis_type": "project",
  "path": "src/",
  "total_files": 142,
  "average_score": 82.3,
  "average_grade": "B+",
  "file_scores": [
    {
      "file": "src/main.rs",
      "total": 78.5,
      "grade": "B"
    }
  ]
}
```

**Example Usage**:
```javascript
// Analyze a single file
const fileResult = await mcpClient.callTool("analyze_technical_debt", {
  path: "src/main.rs",
  include_penalties: true
});

// Analyze an entire project
const projectResult = await mcpClient.callTool("analyze_technical_debt", {
  path: "src/",
  analysis_type: "project"
});
```

**Use Cases**:
- Pre-commit quality checks
- Code review automation
- Technical debt tracking
- Quality trend analysis

---

### 4. `get_quality_recommendations`

**Category**: Code Quality (Sprint 40c)
**Source**: `server/src/mcp_integration/tdg_tools.rs:165`

Generates actionable, contextual refactoring recommendations based on quality analysis.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to file to analyze"
    },
    "max_recommendations": {
      "type": "number",
      "default": 10,
      "description": "Maximum number of recommendations to return"
    },
    "min_severity": {
      "type": "string",
      "enum": ["low", "medium", "high", "critical"],
      "default": "low",
      "description": "Minimum severity level to include"
    }
  },
  "required": ["path"]
}
```

**Output Format**:
```json
{
  "status": "completed",
  "path": "src/main.rs",
  "recommendations": [
    {
      "severity": "high",
      "category": "CyclomaticComplexity",
      "issue": "Function has high cyclomatic complexity (15)",
      "suggestion": "Consider breaking down complex functions into smaller, single-responsibility functions. Extract conditional logic into separate helper functions. Use early returns to reduce nesting depth.",
      "impact": 8.5
    },
    {
      "severity": "medium",
      "category": "NestingDepth",
      "issue": "Deep nesting detected (level 6)",
      "suggestion": "Reduce nesting depth by using early returns, guard clauses, or extracting nested blocks into separate functions. Consider using pattern matching or polymorphism to flatten control flow.",
      "impact": 4.2
    }
  ],
  "total_score": 78.5,
  "grade": "B"
}
```

**Example Usage**:
```javascript
const recommendations = await mcpClient.callTool("get_quality_recommendations", {
  path: "src/complex_module.rs",
  max_recommendations: 5,
  min_severity: "medium"
});

// Focus on critical issues only
const criticalIssues = await mcpClient.callTool("get_quality_recommendations", {
  path: "src/main.rs",
  min_severity: "critical"
});
```

**Contextual Suggestions by Category**:
- **Cyclomatic Complexity**: Break down functions, extract logic
- **Nesting Depth**: Use early returns, guard clauses
- **Duplication**: Extract common logic into shared functions
- **Coupling**: Introduce interfaces, reduce dependencies
- **Documentation**: Add docstrings, explain complex logic
- **Consistency**: Follow naming conventions, formatting rules

**Use Cases**:
- AI-assisted code review
- Automated refactoring suggestions
- Developer education
- Quality improvement roadmaps

---

## Agent-Based Analysis Tools

### 5. `analyze`

**Category**: Agent-Based Analysis
**Source**: `server/src/mcp_integration/tools.rs:13`

Invokes the analyzer agent to analyze code for quality metrics and issues.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "code": {
      "type": "string",
      "description": "Source code to analyze"
    },
    "language": {
      "type": "string",
      "description": "Programming language (rust, python, typescript, etc.)"
    },
    "metrics": {
      "type": "array",
      "items": {"type": "string"},
      "description": "Metrics to calculate (complexity, duplication, etc.)"
    },
    "priority": {
      "type": "string",
      "enum": ["low", "normal", "high", "critical"],
      "default": "normal"
    }
  },
  "required": ["code", "language"]
}
```

**Example Usage**:
```javascript
const analysis = await mcpClient.callTool("analyze", {
  code: "fn main() { println!(\"Hello\"); }",
  language: "rust",
  metrics: ["complexity", "style"],
  priority: "normal"
});
```

---

### 6. `transform`

**Category**: Agent-Based Analysis
**Source**: `server/src/mcp_integration/tools.rs:135`

Invokes the transformer agent to transform code (refactor, optimize, modernize).

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "code": {
      "type": "string",
      "description": "Source code to transform"
    },
    "transformation": {
      "type": "string",
      "description": "Type of transformation (refactor, optimize, modernize)"
    },
    "language": {
      "type": "string"
    }
  },
  "required": ["code", "transformation", "language"]
}
```

---

### 7. `validate`

**Category**: Agent-Based Analysis
**Source**: `server/src/mcp_integration/tools.rs:270`

Invokes the validator agent to validate code against rules and standards.

---

### 8. `orchestrate`

**Category**: Agent-Based Analysis
**Source**: `server/src/mcp_integration/tools.rs:425`

Orchestrates complex multi-agent workflows (analyze → transform → validate).

---

## Deep WASM Analysis Tools

### 9. `deep_wasm_analyze`

**Category**: WebAssembly Analysis
**Source**: `server/src/mcp_integration/deep_wasm_tools.rs:16`

Performs deep WebAssembly bytecode analysis.

---

### 10. `deep_wasm_query_mapping`

**Category**: WebAssembly Analysis
**Source**: `server/src/mcp_integration/deep_wasm_tools.rs:170`

Queries WASM-to-source mappings.

---

### 11. `deep_wasm_trace_execution`

**Category**: WebAssembly Analysis
**Source**: `server/src/mcp_integration/deep_wasm_tools.rs:375`

Traces WebAssembly execution paths.

---

### 12. `deep_wasm_compare_optimizations`

**Category**: WebAssembly Analysis
**Source**: `server/src/mcp_integration/deep_wasm_tools.rs:424`

Compares WebAssembly optimization strategies.

---

### 13. `deep_wasm_detect_issues`

**Category**: WebAssembly Analysis
**Source**: `server/src/mcp_integration/deep_wasm_tools.rs:474`

Detects common WebAssembly issues and anti-patterns.

---

## Semantic Search Tools

### 14. `semantic_search`

**Category**: Semantic Search
**Source**: `server/src/mcp_integration/tools.rs:679`

Searches code semantically using embeddings (requires OpenAI API key).

---

### 15. `find_similar_code`

**Category**: Semantic Search
**Source**: `server/src/mcp_integration/tools.rs:729`

Finds similar code patterns across the codebase.

---

### 16. `cluster_code`

**Category**: Semantic Search
**Source**: `server/src/mcp_integration/tools.rs:777`

Clusters code by semantic similarity.

---

### 17. `analyze_topics`

**Category**: Semantic Search
**Source**: `server/src/mcp_integration/tools.rs:825`

Analyzes code topics and themes.

---

## Testing Tools

### 18. `mutation_test`

**Category**: Testing
**Source**: `server/src/mcp_integration/mutation_tools.rs:11`

Runs mutation testing to verify test suite quality.

---

### 19. `quality_gate`

**Category**: Testing
**Source**: `server/src/mcp_integration/tools.rs:569`

Runs comprehensive quality gate checks.

---

## Error Handling

All tools follow consistent error patterns:

**Error Codes**:
- `-32700`: Parse error
- `-32600`: Invalid request
- `-32601`: Method not found
- `-32602`: Invalid parameters
- `-32603`: Internal error

**Error Response Format**:
```json
{
  "code": -32602,
  "message": "Path does not exist: /invalid/path",
  "data": {
    "path": "/invalid/path",
    "suggestion": "Please provide a valid file or directory path"
  }
}
```

**Best Practices**:
- Always check `status` field in responses
- Handle errors gracefully with fallback logic
- Use `data.suggestion` for actionable error recovery

---

## Tool Discovery

To discover available tools at runtime:

```javascript
const tools = await mcpClient.listTools();
console.log(tools.tools.map(t => t.name));
```

---

## Next Steps

- [Integration Guide](INTEGRATION.md) - How to connect to the MCP server
- [Examples](../examples/mcp/) - Complete usage examples
- [Architecture](../architecture/) - System architecture documentation

---

**Maintained by**: PMAT Development Team
**Last Updated**: Sprint 40d (October 19, 2025)
