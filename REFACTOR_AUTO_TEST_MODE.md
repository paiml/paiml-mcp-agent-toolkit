# Refactor Auto Test Mode

## Overview

`pmat refactor auto` now supports test-specific refactoring - a common use case where a test is failing and you need to fix both the test and its dependencies to meet quality standards.

## Usage

### Fix a Specific Test File
```bash
pmat refactor auto --test server/src/tests/ast_e2e.rs
```

This will:
1. Analyze the test file
2. Find all source files the test depends on (via imports)
3. Refactor BOTH the test and its dependencies until:
   - Test passes
   - All files have ≥ 80% coverage
   - All files meet extreme quality standards

### Fix a Specific Test Function
```bash
pmat refactor auto --test server/src/tests/ast_e2e.rs --test-name test_mixed_language_project_context
```

This focuses on a specific test function within the file.

## How It Works

1. **Dependency Analysis**: Parses the test file to find:
   - `use crate::*` imports
   - `use super::*` imports
   - Direct file path references in the test

2. **Target Files**: Creates a list including:
   - The test file itself
   - All source files the test depends on

3. **Refactoring Priority**: In test mode, it processes files in order:
   - Test file first (to understand what's needed)
   - Then dependency files (to fix the actual issues)

4. **Quality Enforcement**: For each file:
   - Fix all lint violations
   - Ensure compilation passes
   - Achieve 80% test coverage
   - Meet extreme quality standards (complexity ≤ 10, zero SATD)

## Example Workflow

```bash
# A test is failing
cargo test test_mixed_language_project_context
# FAILED: Expected more than 30 AST items

# Use refactor auto to fix it
pmat refactor auto --test server/src/tests/ast_e2e.rs --test-name test_mixed_language_project_context

# Output:
# 🧪 Test-specific mode: server/src/tests/ast_e2e.rs
# 🔍 Looking for specific test: test_mixed_language_project_context
# 📦 Found 3 source files related to test
#   📄 server/src/services/ast_python.rs
#   📄 server/src/services/ast_typescript.rs
#   📄 server/src/tests/fixtures/sample.py
# 
# 🧪 Test mode: Processing server/src/tests/ast_e2e.rs
#    1 of 4 target files completed
# ...
```

## Benefits

1. **Focused Refactoring**: Only touches files related to the failing test
2. **Complete Fix**: Ensures both test and source code meet quality standards
3. **80% Coverage Guarantee**: Won't stop until coverage threshold is met
4. **Automated Workflow**: No need to manually identify dependencies

## Integration with CI/CD

```yaml
# When a test fails in CI
- name: Fix failing test
  run: |
    pmat refactor auto \
      --test ${{ env.FAILING_TEST_FILE }} \
      --test-name ${{ env.FAILING_TEST_NAME }} \
      --max-iterations 5
```

This feature makes it easy to maintain high code quality even when tests fail, by automatically refactoring both tests and their dependencies to meet extreme quality standards.