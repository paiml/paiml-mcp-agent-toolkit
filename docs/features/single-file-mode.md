# Single File Mode

Single file mode is a critical feature in pmat that allows targeted quality improvements following Toyota Way principles. This mode enables focused, incremental improvements on individual files without running full project-wide analysis.

## Overview

Single file mode is available in three key commands:
- `pmat refactor auto --single-file-mode --file <path>`
- `pmat lint-hotspot --file <path>`
- `pmat enforce extreme --file <path>`

These commands work together to form a complete quality improvement workflow for individual files.

## Philosophy (Toyota Way)

Single file mode embodies the Toyota Way principles:

1. **Genchi Genbutsu (現地現物)** - Go and see the actual code at the source level
2. **Kaizen (改善)** - Continuous incremental improvement
3. **Jidoka (自働化)** - Build quality in at each step
4. **Hansei (反省)** - Focus on fixing existing issues before adding features

## Usage

### Refactor Auto Single File Mode

The most powerful single file refactoring tool:

```bash
# Basic usage
pmat refactor auto --single-file-mode --file src/lib.rs

# With specific output format
pmat refactor auto --single-file-mode --file src/lib.rs --format json

# Dry run to see what would be changed
pmat refactor auto --single-file-mode --file src/lib.rs --dry-run

# Limit iterations
pmat refactor auto --single-file-mode --file src/lib.rs --max-iterations 5
```

This command will:
1. Run lint-hotspot on the single file to find all violations
2. Generate a refactoring request for AI agents
3. Apply fixes iteratively until quality standards are met

### Lint Hotspot Single File Mode

Analyze a single file for quality violations:

```bash
# Basic analysis
pmat lint-hotspot --file src/complex.rs

# JSON output for parsing
pmat lint-hotspot --file src/complex.rs --format json

# Filter by severity
pmat lint-hotspot --file src/complex.rs --severity error

# Include only specific lint categories
pmat lint-hotspot --file src/complex.rs --category complexity,satd
```

### Enforce Extreme Single File Mode

Enforce extreme quality standards on a single file:

```bash
# Check if file meets standards
pmat enforce extreme --file src/lib.rs

# Auto-fix violations
pmat enforce extreme --file src/lib.rs --fix

# Custom quality thresholds
pmat enforce extreme --file src/lib.rs --max-complexity 5 --min-coverage 90
```

## Quality Standards

Single file mode enforces the same extreme quality standards as project-wide analysis:

- **Cyclomatic Complexity**: Maximum 10 (target 5)
- **Test Coverage**: Minimum 80% per file
- **SATD**: Zero tolerance (no TODO, FIXME, HACK comments)
- **Lint Violations**: All pedantic and nursery clippy lints must pass

## Integration Workflow

The recommended workflow for single file improvements:

1. **Identify hotspot**: Use `pmat lint-hotspot` to find the worst file
2. **Analyze single file**: `pmat lint-hotspot --file <worst-file>`
3. **Auto-refactor**: `pmat refactor auto --single-file-mode --file <worst-file>`
4. **Verify**: `pmat enforce extreme --file <worst-file>`
5. **Repeat**: Move to next worst file

## Benefits

1. **Focused Improvements**: Work on one file at a time
2. **Faster Feedback**: No need to analyze entire project
3. **Incremental Progress**: Small, manageable changes
4. **CI-Friendly**: Can be integrated into pre-commit hooks
5. **Lower Risk**: Changes isolated to single file

## Examples

### Example 1: Fixing a Complex Function

```bash
# File has function with complexity 57
$ pmat lint-hotspot --file src/handlers.rs
🔍 Analyzing single file: src/handlers.rs
📊 Found 15 violations
   - complexity: 1 (function exceeds max complexity)
   - satd: 3 (TODO/FIXME comments)
   - clippy: 11 (various pedantic violations)

# Auto-refactor the file
$ pmat refactor auto --single-file-mode --file src/handlers.rs
📄 Single file mode: src/handlers.rs
🔧 Iteration 1: Found 15 violations
✨ Applied refactoring...
🔧 Iteration 2: Found 3 violations
✨ Applied refactoring...
✅ File meets quality standards!
```

### Example 2: Pre-commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

# Check staged Rust files
for file in $(git diff --cached --name-only | grep '\.rs$'); do
    echo "Checking $file..."
    pmat enforce extreme --file "$file" || {
        echo "❌ $file does not meet quality standards"
        echo "Run: pmat refactor auto --single-file-mode --file $file"
        exit 1
    }
done
```

### Example 3: CI Pipeline Integration

```yaml
# .github/workflows/quality.yml
- name: Check Modified Files
  run: |
    for file in $(git diff --name-only origin/main...HEAD | grep '\.rs$'); do
      pmat enforce extreme --file "$file"
    done
```

## Technical Details

### How It Works

1. **Violation Detection**: Uses `pmat lint-hotspot` internally to get violations
2. **Targeted Analysis**: Only analyzes the specified file, ignoring exclusions
3. **Incremental Fixes**: Applies fixes one category at a time
4. **Verification**: Re-runs analysis after each iteration

### Performance

Single file mode is significantly faster than full project analysis:
- Typical single file: 1-5 seconds
- Full project: 30-120 seconds

### Limitations

- Coverage analysis may require building the entire project
- Some cross-file dependencies may not be fully analyzed
- Import optimization may require manual verification

## Best Practices

1. **Start with Worst Files**: Use `pmat lint-hotspot` to identify files needing most work
2. **Commit After Each File**: Keep changes atomic and reviewable
3. **Run Tests**: Always run tests after refactoring
4. **Review Changes**: AI-generated refactors should be reviewed
5. **Document Progress**: Track which files have been improved

## Troubleshooting

### "Command not found"
Ensure you have the latest version of pmat with single file support.

### "No violations found"
The file may already meet quality standards. Check with `pmat enforce extreme --file <path>`.

### "Refactoring failed"
Some complex refactorings may require manual intervention. Use `--dry-run` to see what changes would be made.

## Future Enhancements

- Parallel single file processing
- Integration with LSP for real-time feedback
- Custom quality profiles per file
- Automatic PR generation for fixes