# Documentation Enforcement

**Status**: ✅ Production (v2.142.0)
**PMAT-7001**: Phase 3 (REFACTOR) Complete

## Overview

Documentation Enforcement is a quality gate system that automatically validates the quality of CLI help text and MCP tool documentation. It ensures all documentation is complete, accurate, and non-generic.

## Features

### Automated Validation
- **MCP Tool Documentation**: Validates all MCP tool descriptions and parameters
- **CLI Documentation**: Validates CLI command help text (via test suite)
- **Generic Detection**: Identifies placeholder/low-quality documentation
- **Parameter Validation**: Ensures all parameters have types, descriptions, and defaults

### Quality Gate Integration
- Integrated into PMAT's quality gate system
- Works alongside complexity, SATD, and dead code checks
- Configurable CLI/MCP validation flags
- Supports strict mode for build blocking

### JSON Reporting
- Export comprehensive validation reports
- Programmatic analysis for CI/CD integration
- Per-tool and per-parameter metrics
- Issue tracking with severity levels

### Pre-commit Hooks
- Automatic validation before commits
- Fast execution (<100ms)
- Clear error messages with remediation steps
- Configurable enforcement via environment variable

## Usage

### Command Line

```bash
# Run documentation enforcement via quality gate
pmat quality-gate --checks docs-enforcement

# Check MCP documentation only
pmat quality-gate --checks docs-enforcement --check-mcp

# Check both CLI and MCP
pmat quality-gate --checks docs-enforcement --check-cli --check-mcp

# Strict mode (fail build on violations)
pmat quality-gate --checks docs-enforcement --strict
```

### Programmatic API

#### Quality Gate Integration

```rust
use pmat::services::quality_gate_service::{
    QualityCheck, QualityGateInput, QualityGateService
};
use pmat::services::service_base::Service;
use std::path::PathBuf;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let service = QualityGateService::new();

    let input = QualityGateInput {
        path: PathBuf::from("."),
        checks: vec![QualityCheck::DocsEnforcement {
            check_cli: false,
            check_mcp: true,
        }],
        strict: false,
    };

    let output = service.process(input).await?;

    println!("Passed: {}", output.passed);
    println!("Total checks: {}", output.summary.total_checks);
    println!("Violations: {}", output.summary.total_violations);

    Ok(())
}
```

#### JSON Export

```rust
use pmat::docs_enforcement::mcp_checker::generate_validation_report_json;

fn main() -> anyhow::Result<()> {
    let json = generate_validation_report_json()?;
    println!("{}", json);
    Ok(())
}
```

**JSON Output Structure**:
```json
{
  "total_tools": 4,
  "valid_tools": 4,
  "invalid_tools": 0,
  "total_issues": 0,
  "tools": [
    {
      "tool_name": "scaffold_agent",
      "has_description": true,
      "description_length": 92,
      "description_is_generic": false,
      "has_input_schema": true,
      "parameters": [
        {
          "name": "name",
          "has_description": true,
          "description": "Agent project name (lowercase, alphanumeric, hyphens only)",
          "description_is_generic": false,
          "has_type": true,
          "param_type": "string",
          "is_required": true,
          "has_default": false,
          "issues": []
        }
      ],
      "issues": []
    }
  ]
}
```

### Pre-commit Hook

Enable documentation enforcement in your pre-commit hook:

```bash
# In .git/hooks/pre-commit or environment
export PMAT_DOCS_ENFORCEMENT_ENABLED=true

# Hook runs automatically on commit
git commit -m "Your changes"
```

**Manual Installation**:
```bash
# Install PMAT pre-commit hook
pmat hooks install

# Verify hook is active
pmat hooks status
```

## Validation Rules

### Tool Documentation

A valid tool must have:
- ✅ **Description**: Non-empty, >20 characters
- ✅ **Non-generic**: Not placeholder text (e.g., "A tool", "Helper function")
- ✅ **Input Schema**: Valid parameter schema defined
- ✅ **All Parameters**: Every parameter properly documented

### Parameter Documentation

A valid parameter must have:
- ✅ **Description**: Non-empty, >15 characters
- ✅ **Type**: Explicit type specification (string, number, boolean, array, object)
- ✅ **Required Status**: Clear indication if required or optional
- ✅ **Default Values**: Optional parameters document their defaults
- ✅ **Non-generic**: Meaningful description, not "The parameter" or "A value"

### Generic Detection

The system flags these as generic:
- "A tool/function/parameter that..."
- "Helper for..."
- "Utility to..."
- "The [name] parameter"
- Empty or very short descriptions (<15 chars)

## Configuration

### Quality Gate Configuration

```toml
# .pmat.toml or pmat.toml
[quality_gate]
docs_enforcement = { check_cli = true, check_mcp = true }
strict = false
```

### Pre-commit Hook Configuration

```bash
# Environment variables
export PMAT_DOCS_ENFORCEMENT_ENABLED=true   # Enable/disable check
export PMAT_DOCS_CHECK_CLI=true             # Check CLI docs
export PMAT_DOCS_CHECK_MCP=true             # Check MCP docs
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Documentation Quality

on: [push, pull_request]

jobs:
  docs-check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install PMAT
        run: cargo install pmat

      - name: Check Documentation Quality
        run: |
          pmat quality-gate --checks docs-enforcement --strict --format json > docs-report.json

      - name: Upload Report
        uses: actions/upload-artifact@v3
        with:
          name: docs-report
          path: docs-report.json
```

### GitLab CI

```yaml
docs-check:
  stage: test
  script:
    - cargo install pmat
    - pmat quality-gate --checks docs-enforcement --strict
  artifacts:
    reports:
      junit: docs-report.xml
```

## Examples

### Check All MCP Tools

```bash
$ pmat quality-gate --checks docs-enforcement

✅ Documentation Enforcement (PMAT-7001)
   MCP: checked, CLI: skipped

   Results:
   - scaffold_agent: ✅ 5 parameters, 0 issues
   - validate_roadmap: ✅ 2 parameters, 0 issues
   - health_check: ✅ 7 parameters, 0 issues
   - generate_tickets: ✅ 3 parameters, 0 issues

   Total: 4 tools, 17 parameters, 0 issues
```

### Strict Mode (Fail on Violations)

```bash
$ pmat quality-gate --checks docs-enforcement --strict

❌ Documentation Enforcement (PMAT-7001)
   1 documentation issue found

   Violations:
   - MCP tool: my_tool
     Error: Tool description too short (15 chars, min 20)

   Exit code: 1
```

### JSON Output

```bash
$ pmat quality-gate --checks docs-enforcement --format json | jq '.results[] | select(.check | contains("Documentation"))'

{
  "check": "Documentation Enforcement (PMAT-7001)",
  "passed": true,
  "message": "Documentation enforcement passed (MCP: checked, CLI: skipped)",
  "violations": []
}
```

## Performance

| Operation | Time | Notes |
|-----------|------|-------|
| Pre-commit hook | <100ms | Fast enough for real-time feedback |
| Quality gate | ~5s | Full validation with all checks |
| JSON export | <10ms | Suitable for CI/CD pipelines |
| Test suite | <20ms | 6 comprehensive tests |

## Troubleshooting

### Common Issues

**Issue**: "Tool description is generic"
```
Error: Tool description is generic: 'A tool that processes data'
```
**Solution**: Provide specific description explaining what the tool does:
```rust
"Analyze code complexity using cyclomatic and cognitive metrics"
```

**Issue**: "Optional parameter should document default value"
```
Error: Optional parameter 'timeout' should document default value
```
**Solution**: Include default in description:
```rust
"Timeout in seconds (default: 30)"
```

**Issue**: "Parameter description too short"
```
Error: Parameter 'path' description too short (10 chars)
```
**Solution**: Provide detailed description (>15 chars):
```rust
"Path to project directory for analysis"
```

### Debug Mode

```bash
# Run tests with detailed output
cargo test --test docs_enforcement_unit_test -- --nocapture

# Check specific tool validation
pmat validate-docs --tool scaffold_agent --verbose
```

## Implementation Details

### Architecture

```
docs_enforcement/
├── generic_detector.rs  # Generic text detection
├── mcp_checker.rs       # MCP tool validation
└── cli_checker.rs       # CLI documentation validation

services/
└── quality_gate_service.rs  # Quality gate integration
```

### Test Coverage

- **Unit Tests**: 2 tests (validation logic, JSON export)
- **Integration Tests**: 4 tests (quality gate integration)
- **Total Coverage**: 6/6 tests passing (100%)

### Validation Flow

```
1. Load tool definitions
2. For each tool:
   a. Validate description (length, generic check)
   b. Validate input schema exists
   c. For each parameter:
      - Check description (length, generic)
      - Check type specification
      - Check default documentation (optional params)
3. Generate validation report
4. Return pass/fail with violations
```

## Related Features

- **[Quality Gates](../operations/quality-gates.md)** - Overall quality gate system
- **[Pre-commit Hooks](../operations/hooks.md)** - Hook configuration
- **[MCP Protocol](./mcp-protocol.md)** - MCP tool development

## Version History

- **v2.142.0** (2025-10-06): Phase 3 complete - Quality gate integration, JSON export, pre-commit hooks
- **v2.141.0** (2025-10-06): Phase 2 complete - MCP validation working
- **v2.140.0** (2025-10-05): Phase 1 complete - Test framework created

## References

- **Specification**: [CLI_MCP_DOCUMENTATION_ENFORCEMENT.md](../specifications/CLI_MCP_DOCUMENTATION_ENFORCEMENT.md)
- **Ticket**: [TICKET-PMAT-7001.md](../tickets/TICKET-PMAT-7001.md)
- **Phase 3 Results**: [PMAT-7001-REFACTOR-PHASE-RESULTS.md](../tickets/PMAT-7001-REFACTOR-PHASE-RESULTS.md)
- **Changelog**: [CHANGELOG.md](../../CHANGELOG.md)

## Contributing

When adding new MCP tools or CLI commands:

1. Provide detailed, specific descriptions (>20 chars for tools, >15 for params)
2. Avoid generic placeholders ("A tool that...", "Helper for...")
3. Document all parameter types explicitly
4. Document default values for optional parameters
5. Run `cargo test --test docs_enforcement_unit_test` before committing

## License

Part of PMAT - Licensed under MIT OR Apache-2.0
