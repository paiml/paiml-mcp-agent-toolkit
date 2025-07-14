# Release Notes - v0.28.9

## 🚦 CI/CD Integration & Exit Codes

This release adds comprehensive CI/CD support to all analyze commands, enabling seamless integration with GitHub Actions, GitLab CI, Jenkins, and other CI/CD platforms.

### ✨ New Features

#### Exit Code Support for All Analyze Commands
All `pmat analyze` commands now support the `--fail-on-violation` flag:
- **Exit code 0**: Success - no violations found or within thresholds
- **Exit code 1**: Failure - violations exceed configured thresholds

#### Supported Commands
- `pmat analyze complexity --fail-on-violation`
- `pmat analyze dead-code --fail-on-violation`
- `pmat analyze satd --fail-on-violation`
- `pmat quality-gate --fail-on-violation`

#### Configurable Thresholds
- **Complexity**: 
  - `--max-cyclomatic` (default: 20)
  - `--max-cognitive` (default: 15)
- **Dead Code**: 
  - `--max-percentage` (default: 15.0%)
- **SATD**: 
  - Zero tolerance when using `--fail-on-violation`
  - Use `--strict` for comprehensive detection

### 📚 New Examples

Added comprehensive examples demonstrating CI/CD integration:
- `examples/analyze_complexity.rs` - Complexity analysis with CI/CD modes
- `examples/analyze_dead_code.rs` - Dead code detection with thresholds
- `examples/analyze_satd.rs` - Technical debt detection
- `examples/ci_integration.rs` - Complete CI/CD workflow examples
- `examples/exit_codes.rs` - Exit code behavior reference

Run examples with:
```bash
cargo run --example analyze_complexity
cargo run --example ci_integration
```

### 📖 Documentation Updates

- **README.md**: Added CI/CD Integration section with GitHub Actions example
- **CLI Reference**: Updated all analyze commands with new flags
- **CI/CD Guide**: Comprehensive guide with examples for:
  - GitHub Actions
  - GitLab CI
  - Jenkins Pipeline
  - CircleCI
  - Pre-commit hooks
  - Git hooks

### 💡 Usage Examples

#### GitHub Actions
```yaml
- name: Check Code Quality
  run: |
    pmat analyze complexity \
      --max-cyclomatic 15 \
      --max-cognitive 10 \
      --fail-on-violation
```

#### GitLab CI
```yaml
quality-check:
  script:
    - pmat analyze satd --strict --fail-on-violation
    - pmat analyze dead-code --max-percentage 10.0 --fail-on-violation
```

#### Pre-commit Hook
```bash
#!/bin/bash
pmat analyze complexity --max-cyclomatic 15 --fail-on-violation || exit 1
pmat analyze satd --strict --fail-on-violation || exit 1
```

### 🔧 Technical Details

- Added `fail_on_violation` parameter to all analyze command handlers
- Added `max_percentage` parameter to dead-code analysis
- Updated MCP adapters and CLI tests for new parameters
- Fixed clippy lint warnings in examples

### 🚀 Migration Guide

To use the new CI/CD features:

1. Update your CI/CD workflows to use `--fail-on-violation`
2. Set appropriate thresholds for your project:
   - Start with lenient thresholds and gradually tighten
   - Use JSON output format for parsing in CI scripts
   - Consider different thresholds for different branches

3. Example migration:
   ```bash
   # Old (always exits 0)
   pmat analyze complexity
   
   # New (exits 1 if violations found)
   pmat analyze complexity --fail-on-violation
   
   # With custom thresholds
   pmat analyze complexity --max-cyclomatic 15 --fail-on-violation
   ```

### 🙏 Acknowledgments

This release implements the CI/CD integration feature requested in issue #28, enabling pmat to be used as a quality gate in continuous integration pipelines.

---

For more information, see the [CI/CD Integration Guide](docs/integrations/ci-cd-integration.md) and run `pmat analyze --help` for detailed command options.