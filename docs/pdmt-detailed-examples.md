# PDMT (Pragmatic Deterministic MCP Templating) - Detailed Examples and Usage

## Overview

PDMT is a powerful tool within PMAT that generates deterministic, high-quality todo lists with comprehensive validation requirements. This document provides extensive examples and practical usage patterns.

## Quick Start Examples

### Basic Usage via MCP in Claude Code

Simply ask Claude Code:
- "Create a PDMT todo list for implementing user authentication"
- "Generate todos for adding a payment system with Stripe"
- "Break down the task of creating a REST API"

### Command Line Examples

```bash
# Basic todo generation
cargo run --example pmcp_analyze_workflow

# Test the PDMT handler directly
cargo run --example test_pmcp_server

# Run unified MCP demo with PDMT
cargo run --example unified_mcp_demo
```

## Comprehensive Examples

### Example 1: Authentication System Implementation

```bash
# Generate todos for complete auth system
echo '{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "pdmt_deterministic_todos",
    "arguments": {
      "requirements": [
        "implement JWT-based authentication",
        "add OAuth2 support for Google and GitHub",
        "create user registration with email verification",
        "implement password reset functionality",
        "add two-factor authentication"
      ],
      "project_name": "auth_system",
      "granularity": "high",
      "quality_config": {
        "enforcement_mode": "strict",
        "coverage_threshold": 90.0,
        "max_complexity": 8,
        "require_doctests": true,
        "require_property_tests": true,
        "require_examples": true,
        "zero_satd_tolerance": true
      }
    }
  },
  "id": 1
}' | pmat
```

Expected output structure:
```yaml
todos:
  - id: auth-001
    content: "Create JWT token service module"
    implementation:
      primary_files: ["src/auth/jwt.rs"]
      test_files: ["tests/jwt_test.rs"]
      example_files: ["examples/jwt_demo.rs"]
    validation_commands:
      - "cargo test auth::jwt"
      - "cargo run --example jwt_demo"
      - "pmat quality-gate --file src/auth/jwt.rs"
    success_criteria:
      - "All JWT tests pass with >90% coverage"
      - "Token generation and validation work"
      - "Complexity ≤ 8 for all functions"
      
  - id: auth-002
    content: "Implement OAuth2 provider integration"
    dependencies: ["auth-001"]
    # ... continues for all tasks
```

### Example 2: Microservices Architecture

```bash
# Complex distributed system todos
cargo run --bin pmat -- mcp-call pdmt_deterministic_todos \
  --requirements '[
    "design microservice communication protocol",
    "implement service discovery mechanism",
    "add circuit breaker pattern",
    "create distributed tracing",
    "implement saga pattern for transactions"
  ]' \
  --project_name "microservices" \
  --granularity "high"
```

### Example 3: Data Pipeline Implementation

```rust
// Example doctest showing programmatic usage
/// Generate todos for a data pipeline project
/// 
/// ```rust
/// use pmat::services::pdmt_service::PdmtService;
/// use pmat::models::pdmt::{PdmtRequest, QualityConfig, EnforcementMode};
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = PdmtService::new();
/// 
/// let request = PdmtRequest {
///     requirements: vec![
///         "implement ETL pipeline for CSV data".to_string(),
///         "add data validation and cleansing".to_string(),
///         "create real-time streaming processor".to_string(),
///     ],
///     project_name: Some("data_pipeline".to_string()),
///     granularity: Some("medium".to_string()),
///     quality_config: Some(QualityConfig {
///         enforcement_mode: EnforcementMode::Strict,
///         coverage_threshold: 85.0,
///         max_complexity: 10,
///         require_doctests: true,
///         require_property_tests: true,
///         require_examples: true,
///         zero_satd_tolerance: true,
///     }),
/// };
/// 
/// let response = service.generate_todos(request).await?;
/// 
/// assert!(response.success);
/// assert!(response.total_todos > 0);
/// assert!(response.todo_list.is_some());
/// 
/// if let Some(todo_list) = response.todo_list {
///     for todo in todo_list.todos {
///         println!("Todo: {}", todo.content);
///         println!("  Priority: {:?}", todo.priority);
///         println!("  Estimated hours: {}", todo.estimated_hours);
///         
///         // Each todo has validation commands
///         assert!(!todo.validation_commands.unit_tests.is_empty());
///         
///         // Each todo has success criteria
///         assert!(!todo.success_criteria.is_empty());
///     }
/// }
/// # Ok(())
/// # }
/// ```
```

### Example 4: CI/CD Pipeline Setup

```bash
# Generate todos for CI/CD implementation
cat > pdmt_request.json << 'EOF'
{
  "requirements": [
    "setup GitHub Actions workflow for Rust",
    "add automated testing on pull requests",
    "implement semantic versioning and releases",
    "add code coverage reporting",
    "setup dependency security scanning"
  ],
  "project_name": "ci_cd_pipeline",
  "granularity": "high",
  "quality_config": {
    "enforcement_mode": "strict",
    "coverage_threshold": 80.0,
    "max_complexity": 5,
    "require_doctests": true,
    "require_property_tests": false,
    "require_examples": true,
    "zero_satd_tolerance": true
  }
}
EOF

# Use the request file
pmat mcp-call pdmt_deterministic_todos --file pdmt_request.json
```

## Testing PDMT Features

### Unit Tests with Doctests

```rust
/// Example doctest for PDMT validation
/// 
/// ```rust
/// use pmat::services::pdmt_quality_integration::PdmtQualityEnforcer;
/// use pmat::models::pdmt::{Todo, QualityGates};
/// 
/// let enforcer = PdmtQualityEnforcer::new();
/// 
/// let todo = Todo {
///     id: "test-001".to_string(),
///     content: "Implement user authentication".to_string(),
///     status: "pending".to_string(),
///     priority: "high".to_string(),
///     estimated_hours: 8.0,
///     dependencies: vec![],
///     quality_gates: QualityGates {
///         coverage_requirement: 85.0,
///         doctest_requirement: true,
///         property_test_requirement: true,
///         example_requirement: true,
///         complexity_limit: 8,
///         satd_tolerance: false,
///     },
///     validation_commands: Default::default(),
///     success_criteria: vec![
///         "Tests pass with >85% coverage".to_string(),
///         "All doctests execute".to_string(),
///     ],
///     implementation_specs: Default::default(),
/// };
/// 
/// // Validate the todo structure
/// let validation = enforcer.validate_todo(&todo);
/// assert!(validation.is_valid);
/// assert!(validation.messages.is_empty());
/// ```
```

### Property-Based Tests

```rust
/// Property test for PDMT determinism
/// 
/// ```rust
/// use proptest::prelude::*;
/// use pmat::services::pdmt_service::PdmtService;
/// 
/// proptest! {
///     #[test]
///     fn test_pdmt_determinism(
///         requirements in prop::collection::vec("[a-z ]{5,20}", 1..5),
///         seed in 0u64..100u64,
///     ) {
///         let service = PdmtService::new();
///         
///         // Generate todos twice with same seed
///         let todos1 = service.generate_with_seed(&requirements, seed);
///         let todos2 = service.generate_with_seed(&requirements, seed);
///         
///         // Should produce identical results
///         prop_assert_eq!(todos1, todos2);
///     }
/// }
/// ```
```

## Integration Patterns

### Pattern 1: GitHub Issue to PDMT Todos

```typescript
// GitHub Action workflow integration
// .github/workflows/issue-to-todos.yml
name: Convert Issues to PDMT Todos

on:
  issues:
    types: [opened, labeled]

jobs:
  generate-todos:
    if: contains(github.event.issue.labels.*.name, 'pdmt')
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Parse issue and generate todos
        run: |
          # Extract requirements from issue body
          REQUIREMENTS=$(echo "${{ github.event.issue.body }}" | \
            grep -E "^- \[.\]" | \
            sed 's/^- \[.\] //' | \
            jq -R . | jq -s .)
          
          # Generate PDMT todos
          pmat mcp-call pdmt_deterministic_todos \
            --requirements "$REQUIREMENTS" \
            --project_name "${{ github.repository }}" \
            --granularity high > todos.json
      
      - name: Create PR with todos
        uses: peter-evans/create-pull-request@v5
        with:
          title: "PDMT Todos for Issue #${{ github.event.issue.number }}"
          body: |
            Automated PDMT todo generation for issue #${{ github.event.issue.number }}
            
            Generated todos with quality enforcement:
            - Minimum 85% test coverage
            - Maximum complexity of 8
            - Required doctests and examples
            - Zero SATD tolerance
```

### Pattern 2: Pre-commit Quality Enforcement

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check for any TODO/FIXME comments
if git diff --cached | grep -E "TODO|FIXME|HACK|XXX"; then
    echo "❌ Error: SATD comments detected!"
    echo ""
    echo "Instead of adding TODO comments, use PDMT to generate proper todos:"
    echo ""
    echo "  pmat mcp-call pdmt_deterministic_todos \\"
    echo "    --requirements '[\"your task description\"]' \\"
    echo "    --granularity medium"
    echo ""
    echo "Or in Claude Code, simply ask:"
    echo "  'Create a PDMT todo for [your task]'"
    echo ""
    exit 1
fi

# Run quality gate on changed files
for file in $(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$'); do
    echo "Checking quality of $file..."
    if ! pmat quality-gate --file "$file"; then
        echo "❌ Quality gate failed for $file"
        echo "Run: pmat refactor auto --file $file"
        exit 1
    fi
done
```

### Pattern 3: VS Code Integration

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Generate PDMT Todos",
      "type": "shell",
      "command": "cargo",
      "args": [
        "run",
        "--example",
        "pmcp_analyze_workflow"
      ],
      "presentation": {
        "reveal": "always",
        "panel": "new"
      },
      "problemMatcher": []
    },
    {
      "label": "PDMT Todo from Selection",
      "type": "shell",
      "command": "pmat",
      "args": [
        "mcp-call",
        "pdmt_deterministic_todos",
        "--requirements",
        "[\"${selectedText}\"]",
        "--granularity",
        "medium"
      ],
      "presentation": {
        "reveal": "always",
        "focus": true
      }
    }
  ]
}
```

## Advanced Usage

### Custom Quality Configurations

```rust
/// Example of custom quality configuration
/// 
/// ```rust
/// use pmat::models::pdmt::{QualityConfig, EnforcementMode};
/// 
/// // For critical production code
/// let production_config = QualityConfig {
///     enforcement_mode: EnforcementMode::Strict,
///     coverage_threshold: 95.0,
///     max_complexity: 5,
///     require_doctests: true,
///     require_property_tests: true,
///     require_examples: true,
///     zero_satd_tolerance: true,
/// };
/// 
/// // For rapid prototyping
/// let prototype_config = QualityConfig {
///     enforcement_mode: EnforcementMode::Advisory,
///     coverage_threshold: 60.0,
///     max_complexity: 15,
///     require_doctests: false,
///     require_property_tests: false,
///     require_examples: false,
///     zero_satd_tolerance: true, // Always enforce!
/// };
/// 
/// // For legacy code refactoring
/// let refactor_config = QualityConfig {
///     enforcement_mode: EnforcementMode::AutoFix,
///     coverage_threshold: 70.0,
///     max_complexity: 10,
///     require_doctests: true,
///     require_property_tests: false,
///     require_examples: true,
///     zero_satd_tolerance: true,
/// };
/// ```
```

### Batch Processing Multiple Requirements

```bash
# Process multiple feature requests at once
cat > batch_requirements.sh << 'EOF'
#!/bin/bash

FEATURES=(
  "user authentication system"
  "payment processing with Stripe"
  "email notification service"
  "admin dashboard"
  "API rate limiting"
)

for feature in "${FEATURES[@]}"; do
  echo "Generating todos for: $feature"
  
  pmat mcp-call pdmt_deterministic_todos \
    --requirements "[\"implement $feature\"]" \
    --project_name "saas_platform" \
    --granularity "high" \
    --output "todos/${feature// /_}.json"
done

# Combine all todos into master list
jq -s 'add' todos/*.json > master_todos.json
EOF

chmod +x batch_requirements.sh
./batch_requirements.sh
```

## Validation Examples

### Example: Validating Generated Todos

```rust
/// Validate that generated todos meet quality standards
/// 
/// ```rust
/// use pmat::services::pdmt_quality_integration::validate_todo_list;
/// use pmat::models::pdmt::TodoList;
/// 
/// # fn example() -> Result<(), Box<dyn std::error::Error>> {
/// // Load generated todos
/// let todo_list: TodoList = serde_json::from_str(
///     &std::fs::read_to_string("todos.json")?
/// )?;
/// 
/// // Validate all todos
/// let validation_results = validate_todo_list(&todo_list)?;
/// 
/// // Check results
/// assert!(validation_results.all_valid);
/// 
/// for (todo_id, result) in validation_results.individual_results {
///     if !result.is_valid {
///         eprintln!("Todo {} failed validation:", todo_id);
///         for message in result.messages {
///             eprintln!("  - {}", message);
///         }
///     }
/// }
/// # Ok(())
/// # }
/// ```
```

### Example: Quality Gate Integration

```bash
# Validate todos meet quality gates before implementation
pmat mcp-call pdmt_deterministic_todos \
  --requirements '["add caching layer"]' \
  --granularity high | \
  jq '.todo_list.todos[] | select(.quality_gates.coverage_requirement >= 80)'
```

## Common Patterns and Best Practices

### 1. Incremental Feature Development

```bash
# Start with high-level requirement
INITIAL="implement e-commerce platform"

# Generate initial todos
pmat mcp-call pdmt_deterministic_todos \
  --requirements "[$INITIAL]" \
  --granularity low > phase1.json

# Break down each todo into subtasks
for todo in $(jq -r '.todo_list.todos[].content' phase1.json); do
  pmat mcp-call pdmt_deterministic_todos \
    --requirements "[$todo]" \
    --granularity high > "phase2_${todo// /_}.json"
done
```

### 2. Test-Driven Development with PDMT

```rust
/// TDD workflow with PDMT
/// 
/// ```rust
/// use pmat::services::pdmt_service::PdmtService;
/// 
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// let service = PdmtService::new();
/// 
/// // 1. Generate todos with test requirements
/// let todos = service.generate_todos_tdd(vec![
///     "implement shopping cart".to_string(),
/// ]).await?;
/// 
/// // 2. Each todo includes test specifications
/// for todo in todos.todo_list.unwrap().todos {
///     println!("Write tests for: {}", todo.content);
///     
///     // Test files are specified
///     for test_file in todo.implementation_specs.test_files {
///         println!("  Create test: {}", test_file);
///     }
///     
///     // Validation commands include test execution
///     println!("  Run: {}", todo.validation_commands.unit_tests);
/// }
/// # Ok(())
/// # }
/// ```
```

### 3. Continuous Quality Monitoring

```yaml
# .github/workflows/quality-monitor.yml
name: PDMT Quality Monitor

on:
  push:
    branches: [main, develop]
  pull_request:

jobs:
  quality-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Check all todos meet quality standards
        run: |
          # Find all todo files
          find . -name "*.todo.json" -type f | while read todo_file; do
            echo "Validating $todo_file"
            
            # Extract and validate each todo
            jq -r '.todos[].validation_commands.quality_proxy' "$todo_file" | \
              xargs -I {} sh -c '{}'
          done
```

## Troubleshooting

### Issue: "Deterministic seed not producing same results"

Solution: Ensure same environment and version:
```bash
# Lock versions
cargo update -p pmat --precise 2.3.1

# Use fixed seed
export PDMT_SEED=42
pmat mcp-call pdmt_deterministic_todos --requirements '["task"]'
```

### Issue: "Quality validation failing unexpectedly"

Debug with verbose output:
```bash
RUST_LOG=debug pmat mcp-call pdmt_deterministic_todos \
  --requirements '["your task"]' \
  --quality_config '{"enforcement_mode": "advisory"}'
```

### Issue: "Todos too granular/not granular enough"

Adjust granularity:
```bash
# Low: 1-2 todos per requirement
# Medium: 3-5 todos per requirement  
# High: 6-10 todos per requirement with full specs
```

## Additional Resources

- Run `cargo run --example pmcp_analyze_workflow` for live demo
- Run `cargo run --example test_pmcp_server` to test MCP server
- Run `cargo test pdmt` to run all PDMT tests
- See `server/src/services/pdmt_service.rs` for implementation
- See `server/src/models/pdmt.rs` for data structures
- See `docs/pdmt-integration-guide.md` for integration guide