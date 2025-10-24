# JVM Language Analysis Tools for MCP

**Protocol Version**: MCP v2024-11-05
**Last Updated**: October 24, 2025
**Sprint**: 51

## Table of Contents

1. [Overview](#overview)
2. [Java Analysis Tools](#java-analysis-tools)
3. [Scala Analysis Tools](#scala-analysis-tools)
4. [Tool Usage Examples](#tool-usage-examples)
5. [Integration with Other Tools](#integration-with-other-tools)

---

## Overview

The JVM Language Analysis Tools extend PMAT's MCP (Model Context Protocol) capabilities to include comprehensive support for Java and Scala programming languages. These tools enable AI agents to analyze, understand, and work with JVM-based codebases.

**Features:**

- Analysis of code structure and complexity
- Detection of language-specific constructs
- Calculation of quality metrics
- Mutation testing support
- Feature-gated behind Cargo features (`java-ast` and `scala-ast`)

---

## Java Analysis Tools

### 1. `analyze_java`

**Category**: Code Analysis (Sprint 51)
**Source**: `server/src/mcp_integration/java_tools.rs:11`

Analyzes Java source code for complexity, structure, and quality metrics.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to Java file or directory to analyze"
    },
    "max_depth": {
      "type": "number",
      "description": "Maximum depth for recursive directory analysis",
      "default": 3
    },
    "include_metrics": {
      "type": "boolean",
      "description": "Include detailed complexity metrics",
      "default": true
    },
    "include_ast": {
      "type": "boolean",
      "description": "Include AST items in result",
      "default": false
    }
  },
  "required": ["path"]
}
```

**Output Format (File Analysis)**:
```json
{
  "status": "completed",
  "path": "src/main/java/com/example/App.java",
  "language": "java",
  "summary": {
    "class_count": 2,
    "interface_count": 1,
    "method_count": 5,
    "package": "com.example",
    "total_items": 12
  },
  "metrics": {
    "total_complexity": 15,
    "max_complexity": 6,
    "avg_complexity": 3.0,
    "loc": 78
  }
}
```

**Output Format (Directory Analysis)**:
```json
{
  "status": "completed",
  "path": "src/main/java/com/example",
  "language": "java",
  "summary": {
    "file_count": 5,
    "class_count": 8,
    "interface_count": 3,
    "method_count": 24
  },
  "metrics": {
    "total_complexity": 68,
    "max_complexity": 12,
    "avg_complexity": 2.83,
    "total_loc": 345
  }
}
```

### 2. `mutation_test_java`

**Category**: Testing (Sprint 51)
**Source**: `server/src/mcp_integration/java_tools.rs:286`

Performs mutation testing on Java code to assess test suite quality.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project_path": {
      "type": "string",
      "description": "Path to Java project root"
    },
    "source_path": {
      "type": "string",
      "description": "Path to source file or directory to mutate"
    },
    "test_command": {
      "type": "string",
      "description": "Command to run tests (defaults to 'mvn test' or 'gradle test')"
    },
    "mutation_operators": {
      "type": "array",
      "description": "List of mutation operators to apply",
      "items": {"type": "string"},
      "default": ["arithmetic", "conditional", "method", "assignment"]
    },
    "timeout": {
      "type": "number",
      "description": "Timeout in seconds for each test run",
      "default": 30
    }
  },
  "required": ["project_path", "source_path"]
}
```

**Output Format**:
```json
{
  "status": "completed",
  "message": "Java mutation testing completed",
  "project_path": "/path/to/java-project",
  "source_path": "/path/to/java-project/src/main/java",
  "test_command": "mvn test",
  "mutation_operators": ["arithmetic", "conditional", "method", "assignment"],
  "timeout": 30,
  "results": {
    "mutants_generated": 120,
    "mutants_killed": 96,
    "mutants_survived": 24,
    "mutation_score": 80.0,
    "runtime_seconds": 45
  }
}
```

---

## Scala Analysis Tools

### 1. `analyze_scala`

**Category**: Code Analysis (Sprint 51)
**Source**: `server/src/mcp_integration/scala_tools.rs:11`

Analyzes Scala source code for complexity, structure, and quality metrics with a focus on both object-oriented and functional programming patterns.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "path": {
      "type": "string",
      "description": "Path to Scala file or directory to analyze"
    },
    "max_depth": {
      "type": "number",
      "description": "Maximum depth for recursive directory analysis",
      "default": 3
    },
    "include_metrics": {
      "type": "boolean",
      "description": "Include detailed complexity metrics",
      "default": true
    },
    "include_ast": {
      "type": "boolean",
      "description": "Include AST items in result",
      "default": false
    }
  },
  "required": ["path"]
}
```

**Output Format (File Analysis)**:
```json
{
  "status": "completed",
  "path": "src/main/scala/com/example/App.scala",
  "language": "scala",
  "summary": {
    "class_count": 1,
    "trait_count": 2,
    "object_count": 1,
    "case_class_count": 3,
    "method_count": 8,
    "package": "com.example",
    "total_items": 18
  },
  "metrics": {
    "total_complexity": 24,
    "max_complexity": 5,
    "avg_complexity": 3.0,
    "functional_percentage": 72.5,
    "loc": 95
  }
}
```

**Output Format (Directory Analysis)**:
```json
{
  "status": "completed",
  "path": "src/main/scala/com/example",
  "language": "scala",
  "summary": {
    "file_count": 7,
    "class_count": 5,
    "trait_count": 4,
    "object_count": 8,
    "case_class_count": 12,
    "method_count": 36
  },
  "metrics": {
    "total_complexity": 92,
    "max_complexity": 9,
    "avg_complexity": 2.55,
    "functional_percentage": 68.3,
    "total_loc": 520
  }
}
```

### 2. `mutation_test_scala`

**Category**: Testing (Sprint 51)
**Source**: `server/src/mcp_integration/scala_tools.rs:305`

Performs mutation testing on Scala code to assess test suite quality, with additional support for functional programming mutation operators.

**Input Schema**:
```json
{
  "type": "object",
  "properties": {
    "project_path": {
      "type": "string",
      "description": "Path to Scala project root"
    },
    "source_path": {
      "type": "string",
      "description": "Path to source file or directory to mutate"
    },
    "test_command": {
      "type": "string",
      "description": "Command to run tests (defaults to 'sbt test')"
    },
    "mutation_operators": {
      "type": "array",
      "description": "List of mutation operators to apply",
      "items": {"type": "string"},
      "default": ["arithmetic", "conditional", "method", "functional"]
    },
    "timeout": {
      "type": "number",
      "description": "Timeout in seconds for each test run",
      "default": 30
    }
  },
  "required": ["project_path", "source_path"]
}
```

**Output Format**:
```json
{
  "status": "completed",
  "message": "Scala mutation testing completed",
  "project_path": "/path/to/scala-project",
  "source_path": "/path/to/scala-project/src/main/scala",
  "test_command": "sbt test",
  "mutation_operators": ["arithmetic", "conditional", "method", "functional"],
  "timeout": 30,
  "results": {
    "mutants_generated": 150,
    "mutants_killed": 135,
    "mutants_survived": 15,
    "mutation_score": 90.0,
    "runtime_seconds": 60
  }
}
```

---

## Tool Usage Examples

### Java Analysis Example

```javascript
// Analyze a Java file
const javaAnalysis = await mcpClient.callTool("analyze_java", {
  path: "src/main/java/com/example/App.java",
  include_metrics: true
});

console.log(`Found ${javaAnalysis.summary.method_count} methods with average complexity ${javaAnalysis.metrics.avg_complexity}`);

// Analyze a Java project directory
const projectAnalysis = await mcpClient.callTool("analyze_java", {
  path: "src/main/java/com/example",
  max_depth: 4
});

console.log(`Project has ${projectAnalysis.summary.file_count} files with ${projectAnalysis.summary.class_count} classes`);
```

### Scala Analysis Example

```javascript
// Analyze a Scala file with AST information
const scalaAnalysis = await mcpClient.callTool("analyze_scala", {
  path: "src/main/scala/com/example/App.scala",
  include_metrics: true,
  include_ast: true
});

// Check functional programming percentage
console.log(`Functional programming percentage: ${scalaAnalysis.metrics.functional_percentage}%`);
console.log(`Case classes: ${scalaAnalysis.summary.case_class_count}, Traits: ${scalaAnalysis.summary.trait_count}`);

// Find all methods in AST
const methods = scalaAnalysis.items.filter(item => item.kind === "method" || item.kind === "function");
console.log(`Method details:`, methods);
```

### Mutation Testing Example

```javascript
// Run Java mutation testing
const javaMutationResults = await mcpClient.callTool("mutation_test_java", {
  project_path: "/path/to/java-project",
  source_path: "src/main/java/com/example/Calculator.java",
  mutation_operators: ["arithmetic", "conditional"]
});

console.log(`Mutation score: ${javaMutationResults.results.mutation_score}%`);

// Run Scala mutation testing
const scalaMutationResults = await mcpClient.callTool("mutation_test_scala", {
  project_path: "/path/to/scala-project",
  source_path: "src/main/scala/com/example/Calculator.scala"
});

console.log(`Mutation score: ${scalaMutationResults.results.mutation_score}%`);
```

---

## Integration with Other Tools

The JVM language tools work seamlessly with other MCP tools in the PMAT ecosystem:

### With Quality Analysis

```javascript
// Analyze Java code quality
const javaAnalysis = await mcpClient.callTool("analyze_java", {
  path: "src/main/java/com/example/App.java"
});

// Get refactoring recommendations based on complexity
if (javaAnalysis.metrics.avg_complexity > 4.0) {
  const recommendations = await mcpClient.callTool("get_quality_recommendations", {
    path: "src/main/java/com/example/App.java",
    min_severity: "medium"
  });
  
  console.log("Refactoring recommendations:", recommendations);
}
```

### With Documentation Validation

```javascript
// Generate context from Java project
await runCommand('pmat context --include "**/*.java" --output java_context.md');

// Validate documentation against Java codebase
const validation = await mcpClient.callTool("validate_documentation", {
  documentation_path: "java-docs.md",
  deep_context_path: "java_context.md",
  similarity_threshold: 0.7
});

console.log("Documentation validation:", validation.summary);
```

---

**Maintained by**: PMAT Development Team
**Last Updated**: Sprint 51 (October 24, 2025)