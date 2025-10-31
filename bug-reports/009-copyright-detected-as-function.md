# Bug Report: "Copyright" Incorrectly Detected as Function Name

**Date**: 2025-10-31
**Reporter**: User feedback
**Severity**: Medium
**Component**: Function detection - AST parsing

## Description

When running `pmat context` on C++ projects (tested on CPython and Ceph), the analyzer incorrectly detects "Copyright" as a function name. This appears to be a false positive from parsing copyright headers in source files.

## Steps to Reproduce

```bash
cd /path/to/cpython  # or any C/C++ project with copyright headers
pmat context
```

## Actual Output

```markdown
### ./tools/rbd_wnbd/wnbd_wmi.cc
- **Function**: `Copyright` [complexity: 3] [cognitive: 2] [big-o: O(n)] [provability: 43%] [satd: 0] [churn: low(1)] [tdg: 2.5]
```

## Example Copyright Header Format

Likely matching patterns like:

```cpp
// Copyright (c) 2024 Some Organization
// Copyright 2024 Contributors
/* Copyright (c) 2024 */
```

## Expected Behavior

Copyright headers should not be detected as functions. Only actual function definitions should be listed:

```cpp
// Copyright (c) 2024 - should be IGNORED

void initialize_wmi() {  // This should be detected
    // ...
}

int process_data(int input) {  // This should be detected
    return input * 2;
}
```

## Analysis

Possible causes:

1. **Regex Pattern Too Broad**: C/C++ function detection regex matches "Copyright" text
2. **Comment Handling**: Parser not skipping comments before function detection
3. **Tree-sitter Issue**: AST parser incorrectly identifying copyright text as identifier
4. **Pattern Matching**: "Copyright (...)" structure resembles function call syntax

## Impact

- Pollutes function list with false positives
- Reduces trust in analysis accuracy
- Makes it harder to find actual functions in reports
- Skews complexity metrics and function counts

## Files to Investigate

- `server/src/services/languages/c.rs` - C language analyzer
- `server/src/services/languages/cpp.rs` - C++ language analyzer
- `server/src/services/simple_deep_context.rs` - Function detection logic
- Tree-sitter query patterns for C/C++

## Suggested Fix

1. **Pre-filter comments**: Strip comments before AST parsing
2. **Validate function patterns**: Ensure detected "functions" have actual definitions
3. **Exclude copyright patterns**: Explicitly filter out copyright-related matches

```rust
// Example fix
fn is_valid_function_name(name: &str) -> bool {
    // Exclude common header keywords
    let excluded = ["Copyright", "License", "Author", "SPDX"];
    !excluded.contains(&name)
}
```

4. **Tree-sitter query refinement**: Use more specific query patterns

```scheme
; Only match actual function definitions
(function_definition
  declarator: (function_declarator
    declarator: (identifier) @function.name))
```

## Test Case

```rust
#[test]
fn test_copyright_not_detected_as_function() {
    let source = r#"
        // Copyright (c) 2024 Test
        void real_function() {
            // actual code
        }
    "#;

    let functions = analyze_cpp_functions(source);
    assert_eq!(functions.len(), 1);
    assert_eq!(functions[0].name, "real_function");
    assert!(!functions.iter().any(|f| f.name == "Copyright"));
}
```
