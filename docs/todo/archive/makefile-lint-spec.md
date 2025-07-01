# PAIML MCP Agent Toolkit - Makefile Lint Specification

## Overview

This specification defines a comprehensive linting and formatting system for the PAIML MCP Agent Toolkit project, targeting multiple file types with quality enforcement tailored to AI context generation, code analysis, and MCP protocol integration. The system implements resource-aware processing suitable for CI/CD environments while maintaining strict code quality standards.

## Project Context

The PAIML MCP Agent Toolkit is a **zero-configuration AI context generation system** that analyzes codebases through CLI, MCP, and HTTP interfaces. This lint specification ensures consistent quality across:

- **Core Analysis Engine**: Rust-based parsing and metrics generation
- **MCP Protocol Implementation**: TypeScript/JavaScript integration layer
- **CLI Tools**: Multi-language analysis commands
- **Documentation**: Comprehensive guides and API references
- **CI/CD Infrastructure**: GitHub Actions and build automation

## Objectives

- **Multi-Language Quality**: Consistent standards across Rust, TypeScript, Python, and YAML
- **Resource Efficiency**: Memory-aware processing for CI environments
- **Protocol Compliance**: MCP specification adherence validation
- **Security Focus**: Static analysis for security vulnerabilities
- **Documentation Integrity**: Markdown and configuration validation

## File Type Coverage

### Primary Targets
```bash
# Rust codebase (core engine)
src/**/*.rs                    # Main source code
tests/**/*.rs                  # Unit and integration tests
benches/**/*.rs                # Performance benchmarks

# TypeScript/JavaScript (MCP layer)
scripts/**/*.ts                # Build and utility scripts
scripts/**/*.js                # Legacy scripts
**/*.d.ts                      # Type definitions

# Python (analysis utilities)
*.py                           # Root-level scripts
scripts/**/*.py                # Utility scripts

# YAML configurations
**/*.yml                       # CI/CD workflows
**/*.yaml                      # Configuration files
.github/**/*.yml               # GitHub Actions

# Documentation
docs/**/*.md                   # Documentation
*.md                           # Root documentation
rust-docs/**/*.md              # Generated docs

# Build system
Makefile*                      # Build automation
*.mk                           # Makefile fragments
```

### Exclusions
- `target/` (Rust build artifacts)
- `node_modules/` (Node.js dependencies)
- `.cargo/` (Cargo registry cache)
- Coverage reports and generated artifacts
- Binary files and compressed archives

## Formatting Requirements

### Rust Formatting
```makefile
.PHONY: fmt-rust
fmt-rust:
	@echo "→ Formatting Rust code..."
	@cargo fmt --all --verbose
	@echo "✓ Rust formatting complete"

.PHONY: fmt-check-rust
fmt-check-rust:
	@echo "→ Checking Rust formatting..."
	@cargo fmt --all -- --check
```

**Configuration**: Enhanced `rustfmt.toml` for analysis tools:
```toml
# Code structure
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"

# Import organization
reorder_imports = true
reorder_modules = true
imports_granularity = "Crate"
group_imports = "StdExternalCrate"

# Documentation
normalize_doc_attributes = true
wrap_comments = true
comment_width = 80

# Function formatting
fn_args_layout = "Tall"
where_single_line = true
force_multiline_blocks = true

# Specific to analysis tools
unstable_features = true
format_code_in_doc_comments = true
format_macro_matchers = true
```

### TypeScript/JavaScript Formatting
```makefile
.PHONY: fmt-deno
fmt-deno:
	@echo "→ Formatting TypeScript/JavaScript..."
	@deno fmt scripts/
	@find . -name "*.ts" -not -path "./target/*" -not -path "./node_modules/*" | xargs deno fmt
	@echo "✓ Deno formatting complete"

.PHONY: fmt-check-deno
fmt-check-deno:
	@echo "→ Checking TypeScript formatting..."
	@deno fmt --check scripts/
	@find . -name "*.ts" -not -path "./target/*" -not -path "./node_modules/*" | xargs deno fmt --check
```

**Configuration**: `deno.json` with MCP-specific settings:
```json
{
  "fmt": {
    "options": {
      "lineWidth": 100,
      "indentWidth": 2,
      "semiColons": true,
      "singleQuote": false
    },
    "include": ["scripts/", "docs/"],
    "exclude": ["target/", "coverage/"]
  },
  "lint": {
    "rules": {
      "tags": ["recommended"],
      "include": ["ban-untagged-todo"],
      "exclude": ["no-explicit-any"]
    },
    "include": ["scripts/"],
    "exclude": ["target/", "node_modules/"]
  }
}
```

### Python Formatting
```makefile
.PHONY: fmt-python
fmt-python:
	@echo "→ Formatting Python code..."
	@if command -v black >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | xargs black --line-length 100; \
	else \
		echo "⚠️  black not installed, skipping Python formatting"; \
	fi
	@if command -v isort >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | xargs isort --profile black; \
	else \
		echo "⚠️  isort not installed, skipping import sorting"; \
	fi
	@echo "✓ Python formatting complete"

.PHONY: fmt-check-python
fmt-check-python:
	@echo "→ Checking Python formatting..."
	@if command -v black >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | xargs black --check --line-length 100; \
	fi
	@if command -v isort >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | xargs isort --check-only --profile black; \
	fi
```

### YAML Formatting
```makefile
.PHONY: fmt-yaml
fmt-yaml:
	@echo "→ Formatting YAML files..."
	@if command -v yamlfmt >/dev/null 2>&1; then \
		find . -name "*.yml" -o -name "*.yaml" | \
		grep -v target | grep -v node_modules | \
		xargs yamlfmt -conf .yamlfmt.yml; \
	else \
		echo "⚠️  yamlfmt not installed, skipping YAML formatting"; \
	fi
	@echo "✓ YAML formatting complete"

.PHONY: fmt-check-yaml
fmt-check-yaml:
	@echo "→ Checking YAML formatting..."
	@if command -v yamlfmt >/dev/null 2>&1; then \
		find . -name "*.yml" -o -name "*.yaml" | \
		grep -v target | grep -v node_modules | \
		xargs yamlfmt -dry -conf .yamlfmt.yml; \
	fi
```

**Configuration**: `.yamlfmt.yml` for CI/CD workflows:
```yaml
line_length: 120
indent: 2
include_document_start: false
pad_line_comments: 1
retain_line_breaks: true
scan_folded_as_literal: true
indentless_arrays: false
drop_merge_tag: true
```

### Makefile Formatting
```makefile
.PHONY: fmt-makefile
fmt-makefile:
	@echo "→ Formatting Makefiles..."
	@find . -name "Makefile*" -o -name "*.mk" | while read file; do \
		echo "  Processing $$file..."; \
		# Ensure proper tab indentation for recipes
		sed -i.bak 's/^        /\t/' "$$file" && rm "$$file.bak"; \
		# Normalize .PHONY declarations
		sed -i.bak 's/^\.PHONY: */.PHONY: /' "$$file" && rm "$$file.bak"; \
	done
	@echo "✓ Makefile formatting complete"
```

## Linting Requirements

### Rust Linting (Enhanced for Analysis Tools)
```makefile
.PHONY: lint-rust
lint-rust:
	@echo "→ Running Rust linting..."
	RUST_TEST_THREADS=2 CARGO_BUILD_JOBS=2 cargo clippy \
		--all-targets --all-features -- \
		-D warnings \
		-D clippy::all \
		-D clippy::pedantic \
		-D clippy::nursery \
		-D clippy::cargo \
		-D clippy::suspicious \
		-D clippy::complexity \
		-D clippy::perf \
		-A clippy::missing_docs_in_private_items \
		-A clippy::module_name_repetitions \
		-A clippy::too_many_lines \
		-A clippy::cast_precision_loss \
		-A clippy::similar_names
	@echo "✓ Rust linting complete"
```

**Enhanced Rules for AI/Analysis Context**:
- **Memory Safety**: All unsafe blocks must be documented
- **Error Handling**: Comprehensive `Result` type usage
- **Performance**: Zero-allocation paths for hot code
- **API Design**: Consistent naming for analysis functions
- **Documentation**: Public APIs must have examples

### TypeScript/MCP Protocol Linting
```makefile
.PHONY: lint-deno
lint-deno:
	@echo "→ Running Deno lint..."
	@deno lint scripts/
	@echo "→ Running Deno type check..."
	@deno check scripts/**/*.ts
	@echo "→ Checking MCP protocol compliance..."
	@find scripts/ -name "*.ts" | xargs grep -l "mcp" | while read file; do \
		echo "  Validating MCP types in $$file..."; \
		deno check --lib=dom,es2020 "$$file"; \
	done
	@echo "✓ Deno linting complete"
```

**MCP-Specific Rules**:
- **Protocol Compliance**: Validate MCP message schemas
- **Type Safety**: Strict TypeScript mode for MCP handlers
- **Async Patterns**: Proper Promise handling
- **Error Boundaries**: Comprehensive error handling

### Python Linting
```makefile
.PHONY: lint-python
lint-python:
	@echo "→ Linting Python code..."
	@if command -v flake8 >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | \
		xargs flake8 --max-line-length=100 --extend-ignore=E203,W503; \
	else \
		echo "⚠️  flake8 not installed, skipping Python linting"; \
	fi
	@if command -v mypy >/dev/null 2>&1; then \
		find . -name "*.py" -not -path "./target/*" | xargs mypy --ignore-missing-imports; \
	else \
		echo "⚠️  mypy not installed, skipping type checking"; \
	fi
	@echo "✓ Python linting complete"
```

### YAML/CI Linting
```makefile
.PHONY: lint-yaml
lint-yaml:
	@echo "→ Linting YAML files..."
	@if command -v yamllint >/dev/null 2>&1; then \
		find . -name "*.yml" -o -name "*.yaml" | \
		grep -v target | grep -v node_modules | \
		xargs yamllint -c .yamllint.yml; \
	else \
		echo "⚠️  yamllint not installed, skipping YAML linting"; \
	fi
	@echo "→ Validating GitHub Actions..."
	@if command -v actionlint >/dev/null 2>&1; then \
		actionlint .github/workflows/*.yml; \
	else \
		echo "⚠️  actionlint not installed, skipping workflow validation"; \
	fi
	@echo "✓ YAML linting complete"
```

**Configuration**: `.yamllint.yml` with CI focus:
```yaml
extends: relaxed
rules:
  line-length:
    max: 120
    allow-non-breakable-inline-mappings: true
  comments:
    min-spaces-from-content: 1
  indentation:
    spaces: 2
    indent-sequences: true
  truthy:
    allowed-values: ['true', 'false', 'on', 'off']
    check-keys: false
```

### Markdown Documentation Linting
```makefile
.PHONY: lint-markdown
lint-markdown:
	@echo "→ Linting Markdown documentation..."
	@if command -v markdownlint >/dev/null 2>&1; then \
		find . -name "*.md" -not -path "./target/*" | \
		xargs markdownlint --config .markdownlint.json; \
	else \
		echo "⚠️  markdownlint not installed, skipping Markdown linting"; \
	fi
	@echo "→ Checking documentation links..."
	@find docs/ -name "*.md" | while read file; do \
		grep -o '\[.*\](.*\.md)' "$$file" | sed 's/.*(\(.*\))/\1/' | while read link; do \
			if [ ! -f "docs/$$link" ] && [ ! -f "$$link" ]; then \
				echo "  Broken link in $$file: $$link"; \
			fi; \
		done; \
	done
	@echo "✓ Markdown linting complete"
```

### Makefile Validation
```makefile
.PHONY: lint-makefile
lint-makefile:
	@echo "→ Linting Makefiles..."
	@find . -name "Makefile*" -o -name "*.mk" | while read file; do \
		echo "  Checking $$file..."; \
		# Validate tab indentation
		if grep -n '^        ' "$$file" > /dev/null; then \
			echo "    ERROR: Use tabs for recipe indentation"; \
			grep -n '^        ' "$$file"; \
			exit 1; \
		fi; \
		# Check for .PHONY declarations
		if ! grep -q '\.PHONY:' "$$file"; then \
			echo "    WARNING: No .PHONY declarations found"; \
		fi; \
		# Validate resource constraints
		if grep -q 'RUST_TEST_THREADS\|CARGO_BUILD_JOBS' "$$file"; then \
			echo "    ✓ Resource constraints configured"; \
		fi; \
		# Check for memory management patterns
		if grep -q 'ulimit\|swap' "$$file"; then \
			echo "    ✓ Memory management configured"; \
		fi; \
	done
	@echo "✓ Makefile linting complete"
```

## Composite Targets

### Format All Files
```makefile
.PHONY: fmt
fmt: fmt-rust fmt-deno fmt-python fmt-yaml fmt-makefile
	@echo "🎨 All formatting complete!"

.PHONY: fmt-check
fmt-check: fmt-check-rust fmt-check-deno fmt-check-python fmt-check-yaml
	@echo "✅ All format checks passed!"
```

### Comprehensive Linting
```makefile
.PHONY: lint
lint: lint-rust lint-deno lint-python lint-yaml lint-markdown lint-makefile
	@echo "🔍 All linting complete!"

.PHONY: lint-core
lint-core: lint-rust lint-deno
	@echo "⚡ Core linting complete!"

.PHONY: lint-docs
lint-docs: lint-markdown lint-yaml
	@echo "📚 Documentation linting complete!"
```

### Combined Operations
```makefile
.PHONY: format-and-lint
format-and-lint: fmt lint
	@echo "🚀 Format and lint complete!"

.PHONY: pre-commit
pre-commit: fmt-check lint-core
	@echo "✅ Pre-commit checks passed!"

.PHONY: ci-full
ci-full: check-tools fmt-check lint
	@echo "🎯 CI full validation complete!"
```

## Resource Management & Performance

### Memory-Constrained Execution
```makefile
.PHONY: lint-low-memory
lint-low-memory:
	@echo "→ Running memory-efficient linting..."
	@export RUST_TEST_THREADS=1 && \
	export CARGO_BUILD_JOBS=1 && \
	$(MAKE) lint-rust
	@$(MAKE) lint-deno
	@$(MAKE) lint-python
	@$(MAKE) lint-yaml
	@echo "💾 Low-memory linting complete"

.PHONY: lint-parallel
lint-parallel:
	@echo "→ Running parallel linting..."
	@$(MAKE) -j4 lint-rust lint-deno lint-python lint-yaml
	@echo "⚡ Parallel linting complete"
```

### Incremental Processing
```makefile
.PHONY: lint-changed
lint-changed:
	@echo "→ Linting changed files only..."
	@git diff --name-only HEAD~1 | grep '\.rs$$' | xargs -r cargo clippy --
	@git diff --name-only HEAD~1 | grep -E '\.(ts|js)$$' | xargs -r deno lint
	@git diff --name-only HEAD~1 | grep '\.py$$' | xargs -r flake8
	@git diff --name-only HEAD~1 | grep -E '\.(yml|yaml)$$' | xargs -r yamllint
	@echo "📈 Incremental linting complete"
```

## Tool Installation & Dependencies

### Required Tools Check
```makefile
.PHONY: check-tools
check-tools:
	@echo "→ Checking required tools..."
	@command -v cargo >/dev/null || (echo "❌ cargo not found" && exit 1)
	@command -v rustfmt >/dev/null || (echo "❌ rustfmt not found" && exit 1)
	@command -v deno >/dev/null || (echo "❌ deno not found" && exit 1)
	@echo "✅ Core tools available"
	@echo "→ Checking optional tools..."
	@command -v black >/dev/null || echo "⚠️  black not found (Python formatting)"
	@command -v flake8 >/dev/null || echo "⚠️  flake8 not found (Python linting)"
	@command -v yamllint >/dev/null || echo "⚠️  yamllint not found (YAML linting)"
	@command -v yamlfmt >/dev/null || echo "⚠️  yamlfmt not found (YAML formatting)"
	@command -v actionlint >/dev/null || echo "⚠️  actionlint not found (workflow validation)"
	@command -v markdownlint >/dev/null || echo "⚠️  markdownlint not found (doc linting)"
```

### Tool Installation Helper
```makefile
.PHONY: install-tools
install-tools:
	@echo "→ Installing linting tools..."
	# Rust components
	rustup component add clippy rustfmt
	# Python tools
	pip install black isort flake8 mypy yamllint
	# YAML tools
	go install github.com/google/yamlfmt/cmd/yamlfmt@latest
	# GitHub Actions
	go install github.com/rhymond/actionlint/cmd/actionlint@latest
	# Markdown
	npm install -g markdownlint-cli
	@echo "🛠️  Tool installation complete"
```

## Error Handling & Recovery

### Graceful Degradation
```makefile
.PHONY: lint-safe
lint-safe:
	@echo "→ Running safe linting (continues on errors)..."
	@$(MAKE) lint-rust || echo "⚠️  Rust linting failed"
	@$(MAKE) lint-deno || echo "⚠️  Deno linting failed"
	@$(MAKE) lint-python || echo "⚠️  Python linting failed"
	@$(MAKE) lint-yaml || echo "⚠️  YAML linting failed"
	@$(MAKE) lint-makefile || echo "⚠️  Makefile linting failed"
	@echo "→ Safe linting completed (check warnings above)"
```

### Fix Common Issues
```makefile
.PHONY: fix-auto
fix-auto:
	@echo "→ Auto-fixing common issues..."
	@cargo fix --allow-dirty --allow-staged
	@$(MAKE) fmt
	@echo "🔧 Auto-fix complete"

.PHONY: fix-syntax
fix-syntax:
	@echo "→ Fixing syntax errors with Python scripts..."
	@if [ -f fix_syntax_errors.py ]; then \
		python fix_syntax_errors.py; \
	fi
	@if [ -f fix_braces_smart.py ]; then \
		python fix_braces_smart.py; \
	fi
	@echo "🔧 Syntax fixes applied"
```

## Integration & Metrics

### CI/CD Integration
```makefile
.PHONY: ci-lint
ci-lint: check-tools
	@echo "🚀 Running CI linting pipeline..."
	@$(MAKE) fmt-check
	@$(MAKE) lint-core
	@$(MAKE) lint-docs
	@echo "✅ CI linting passed!"

.PHONY: ci-quality-gate
ci-quality-gate: ci-lint
	@echo "→ Running quality gate analysis..."
	@if command -v pmat >/dev/null 2>&1; then \
		pmat quality-gate --strict; \
	else \
		echo "⚠️  pmat not available for quality gate"; \
	fi
	@echo "🎯 Quality gate validation complete"
```

### Performance Monitoring
```makefile
.PHONY: lint-benchmark
lint-benchmark:
	@echo "→ Benchmarking lint performance..."
	@time $(MAKE) lint-core 2>&1 | tee lint-performance.log
	@echo "📊 Performance metrics saved to lint-performance.log"

.PHONY: lint-memory-profile
lint-memory-profile:
	@echo "→ Profiling memory usage during linting..."
	@/usr/bin/time -v $(MAKE) lint-core 2>&1 | \
		grep -E "(Maximum resident set size|User time|System time)"
```

## Configuration Files

### Project Root Configuration

**`.markdownlint.json`**:
```json
{
  "default": true,
  "MD013": {
    "line_length": 120,
    "code_blocks": false,
    "tables": false
  },
  "MD033": false,
  "MD041": false,
  "MD034": false
}
```

**`.flake8`**:
```ini
[flake8]
max-line-length = 100
extend-ignore = E203, W503, E501
exclude = target/, .cargo/, node_modules/
per-file-ignores = __init__.py:F401
```

## Success Metrics & Quality Gates

### Performance Targets
- **Full lint run**: < 60 seconds (multi-language complexity)
- **Core lint (Rust+TS)**: < 30 seconds
- **Incremental lint**: < 10 seconds for typical changes
- **Memory usage**: < 4GB peak (analysis tool context)

### Quality Standards
- **Zero Rust clippy warnings** on deny-level rules
- **TypeScript strict mode** with zero type errors
- **MCP protocol compliance** validation
- **Documentation completeness** for public APIs
- **CI/CD workflow validation** passes

### Monitoring & Reporting
```makefile
.PHONY: quality-report
quality-report:
	@echo "📊 Generating quality report..."
	@echo "Rust complexity analysis..."
	@pmat analyze complexity --top-files 10 || true
	@echo "Technical debt analysis..."
	@pmat analyze satd --top-files 5 || true
	@echo "Makefile quality..."
	@pmat analyze makefile-lint || true
	@echo "📋 Quality report complete"
```

## Future Enhancements

### Planned Integrations
1. **Semantic Analysis**: Symbol resolution and cross-reference validation
2. **Security Scanning**: `cargo audit` and `npm audit` integration
3. **Dependency Analysis**: License compliance and vulnerability scanning
4. **Performance Regression**: Benchmark comparison against baseline
5. **AI Model Validation**: MCP protocol schema validation

### Advanced Features
1. **Real-time Linting**: Watch mode for development
2. **IDE Integration**: LSP-based linting feedback
3. **Metrics Dashboard**: Quality trends visualization
4. **Custom Rules**: Project-specific linting extensions
5. **Automated Fixes**: ML-powered code improvement suggestions

This specification provides a comprehensive, resource-aware linting system specifically designed for the PAIML MCP Agent Toolkit's multi-language, AI-focused codebase.
