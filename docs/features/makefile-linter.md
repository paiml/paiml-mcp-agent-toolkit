# Makefile Linter

## Overview

The Makefile Linter is a comprehensive static analysis tool that validates Makefiles against 50+ best practice rules. It provides actionable feedback to improve build system reliability, portability, and maintainability.

## Features

- **50+ Built-in Rules**: Covers syntax, portability, performance, and best practices
- **Severity Levels**: Error, Warning, Info, and Performance categorization
- **Quality Score**: Overall Makefile quality rating (0-100%)
- **False Positive Filtering**: Intelligent filtering of non-actionable issues
- **AST-based Analysis**: Accurate parsing with proper understanding of Makefile semantics
- **Integration Ready**: Works with CI/CD pipelines and build systems

## Usage

### Command Line

```bash
# Lint a Makefile
pmat lint-makefile Makefile

# Lint with custom config
pmat lint-makefile Makefile --config lint-config.toml

# Output in different formats
pmat lint-makefile Makefile --format json
pmat lint-makefile Makefile --format sarif

# Filter by severity
pmat lint-makefile Makefile --min-severity warning
```

### Programmatic Usage

```rust
use paiml_mcp_agent_toolkit::services::makefile_linter::{
    MakefileLinter, LintConfig, LintResult
};

let linter = MakefileLinter::new();
let config = LintConfig::default();
let result = linter.lint_file("Makefile", &config)?;

println!("Quality Score: {}%", result.quality_score);
for issue in result.issues {
    println!("{}: {}", issue.severity, issue.message);
}
```

## Rule Categories

### 1. **Syntax Rules**
- Missing dependencies
- Invalid target names
- Malformed variable assignments
- Recipe syntax errors

### 2. **Portability Rules**
- Shell-specific constructs
- Path separator issues
- Command availability
- Make flavor compatibility

### 3. **Performance Rules**
- Missing `.PHONY` declarations
- Inefficient wildcard usage
- Redundant shell invocations
- Parallel execution blockers

### 4. **Best Practice Rules**
- Target naming conventions
- Variable naming standards
- Documentation requirements
- Dependency organization

## Configuration

### Configuration File Format (TOML)

```toml
# lint-config.toml
[rules]
# Disable specific rules
disabled = ["simplify-conditional", "wildcard-optimization"]

# Set custom severity levels
severity_overrides = { "missing-phony" = "error" }

[thresholds]
# Minimum quality score to pass
min_quality_score = 80

# Maximum allowed issues by severity
max_errors = 0
max_warnings = 10

[output]
# Include rule explanations
include_explanations = true

# Group issues by category
group_by_category = true
```

## Rule Reference

### High-Impact Rules

#### `missing-phony`
- **Severity**: Warning
- **Description**: Targets that don't create files should be marked `.PHONY`
- **Example**:
  ```makefile
  # Bad
  clean:
      rm -rf build/

  # Good
  .PHONY: clean
  clean:
      rm -rf build/
  ```

#### `undefined-variable`
- **Severity**: Error
- **Description**: Using undefined variables can cause silent failures
- **Example**:
  ```makefile
  # Bad
  build:
      $(UNDEFINED_CC) -o app main.c

  # Good
  CC ?= gcc
  build:
      $(CC) -o app main.c
  ```

#### `recursive-assignment`
- **Severity**: Performance
- **Description**: Use `:=` for immediate assignment when appropriate
- **Example**:
  ```makefile
  # Bad (evaluated every time)
  FILES = $(shell find . -name "*.c")

  # Good (evaluated once)
  FILES := $(shell find . -name "*.c")
  ```

### Complete Rule List

| Rule ID | Category | Default Severity | Description |
|---------|----------|-----------------|-------------|
| `missing-phony` | Correctness | Warning | Non-file targets should be .PHONY |
| `undefined-variable` | Correctness | Error | Variable used before definition |
| `recursive-assignment` | Performance | Info | Prefer := over = for performance |
| `shell-in-recipe` | Portability | Warning | Shell-specific syntax detected |
| `tab-vs-space` | Syntax | Error | Recipes must start with tab |
| `wildcard-target` | Performance | Info | Wildcard in target may be slow |
| `duplicate-target` | Correctness | Error | Target defined multiple times |
| `circular-dependency` | Correctness | Error | Circular dependency detected |
| `missing-default` | Best Practice | Info | No default target specified |
| `long-line` | Style | Info | Line exceeds 120 characters |

## Output Formats

### Default Format
```
Analyzing Makefile...

⚠️ Line 10: missing-phony - Target 'clean' should be marked .PHONY
❌ Line 25: undefined-variable - Variable 'CC' is not defined
ℹ️ Line 30: recursive-assignment - Consider using := for immediate assignment

Found 3 issues (1 error, 1 warning, 1 info)
Quality score: 75%
```

### JSON Format
```json
{
  "file": "Makefile",
  "quality_score": 75,
  "issues": [
    {
      "line": 10,
      "column": 1,
      "severity": "warning",
      "rule": "missing-phony",
      "message": "Target 'clean' should be marked .PHONY",
      "suggestion": "Add '.PHONY: clean' before the target"
    }
  ],
  "summary": {
    "total": 3,
    "errors": 1,
    "warnings": 1,
    "info": 1,
    "performance": 0
  }
}
```

### SARIF Format
Compatible with GitHub Code Scanning and other SARIF consumers.

## Integration Examples

### GitHub Actions

```yaml
name: Lint Makefile
on: [push, pull_request]

jobs:
  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Install PMAT
        run: |
          curl -sSfL https://raw.githubusercontent.com/paiml/paiml-mcp-agent-toolkit/master/scripts/install.sh | sh
          echo "$HOME/.local/bin" >> $GITHUB_PATH
      
      - name: Lint Makefile
        run: pmat lint-makefile Makefile --format sarif > makefile-lint.sarif
      
      - name: Upload SARIF
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: makefile-lint.sarif
```

### Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check if Makefile was modified
if git diff --cached --name-only | grep -q "Makefile"; then
    echo "Linting Makefile..."
    pmat lint-makefile Makefile --min-severity warning
    if [ $? -ne 0 ]; then
        echo "Makefile linting failed. Please fix the issues before committing."
        exit 1
    fi
fi
```

## Best Practices

1. **Start with Default Config**: The default configuration is suitable for most projects
2. **Fix Errors First**: Address errors before warnings and info messages
3. **Use `.PHONY` Properly**: Mark all non-file targets as phony
4. **Prefer Immediate Assignment**: Use `:=` unless recursive expansion is needed
5. **Document Complex Rules**: Add comments for non-obvious make logic
6. **Test Portability**: Ensure Makefile works across different make implementations

## Troubleshooting

### Common Issues

**Q: Why am I getting "tab-vs-space" errors?**
A: Makefile recipes must start with a tab character, not spaces. Configure your editor to use tabs for Makefiles.

**Q: How do I suppress a specific warning?**
A: Use a comment directive: `# pmat-lint-disable missing-phony`

**Q: The linter reports false positives for generated files**
A: Use pattern exclusions in your config file or mark generated targets appropriately.

## Architecture

The Makefile linter uses a multi-stage architecture:

1. **Lexer**: Tokenizes the Makefile into meaningful units
2. **Parser**: Builds an Abstract Syntax Tree (AST)
3. **Analyzer**: Runs rules against the AST
4. **Reporter**: Formats and filters results

```rust
pub struct MakefileLinter {
    parser: MakefileParser,
    rules: Vec<Box<dyn Rule>>,
    config: LintConfig,
}

pub trait Rule: Send + Sync {
    fn check(&self, ast: &MakefileAst) -> Vec<Issue>;
    fn id(&self) -> &'static str;
    fn category(&self) -> RuleCategory;
}
```

## Performance

The linter is optimized for speed:
- Typical Makefile (<1000 lines): <10ms
- Large Makefile (10k lines): <100ms
- Memory usage: O(n) with file size

## Future Enhancements

- **Auto-fix Support**: Automatic correction of common issues
- **Custom Rule API**: Define project-specific rules
- **IDE Integration**: Language server protocol support
- **Incremental Linting**: Lint only changed sections