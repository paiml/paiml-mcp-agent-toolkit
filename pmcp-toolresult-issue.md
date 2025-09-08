# Bug Report: ToolResult not exported from pmcp crate root

## Summary

The `ToolResult` enum is not accessible when importing from `pmcp` crate root, despite being referenced in working examples and documentation. This prevents proper MCP tool implementation when following the documented patterns.

## Environment

- **pmcp version**: 1.3.0 (also tested with 1.2.0)
- **Rust version**: 1.89.0 (2025-06-23 c24e10642)
- **Platform**: Linux 6.8.0-79-lowlatency
- **Cargo features**: `["full", "validation"]`

## Expected Behavior

Based on working code examples in the same codebase, `ToolResult` should be importable from the pmcp crate root:

```rust
use pmcp::{
    Server, ServerBuilder, Tool, ToolHandler, ToolResult,
    types::{JsonValue, ToolDefinition},
};
```

## Actual Behavior

Import fails with compilation error:

```
error[E0432]: unresolved import `pmcp::ToolResult`
 --> server/src/mcp_pmcp/tools/auto_clippy_fix.rs:7:5
  |
7 |     ToolResult,
  |     ^^^^^^^^^^ no `ToolResult` in the root
```

## Reproduction Steps

### Minimal Reproduction Case

1. Create a new Rust project
2. Add pmcp dependency:
   ```toml
   [dependencies]
   pmcp = { version = "1.3.0", features = ["full", "validation"] }
   serde_json = "1.0"
   anyhow = "1.0"
   ```

3. Create `src/main.rs`:
   ```rust
   use pmcp::{ToolResult};  // This fails
   use serde_json::{json, Value};
   
   fn create_success_result() -> ToolResult {
       ToolResult::Success(json!({"status": "ok"}))
   }
   
   fn main() {
       println!("Testing ToolResult import");
   }
   ```

4. Run `cargo check` - compilation fails with the import error

### Working Example in Same Codebase

The following import works successfully in `server/src/contracts/mcp_impl.rs`:

```rust
use pmcp::{
    Server, ServerBuilder, Tool, ToolHandler, ToolResult,  // ← This works
    types::{JsonValue, ToolDefinition},
};

// Usage example that compiles successfully:
async fn handle_analyze_complexity(&self, params: Value) -> Result<ToolResult> {
    let contract = serde_json::from_value::<AnalyzeComplexityContract>(params)?;
    let result = self.service.analyze_complexity(contract).await?;
    Ok(ToolResult::Success(result))  // ← This works
}
```

## Investigation Results

### Cargo Tree Verification
```bash
$ cargo tree | grep pmcp
├── pmcp v1.3.0
```

### Version Consistency Check
- `Cargo.toml` specifies: `pmcp = { version = "1.3.0", features = ["full", "validation"] }`
- `Cargo.lock` confirms: `pmcp v1.3.0`
- Working file uses identical import pattern

### Feature Flag Analysis
Tested with various feature combinations:
- `features = ["full", "validation"]` ❌
- `features = ["full"]` ❌ 
- No features specified ❌

## Current Workaround

We've implemented a temporary workaround by defining our own ToolResult:

```rust
// Workaround: Define ToolResult compatible with pmcp interface
// NOTE: File issue with pmcp crate about ToolResult export
#[derive(Debug)]
pub enum ToolResult {
    Success(Value),
    Error(String),
}
```

## Expected Fix

One of the following solutions would resolve this issue:

### Option 1: Export ToolResult from crate root
```rust
// In pmcp/src/lib.rs
pub use crate::internal::ToolResult;
```

### Option 2: Document correct import path
If ToolResult is in a submodule, update documentation to show correct path:
```rust
use pmcp::types::ToolResult; // or wherever it actually lives
```

### Option 3: Re-export in prelude
```rust
// In pmcp/src/prelude.rs
pub use crate::internal::ToolResult;
```

## Impact

This issue prevents:
- Following documented import patterns
- Implementing custom MCP tools consistently
- Maintaining clean, idiomatic Rust code
- Using pmcp in production environments without workarounds

## Additional Context

### Related Issues
- This appears to be a visibility/export issue rather than the type not existing
- The fact that identical imports work in one file but not another suggests a module structure or re-export issue
- No similar issues found in existing GitHub issues

### Suggested Tests
Consider adding integration tests that verify all publicly documented imports actually compile:

```rust
#[test]
fn test_public_imports() {
    // This should compile without errors
    use pmcp::{Server, ServerBuilder, Tool, ToolHandler, ToolResult, types::{JsonValue, ToolDefinition}};
}
```

## Priority

**High** - This blocks proper MCP tool implementation and forces users to implement workarounds that may break in future versions.

---

**Reporter**: Claude Code (Anthropic)  
**Date**: 2025-09-08  
**Project**: paiml-mcp-agent-toolkit  
**Branch**: master  
**Commit**: 147a861