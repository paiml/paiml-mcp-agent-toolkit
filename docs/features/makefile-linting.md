# Makefile Linting

## Overview

The Makefile linting feature provides comprehensive analysis of Makefile quality, identifying common issues, suggesting improvements, and ensuring best practices. It implements over 50 rules covering portability, performance, maintainability, and correctness.

## Command Usage

### Basic Linting

```bash
# Lint Makefile in current directory
pmat analyze makefile-lint

# Lint specific Makefile
pmat analyze makefile-lint --makefile /path/to/Makefile

# Multiple Makefiles
pmat analyze makefile-lint --makefile Makefile --makefile Makefile.inc
```

### Output Formats

```bash
# Default human-readable format
pmat analyze makefile-lint

# JSON for tooling integration
pmat analyze makefile-lint --format json

# SARIF for IDE integration
pmat analyze makefile-lint --format sarif

# Checkmake-compatible output
pmat analyze makefile-lint --format checkmake
```

### Filtering and Configuration

```bash
# Show only errors (no warnings)
pmat analyze makefile-lint --min-severity error

# Enable all rules including pedantic
pmat analyze makefile-lint --pedantic

# Disable specific rules
pmat analyze makefile-lint --disable-rules "line-length,tab-usage"

# Use configuration file
pmat analyze makefile-lint --config .makefile-lint.yml
```

## Rule Categories

### 1. Portability Rules

Ensure Makefiles work across different platforms and make implementations:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| `portable-shell` | Use POSIX-compatible shell constructs | Error |
| `shell-assignment` | Use `=` not `:=` for shell assignments | Warning |
| `platform-commands` | Avoid platform-specific commands | Warning |
| `gnu-make-features` | Flag GNU-specific features | Info |
| `bsd-make-compat` | Check BSD make compatibility | Info |

Example violations:

```makefile
# Bad: Bash-specific syntax
check:
    if [[ -f file ]]; then echo "exists"; fi

# Good: POSIX syntax
check:
    if [ -f file ]; then echo "exists"; fi

# Bad: GNU-specific
sources := $(wildcard *.c)

# Good: Portable
sources = main.c utils.c parser.c
```

### 2. Performance Rules

Optimize Makefile execution speed:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| `recursive-make` | Avoid unnecessary recursive make | Warning |
| `shell-loops` | Use make patterns instead of shell loops | Warning |
| `expensive-commands` | Cache results of expensive operations | Info |
| `parallel-unsafe` | Identify parallel execution issues | Error |
| `phony-targets` | Declare phony targets properly | Warning |

Example improvements:

```makefile
# Bad: Shell loop
all:
    for file in *.c; do \
        gcc -c $$file; \
    done

# Good: Make pattern
SRCS = $(wildcard *.c)
OBJS = $(SRCS:.c=.o)

all: $(OBJS)

%.o: %.c
    gcc -c $< -o $@
```

### 3. Maintainability Rules

Improve readability and maintainability:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| `line-length` | Lines should not exceed 80 characters | Info |
| `indentation` | Consistent tab indentation | Warning |
| `variable-naming` | Use UPPER_CASE for variables | Info |
| `target-naming` | Use lowercase-hyphenated for targets | Info |
| `documentation` | Document complex rules | Info |
| `variable-expansion` | Use ${} consistently | Info |

Example style improvements:

```makefile
# Bad: Inconsistent, undocumented
src=$(wildcard src/*.c)
buildDir=build

compile: $(src)
	gcc $(src) -o $(buildDir)/app

# Good: Consistent, documented
# Source files
SRC = $(wildcard src/*.c)
BUILD_DIR = build

# Build the application
compile: ${SRC}
	gcc ${SRC} -o ${BUILD_DIR}/app
```

### 4. Correctness Rules

Prevent common errors:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| `missing-deps` | Targets missing dependencies | Error |
| `circular-deps` | Circular dependency detection | Error |
| `undefined-vars` | Reference to undefined variables | Warning |
| `duplicate-targets` | Multiple definitions of same target | Error |
| `whitespace-errors` | Mixed spaces/tabs in recipes | Error |
| `continuation-errors` | Incorrect line continuations | Error |

Example fixes:

```makefile
# Bad: Missing dependencies
app:
    gcc main.c utils.c -o app

# Good: Explicit dependencies
app: main.o utils.o
    gcc $^ -o $@

main.o: main.c common.h
utils.o: utils.c utils.h common.h
```

### 5. Best Practice Rules

Encourage modern Makefile practices:

| Rule ID | Description | Severity |
|---------|-------------|----------|
| `default-goal` | Explicitly set .DEFAULT_GOAL | Info |
| `silent-rules` | Use @ for cleaner output | Info |
| `error-handling` | Check command success | Warning |
| `clean-target` | Provide proper clean target | Warning |
| `help-target` | Include help documentation | Info |
| `version-check` | Check make version if needed | Info |

Example best practices:

```makefile
# Good: Modern Makefile structure
.DEFAULT_GOAL := help

# Version check
MINIMUM_MAKE_VERSION := 4.0
ifneq ($(firstword $(sort $(MAKE_VERSION) $(MINIMUM_MAKE_VERSION))),$(MINIMUM_MAKE_VERSION))
    $(error Make version $(MINIMUM_MAKE_VERSION) or higher required)
endif

.PHONY: help
help: ## Show this help message
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-20s\033[0m %s\n", $$1, $$2}'

.PHONY: build
build: ## Build the project
	@echo "Building..."
	@gcc $(CFLAGS) $(SRCS) -o $(TARGET)

.PHONY: clean
clean: ## Clean build artifacts
	@echo "Cleaning..."
	@rm -rf $(BUILD_DIR) $(TARGET)
```

## Output Examples

### Human-Readable Format

```
Makefile Linting Results
════════════════════════════════════════════════════════════════

Makefile:12:1 [ERROR] missing-deps
  Target 'install' is missing dependency on 'build'
  
  12 | install:
  13 |     cp app /usr/local/bin/
  
  Suggestion: Add 'build' as a dependency:
  install: build

Makefile:25:5 [WARNING] shell-loops
  Using shell loop instead of Make pattern
  
  25 |     for test in $(TESTS); do \
  26 |         ./$$test || exit 1; \
  27 |     done
  
  Suggestion: Use pattern rule or foreach function

Makefile:35:80 [INFO] line-length
  Line exceeds 80 characters (currently 92)
  
  Consider breaking into multiple lines with \

Summary:
  Errors:   2
  Warnings: 5
  Info:     8
  Total:    15
```

### JSON Format

```json
{
  "issues": [
    {
      "rule": "missing-deps",
      "severity": "error",
      "file": "Makefile",
      "line": 12,
      "column": 1,
      "message": "Target 'install' is missing dependency on 'build'",
      "suggestion": "Add 'build' as a dependency",
      "fix": {
        "line": 12,
        "original": "install:",
        "replacement": "install: build"
      }
    }
  ],
  "summary": {
    "total": 15,
    "errors": 2,
    "warnings": 5,
    "info": 8,
    "fixable": 12
  },
  "metrics": {
    "lines": 156,
    "targets": 12,
    "variables": 23,
    "complexity": 4.5
  }
}
```

## Configuration File

Create `.makefile-lint.yml` for project-specific configuration:

```yaml
# Rule severity overrides
rules:
  line-length:
    severity: warning
    max-length: 100
  
  variable-naming:
    enabled: false
  
  portable-shell:
    severity: error
    shells: [sh, bash, zsh]

# Global settings
settings:
  pedantic: false
  fix-suggestions: true
  color-output: true

# Ignore patterns
ignore:
  - "vendor/**"
  - "third_party/**"
  - "*.generated.mk"

# Custom rules
custom-rules:
  - id: "project-prefix"
    pattern: "^[^A-Z]*="
    message: "Variables should start with PROJECT_"
    severity: warning
```

## Auto-Fix Support

Some issues can be automatically fixed:

```bash
# Show what would be fixed
pmat analyze makefile-lint --fix --dry-run

# Apply fixes
pmat analyze makefile-lint --fix

# Fix specific rules only
pmat analyze makefile-lint --fix --fix-rules "whitespace,line-continuation"
```

Fixable issues include:
- Whitespace errors
- Line continuation formatting
- Variable expansion style
- Missing .PHONY declarations
- Simple dependency additions

## Integration

### Pre-commit Hook

```yaml
# .pre-commit-config.yaml
repos:
  - repo: local
    hooks:
      - id: makefile-lint
        name: Lint Makefiles
        entry: pmat analyze makefile-lint
        language: system
        files: ^Makefile$|\.mk$
```

### CI/CD Integration

```yaml
# GitHub Actions
- name: Lint Makefiles
  run: |
    pmat analyze makefile-lint --format json > makefile-lint.json
    
    errors=$(jq '.summary.errors' makefile-lint.json)
    if [ "$errors" -gt 0 ]; then
      echo "::error::Found $errors errors in Makefiles"
      jq '.issues[] | select(.severity == "error")' makefile-lint.json
      exit 1
    fi
```

### Editor Integration

```vim
" Vim integration
autocmd BufWritePost Makefile,*.mk !pmat analyze makefile-lint --format short %
```

## Common Issues and Solutions

### 1. Tab vs Space Issues

```makefile
# Bad: Mixed indentation
target:
    command1  # Tab
  command2    # Spaces - ERROR!

# Good: Consistent tabs
target:
	command1
	command2
```

### 2. Shell Compatibility

```makefile
# Bad: Bash-specific
check:
	[[ -f file ]] && echo "exists"

# Good: POSIX
check:
	[ -f file ] && echo "exists"
```

### 3. Variable Expansion

```makefile
# Bad: Inconsistent
FILES = $(SRC_DIR)/*.c
OBJ = ${FILES:.c=.o}

# Good: Consistent
FILES = ${SRC_DIR}/*.c
OBJ = ${FILES:.c=.o}
```

### 4. Dependency Management

```makefile
# Bad: Manual dependencies
app: main.c utils.c
	gcc main.c utils.c -o app

# Good: Automatic dependencies
SRCS = main.c utils.c
DEPS = $(SRCS:.c=.d)

-include $(DEPS)

%.d: %.c
	@gcc -MM $< > $@
```

## Performance Tips

### 1. Avoid Recursive Make

```makefile
# Bad: Recursive
all:
	$(MAKE) -C subdir1
	$(MAKE) -C subdir2

# Good: Include pattern
include subdir1/rules.mk
include subdir2/rules.mk
```

### 2. Use Pattern Rules

```makefile
# Bad: Repetitive
main.o: main.c
	gcc -c main.c

utils.o: utils.c
	gcc -c utils.c

# Good: Pattern
%.o: %.c
	gcc -c $< -o $@
```

### 3. Cache Expensive Operations

```makefile
# Bad: Runs every time
VERSION = $(shell git describe --tags)

# Good: Cache result
VERSION ?= $(shell git describe --tags)
```

## Best Practices Summary

1. **Always declare .PHONY targets**
2. **Use consistent variable naming (UPPER_CASE)**
3. **Document complex rules with comments**
4. **Provide help target for discoverability**
5. **Check dependencies are complete**
6. **Use pattern rules to reduce duplication**
7. **Test portability across make implementations**
8. **Enable parallel execution where safe**
9. **Handle errors gracefully**
10. **Keep lines under 80 characters when possible**

## Related Commands

- `pmat generate makefile` - Generate Makefile from template
- `pmat scaffold` - Create project with proper Makefile
- `pmat analyze complexity` - Analyze code complexity
- `pmat demo` - See Makefile linting in action