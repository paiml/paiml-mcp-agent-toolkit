# Bug Report: Single File Refactor Mode Fails with "No such file or directory" Error

## Summary
The `pmat refactor auto` command fails with "No such file or directory (os error 2)" when attempting to refactor a single file, even though the file exists and is successfully analyzed by PMAT.

## Environment
- **PMAT Version**: 0.28.3
- **Operating System**: Linux 6.8.0-62-lowlatency
- **Date**: 2025-07-04
- **Project**: assetgen (Rust project)

## Steps to Reproduce
1. Navigate to a Rust project with high-complexity files
2. Run complexity analysis to identify files needing refactoring:
   ```bash
   pmat analyze complexity -p . --top-files 10
   ```
3. Attempt to refactor a high-complexity file using single file mode:
   ```bash
   pmat refactor auto --single-file-mode --file src/providers/mcp.rs
   ```

## Expected Behavior
PMAT should:
1. Load the specified file
2. Analyze its complexity
3. Apply automated refactoring to reduce complexity below the threshold
4. Write the refactored file back to disk

## Actual Behavior
PMAT:
1. Successfully starts automated refactoring
2. Correctly identifies single file mode with the target file
3. Successfully analyzes the file ("🎯 Analyzing single file: src/providers/mcp.rs")
4. Reports "📊 Found 0 lint violations"
5. Immediately fails with: `Error: No such file or directory (os error 2)`

## Debug Output
```
2025-07-04T18:59:53.584867Z  INFO pmat: Starting PAIML MCP Agent Toolkit v0.28.3
2025-07-04T18:59:53.584946Z DEBUG pmat: Debug logging enabled
2025-07-04T18:59:53.584987Z DEBUG pmat: Template server initialized
2025-07-04T18:59:53.584999Z DEBUG pmat: Detected CLI mode
2025-07-04T18:59:53.585003Z  INFO pmat: Running in CLI mode
2025-07-04T18:59:53.586112Z DEBUG pmat::cli: CLI arguments parsed
🚀 Starting automated refactoring...
📁 Project: .
📄 Single file mode: src/providers/mcp.rs
🎯 Analyzing single file: src/providers/mcp.rs
📊 Found 0 lint violations
Error: No such file or directory (os error 2)
```

## Additional Information
- The file definitely exists: `ls -la src/providers/mcp.rs` shows the file with proper permissions
- The same error occurs with or without the `--single-file-mode` flag
- The same error occurs when using absolute paths: `/home/noah/src/assetgen/src/providers/mcp.rs`
- The complexity analysis correctly identifies this file as having high complexity:
  ```
  2. `mcp.rs` - Cyclomatic: 175, Cognitive: 119, Functions: 108
  ```

## Attempted Workarounds
1. Using absolute path - same error
2. Removing `--single-file-mode` flag - same error (it automatically detects single file mode)
3. Running with `--debug` flag - provides no additional error context after the failure

## Impact
This bug prevents users from using PMAT's automated refactoring feature on individual high-complexity files, which is a critical workflow for incrementally improving code quality in large projects.

## Possible Root Cause
The error suggests PMAT is looking for a file or directory that doesn't exist, possibly:
- A temporary file or working directory that failed to be created
- A configuration file that's expected but missing
- An issue with the refactoring engine initialization after the analysis phase

## Reproduction Repository
The issue can be reproduced in the assetgen repository:
- Repository: assetgen
- File with high complexity: `src/providers/mcp.rs`
- Complexity metrics: Cyclomatic: 175, Cognitive: 119

## Suggested Fix
1. Add more detailed error logging to identify which file/directory operation is failing
2. Check if the refactoring engine requires additional setup or temporary directories
3. Verify that the transition from analysis phase to refactoring phase properly handles file paths