# Release Notes - v0.28.13

## 🐛 Bug Fixes

### Deep Context Analysis Improvements

This release fixes a critical issue with the deep context analysis command:

#### Issue #33: analyze deep-context shows all complexities as 1.0
- **Fixed**: Deep context now uses proper AST-based complexity analysis
- **Removed**: Eliminated heuristic-based complexity estimation that always returned 1.0
- **Impact**: Deep context analysis now shows accurate complexity values for all functions

### 🏗️ Code Quality Improvements

#### Unified Complexity Analysis
- Deep context now uses the same AST-based complexity analyzer as all other commands
- Follows Rule 6: NEVER Use Simple Heuristics
- Follows Rule 7: NEVER Duplicate Core Logic - ONE implementation for complexity

### 📚 Documentation Updates

#### New Example
- `examples/deep_context_complexity.rs` - Demonstrates accurate complexity analysis in deep context

#### Technical Details
Before (Heuristic):
```
- simple.rs: 1.0 avg complexity
- moderate.rs: 1.0 avg complexity  
- complex.rs: 1.0 avg complexity
```

After (AST-based):
```
- simple.rs: 1.0 avg complexity (3 functions)
- moderate.rs: 4.5 avg complexity (2 functions)
- complex.rs: 15.0 avg complexity (2 functions)
```

### 🧪 Testing

Added comprehensive tests:
- Unit test verifying AST-based analysis returns accurate values
- Property test ensuring complexity values vary based on code structure
- Example demonstrating real-world usage with different complexity levels

### 🔧 Implementation Details

The fix replaces the heuristic `estimate_function_complexity` method with a call to the unified `analyze_project_files` function:

```rust
// Before: Heuristic always returned ~1.0
let complexity = self.estimate_function_complexity(&content, line);

// After: Proper AST analysis
let file_metrics = analyze_project_files(
    project_path,
    Some(toolchain),
    &include_patterns,
    20,  // cyclomatic threshold
    15,  // cognitive threshold
).await?;
```

### 🚀 Usage

```bash
# Analyze deep context with accurate complexity metrics
pmat analyze deep-context --top-files 10

# Output now shows real complexity values:
# 1. `complex_file.rs` - 15.3 avg complexity (40 functions, 12 high complexity)
# 2. `moderate_file.rs` - 6.7 avg complexity (25 functions, 3 high complexity)
# 3. `simple_file.rs` - 2.1 avg complexity (50 functions, 0 high complexity)
```

### 🙏 Acknowledgments

This release addresses issue #33 where deep-context analysis showed all function complexities as 1.0, making it impossible to identify truly complex code that needs refactoring.

---

**Remember**: Always use proper AST-based analysis. No heuristics, no shortcuts, no duplicate implementations.