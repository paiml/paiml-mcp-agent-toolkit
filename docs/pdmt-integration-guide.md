# PDMT Integration Guide

## Overview

The PAIML MCP Agent Toolkit (pmat) now includes comprehensive integration with PDMT (Pragmatic Deterministic MCP Templating) to provide enterprise-grade, deterministic todo generation with comprehensive quality enforcement. This integration ensures that all AI-generated todos and their implementations meet strict quality standards through pmat's quality proxy infrastructure.

## Key Features

### 1. Deterministic Todo Generation
- **Reproducible outputs**: Same requirements always produce the same todo structure
- **Fixed seed generation**: Ensures consistency across runs
- **Structured task breakdown**: Automatic decomposition based on granularity levels

### 2. Quality Gate Integration
- **Mandatory quality enforcement**: All todos pass through pmat's quality proxy
- **Zero SATD tolerance**: No TODO/FIXME/HACK comments allowed
- **Coverage requirements**: Minimum 80% test coverage enforced
- **Complexity limits**: Maximum cyclomatic complexity of 8 per function

### 3. Comprehensive Validation
- **Doctest requirements**: All public APIs must have working examples
- **Property test coverage**: Complex logic requires property-based testing
- **Example validation**: All examples must execute successfully
- **Multi-layered checks**: Structure, coverage, complexity, and SATD validation

## Using the PDMT Tool

### MCP Tool Access

The PDMT tool is available as `pdmt_deterministic_todos` in the MCP server:

```json
{
  "tool": "pdmt_deterministic_todos",
  "arguments": {
    "requirements": [
      "implement user authentication with OAuth2",
      "add comprehensive logging system",
      "create REST API endpoints"
    ],
    "project_name": "my_project",
    "granularity": "high",
    "quality_config": {
      "enforcement_mode": "strict",
      "coverage_threshold": 85.0,
      "max_complexity": 8,
      "require_doctests": true,
      "require_property_tests": true,
      "require_examples": true,
      "zero_satd_tolerance": true
    }
  }
}
```

### Input Parameters

#### Required Parameters

- **`requirements`** (array of strings): List of requirements to convert into actionable todos
  - Each requirement should be a clear, specific task description
  - Example: `["implement user authentication", "add logging system"]`

#### Optional Parameters

- **`project_name`** (string): Name of the project or component
  - Default: `"project"`
  - Used for organizing and labeling generated todos

- **`granularity`** (string): Level of task detail and breakdown
  - Options: `"low"`, `"medium"`, `"high"`
  - Default: `"high"`
  - Higher granularity produces more detailed task breakdown

- **`quality_config`** (object): Quality enforcement configuration
  - `enforcement_mode`: `"strict"` (reject), `"advisory"` (warn), `"auto_fix"` (refactor)
  - `coverage_threshold`: Minimum test coverage percentage (50-100, default: 80)
  - `max_complexity`: Maximum cyclomatic complexity (1-20, default: 8)
  - `require_doctests`: Require doctest examples (default: true)
  - `require_property_tests`: Require property-based tests (default: true)
  - `require_examples`: Require working examples (default: true)
  - `zero_satd_tolerance`: Forbid SATD comments (default: true)

### Output Structure

The tool returns a comprehensive response including:

```json
{
  "success": true,
  "message": "Successfully generated 6 deterministic todos with quality enforcement",
  "total_todos": 6,
  "estimated_total_hours": 42.0,
  "todo_list": {
    "project_name": "my_project",
    "todos": [
      {
        "id": "uuid-v4",
        "content": "Implement user authentication with OAuth2",
        "status": "pending",
        "priority": "high",
        "estimated_hours": 8.0,
        "dependencies": [],
        "quality_gates": {
          "coverage_requirement": 85.0,
          "doctest_requirement": true,
          "property_test_requirement": true,
          "example_requirement": true,
          "complexity_limit": 8,
          "satd_tolerance": false
        },
        "validation_commands": {
          "unit_tests": "cargo test",
          "doctests": "cargo test --doc",
          "property_tests": "cargo test --features property-tests",
          "examples": ["cargo run --example demo"],
          "coverage_check": "cargo tarpaulin --min 85",
          "quality_proxy": "pmat quality-gate --file"
        },
        "success_criteria": [
          "Unit tests pass with >85% coverage",
          "All doctests execute successfully",
          "Property tests validate invariants",
          "Examples run without errors",
          "Quality proxy approves all changes",
          "Zero SATD comments present",
          "Complexity stays under 8 limit"
        ],
        "implementation_specs": {
          "primary_files": ["src/auth.rs"],
          "test_files": ["tests/auth_test.rs"],
          "doc_files": ["README.md"],
          "example_files": ["examples/auth_demo.rs"]
        }
      }
    ],
    "quality_config": { ... },
    "generated_at": "2024-01-15T10:30:00Z",
    "deterministic_seed": 42
  },
  "quality_validation": {
    "overall_passed": true,
    "detailed_results": { ... },
    "recommendations": []
  }
}
```

## Quality Enforcement Modes

### Strict Mode (Default)
- **Behavior**: Rejects any todo or code that doesn't meet quality standards
- **Use Case**: Production environments requiring highest quality
- **Example**: CI/CD pipelines, release branches

### Advisory Mode
- **Behavior**: Warns about quality issues but allows proceeding
- **Use Case**: Development environments, prototyping
- **Example**: Local development, feature branches

### Auto-Fix Mode
- **Behavior**: Automatically refactors code to meet quality standards
- **Use Case**: Automated code improvement workflows
- **Example**: Pre-commit hooks, automated refactoring

## Integration with Existing Workflows

### 1. CI/CD Pipeline Integration

```yaml
# .github/workflows/pdmt-todos.yml
name: Generate Quality-Enforced Todos

on:
  issues:
    types: [opened, labeled]

jobs:
  generate-todos:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install pmat
        run: cargo install pmat
      
      - name: Generate todos from issue
        run: |
          pmat mcp-call pdmt_deterministic_todos \
            --requirements "${{ github.event.issue.body }}" \
            --project_name "${{ github.repository }}" \
            --granularity high \
            --enforcement_mode strict
```

### 2. Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if any new todos are being added
if git diff --cached --name-only | xargs grep -l "TODO\|FIXME\|HACK" 2>/dev/null; then
  echo "Error: SATD comments detected. Use PDMT for proper todo generation:"
  echo "pmat mcp-call pdmt_deterministic_todos --requirements 'your task'"
  exit 1
fi
```

### 3. IDE Integration

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Generate Quality Todos",
      "type": "shell",
      "command": "pmat",
      "args": [
        "mcp-call",
        "pdmt_deterministic_todos",
        "--requirements",
        "${input:requirements}",
        "--granularity",
        "high"
      ]
    }
  ],
  "inputs": [
    {
      "id": "requirements",
      "type": "promptString",
      "description": "Enter requirements for todo generation"
    }
  ]
}
```

## Best Practices

### 1. Requirement Writing

**Good Requirements:**
- ✅ "Implement user authentication with OAuth2 support"
- ✅ "Add comprehensive error logging with structured output"
- ✅ "Create REST API endpoints for user management"

**Poor Requirements:**
- ❌ "Fix stuff" (too vague)
- ❌ "Make it work" (not actionable)
- ❌ "TODO" (not a requirement)

### 2. Granularity Selection

- **Low**: Single todo per requirement, best for simple tasks
- **Medium**: 2-3 todos per requirement, balanced approach
- **High**: Complete breakdown with tests and docs, best for complex features

### 3. Quality Configuration

For production code:
```json
{
  "enforcement_mode": "strict",
  "coverage_threshold": 90.0,
  "max_complexity": 5,
  "require_doctests": true,
  "require_property_tests": true,
  "require_examples": true,
  "zero_satd_tolerance": true
}
```

For prototypes:
```json
{
  "enforcement_mode": "advisory",
  "coverage_threshold": 70.0,
  "max_complexity": 10,
  "require_doctests": false,
  "require_property_tests": false,
  "require_examples": false,
  "zero_satd_tolerance": true
}
```

## Validation Pipeline

The PDMT integration validates todos through multiple phases:

1. **Structure Validation**
   - Content length (15-80 characters)
   - Action verb presence
   - Time estimate reasonableness (0.5-8 hours)

2. **Coverage Enforcement**
   - Minimum threshold verification
   - Test specification requirements

3. **Doctest Validation**
   - Public API documentation
   - Working examples

4. **Property Test Enforcement**
   - Complex logic coverage
   - Invariant validation

5. **Example Validation**
   - Executable examples
   - Demonstration code

6. **SATD Detection**
   - Zero tolerance for TODO/FIXME/HACK
   - Clean code enforcement

7. **Quality Proxy Validation**
   - Final quality gate
   - Comprehensive checks

## Troubleshooting

### Common Issues

#### 1. "Quality validation failed"
- **Cause**: Generated todos don't meet quality standards
- **Solution**: Adjust quality_config parameters or fix requirements

#### 2. "Requirements list cannot be empty"
- **Cause**: No requirements provided
- **Solution**: Provide at least one requirement string

#### 3. "Invalid enforcement mode"
- **Cause**: Unrecognized enforcement mode
- **Solution**: Use "strict", "advisory", or "auto_fix"

### Debug Mode

Enable debug logging for detailed information:

```bash
RUST_LOG=debug pmat mcp-call pdmt_deterministic_todos ...
```

## Examples

### Example 1: Simple Feature Implementation

```bash
pmat mcp-call pdmt_deterministic_todos \
  --requirements '["implement user login"]' \
  --granularity low
```

### Example 2: Complex System with High Quality

```bash
pmat mcp-call pdmt_deterministic_todos \
  --requirements '[
    "implement microservice communication layer",
    "add distributed tracing",
    "create health check endpoints"
  ]' \
  --project_name "microservices-platform" \
  --granularity high \
  --quality_config '{
    "enforcement_mode": "strict",
    "coverage_threshold": 95.0,
    "max_complexity": 5
  }'
```

### Example 3: Rapid Prototyping

```bash
pmat mcp-call pdmt_deterministic_todos \
  --requirements '["create MVP dashboard"]' \
  --granularity medium \
  --quality_config '{
    "enforcement_mode": "advisory",
    "coverage_threshold": 60.0
  }'
```

## API Reference

### Tool: `pdmt_deterministic_todos`

Generate deterministic, quality-enforced todo lists from requirements.

**Handler**: `PdmtTool` (server/src/mcp_pmcp/pdmt_handler.rs:100)

**Service**: `PdmtService` (server/src/services/pdmt_service.rs:11)

**Quality Enforcer**: `PdmtQualityEnforcer` (server/src/services/pdmt_quality_integration.rs:11)

**Models**: `server/src/models/pdmt.rs`

## Future Enhancements

Planned improvements for the PDMT integration:

1. **ML-Powered Quality Prediction**: Predict quality issues before generation
2. **Custom Quality Rules**: User-defined quality standards
3. **Performance Profiling Integration**: Automatic performance validation
4. **Security Scanning Integration**: Vulnerability detection in generated code
5. **Multi-language Support**: Extend beyond Rust to other languages
6. **Template Customization**: User-defined todo templates
7. **Batch Processing**: Generate todos for multiple projects simultaneously
8. **Historical Analysis**: Learn from past todo completions

## Support

For issues, questions, or contributions:

- **GitHub Issues**: https://github.com/paiml/paiml-mcp-agent-toolkit/issues
- **Documentation**: https://docs.paiml.com/pdmt-integration
- **Discord**: https://discord.gg/paiml

## License

The PDMT integration is part of the PAIML MCP Agent Toolkit and is licensed under the same terms.