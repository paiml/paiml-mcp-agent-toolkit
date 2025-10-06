# TICKET-PMAT-6022: Add MCP Error Handling & Result Types

**Sprint:** Sprint 22 - MCP Phase 2
**Priority:** P0 - Critical
**Estimated Effort:** 1-2 hours
**Actual Effort:** 1 hour
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06

## Problem Statement

MCP tool handlers lack consistent error handling. Errors are either:
1. Propagated as Result::Err (breaks MCP protocol)
2. Converted to generic error messages (loses detail)
3. Handled inconsistently across tools

This makes debugging difficult and provides poor user experience for agents.

## Solution

Create a standard `McpOperationResult` type that wraps all MCP tool responses. This provides:
- Consistent success/error format
- Detailed error information
- Error chain for debugging
- Protocol-compliant responses

## Implementation

### McpOperationResult Type

```rust
/// MCP operation result for consistent error handling (TICKET-PMAT-6022)
#[derive(Debug, Serialize, Deserialize)]
pub struct McpOperationResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_details: Option<Vec<String>>,
}

impl McpOperationResult {
    /// Create a success result
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            error_details: None,
        }
    }

    /// Create an error result
    pub fn error(message: String, details: Option<Vec<String>>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            error_details: details,
        }
    }

    /// Create an error result from an anyhow error
    pub fn from_error(err: anyhow::Error) -> Self {
        let error_chain: Vec<String> = err
            .chain()
            .map(|e| e.to_string())
            .collect();

        Self {
            success: false,
            data: None,
            error: Some(err.to_string()),
            error_details: if error_chain.len() > 1 {
                Some(error_chain)
            } else {
                None
            },
        }
    }
}
```

### Handler Wrapper Pattern

```rust
async fn handle_some_tool(&self, params: Value) -> Result<ToolResult> {
    match self.some_tool_internal(params).await {
        Ok(data) => {
            let result = McpOperationResult::success(data);
            Ok(ToolResult::Success(serde_json::to_value(result)?))
        }
        Err(e) => {
            let result = McpOperationResult::from_error(e);
            Ok(ToolResult::Success(serde_json::to_value(result)?))
        }
    }
}

async fn some_tool_internal(&self, params: Value) -> Result<Value> {
    // Business logic that can return errors
    // Errors automatically converted to McpOperationResult
}
```

## Key Features

1. **Consistent Format:** All tools return same structure
2. **Success Flag:** Easy boolean check
3. **Optional Data:** Only present on success
4. **Error Message:** Human-readable error
5. **Error Chain:** Full error context for debugging
6. **Serde Skip:** Omits null fields from JSON

## Response Examples

### Success Response

```json
{
  "success": true,
  "data": {
    "project_name": "my-agent",
    "template": "mcp-server",
    "message": "Agent scaffolded successfully"
  }
}
```

### Error Response

```json
{
  "success": false,
  "error": "Missing required parameter: 'name'",
  "error_details": [
    "Missing required parameter: 'name'",
    "Parameter extraction failed"
  ]
}
```

### File System Error

```json
{
  "success": false,
  "error": "Permission denied: /restricted/path",
  "error_details": [
    "Permission denied: /restricted/path",
    "Failed to create directory",
    "Scaffolding operation failed"
  ]
}
```

## Acceptance Criteria

- [x] Created `McpOperationResult` type
- [x] Implemented `success()` constructor
- [x] Implemented `error()` constructor
- [x] Implemented `from_error()` for anyhow errors
- [x] Error chain extraction working
- [x] All MCP handlers use wrapper pattern
- [x] Serde serialization configured
- [x] Optional fields skip null values
- [x] Code compiles successfully
- [x] Complexity CC <3

## Files Modified

1. **server/src/contracts/mcp_impl.rs**
   - Added `McpOperationResult` type (+65 lines)
   - Updated all MCP handlers to use wrapper pattern
   - Separated handlers into wrapper + internal functions

## Tools Updated

All 5 MCP tools now use consistent error handling:
1. scaffold_agent
2. validate_roadmap
3. health_check
4. generate_tickets
5. scaffold_wasm (still mock, but wrapped)

## Benefits

### For Developers

- **Debugging:** Error chains show full context
- **Testing:** Easy to check success/failure
- **Consistency:** Same pattern everywhere

### For Agents

- **Predictable:** Always same response structure
- **Actionable:** Clear error messages
- **Detailed:** Error chains help diagnose issues

### For Maintainability

- **Single Pattern:** Wrapper + internal functions
- **Type Safety:** Compile-time checks
- **Extensibility:** Easy to add new fields

## Pattern Established

```rust
// Public handler - wraps errors
async fn handle_X(&self, params: Value) -> Result<ToolResult> {
    match self.X_internal(params).await {
        Ok(data) => Ok(ToolResult::Success(
            serde_json::to_value(McpOperationResult::success(data))?
        )),
        Err(e) => Ok(ToolResult::Success(
            serde_json::to_value(McpOperationResult::from_error(e))?
        )),
    }
}

// Private internal - business logic
async fn X_internal(&self, params: Value) -> Result<Value> {
    // Can use ? operator freely
    // Errors automatically converted by wrapper
    Ok(json!({ ... }))
}
```

## Impact

**Before PMAT-6022:**
- Inconsistent error handling
- Lost error context
- Hard to debug failures
- Tools either succeed or crash

**After PMAT-6022:**
- ✅ Consistent error format
- ✅ Full error chains
- ✅ Easy debugging
- ✅ Graceful error handling
- ✅ Agent-friendly responses

## Complexity

- `McpOperationResult` struct: CC=1
- `success()`: CC=1
- `error()`: CC=1
- `from_error()`: CC=2
- Wrapper pattern: CC=1 per tool

**Total:** Well under complexity targets

## Related Tickets

All Sprint 22 tickets use this error handling:
- PMAT-6017: scaffold_agent
- PMAT-6019: validate_roadmap
- PMAT-6020: health_check
- PMAT-6021: generate_tickets

---

*Completed: October 6, 2025*
*Sprint 22 - MCP Phase 2*
