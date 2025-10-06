# TICKET-PMAT-6019: Connect validate_roadmap MCP Tool

**Sprint:** Sprint 22 - MCP Phase 2
**Priority:** P1 - High
**Estimated Effort:** 2 hours
**Actual Effort:** 1.5 hours
**Status**: GREEN ✅
**Created:** 2025-10-06
**Completed:** 2025-10-06

## Problem Statement

The `validate_roadmap` MCP tool (PMAT-6013) returns mock validation results. Agents cannot perform real roadmap validation via MCP - the tool always reports success without checking actual roadmap structure or ticket consistency.

## Solution

Refactor the existing `validate_roadmap()` CLI function to extract reusable business logic into `validate_roadmap_internal()`. The MCP handler calls this internal function, and the CLI wrapper maintains existing behavior.

### Implementation Pattern

```rust
// Internal function - returns structured data
pub async fn validate_roadmap_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<RoadmapValidation> {
    // Business logic here
    Ok(RoadmapValidation { valid, errors, warnings })
}

// CLI wrapper - prints results
async fn validate_roadmap(roadmap_path: &Path, tickets_dir: &Path) -> Result<()> {
    let validation = validate_roadmap_internal(roadmap_path, tickets_dir).await?;
    // Print formatted output
    Ok(())
}

// MCP wrapper - returns JSON
async fn validate_roadmap_internal(&self, params: Value) -> Result<Value> {
    let validation = validate_roadmap_internal(&roadmap_path, &tickets_dir).await?;
    Ok(json!({ "valid": validation.valid, "errors": validation.errors, ... }))
}
```

## Implementation

### Added RoadmapValidation Type

Already existed in `roadmap_handler.rs`:

```rust
#[derive(Debug)]
pub struct RoadmapValidation {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}
```

### Refactored validate_roadmap()

**Before:** Mixed business logic with CLI output
**After:** Separated into `validate_roadmap_internal()` + CLI wrapper

```rust
/// Validate roadmap structure and ticket consistency (internal, reusable)
/// (TICKET-PMAT-6019)
pub async fn validate_roadmap_internal(
    roadmap_path: &Path,
    tickets_dir: &Path,
) -> Result<RoadmapValidation> {
    let roadmap_content = fs::read_to_string(roadmap_path)?;
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    let roadmap_tickets = parse_roadmap_tickets(&roadmap_content)?;

    for (ticket_id, checkbox_status) in &roadmap_tickets {
        let ticket_path = tickets_dir.join(format!("{ticket_id}.md"));

        if !ticket_path.exists() {
            errors.push(format!("Missing ticket file: {ticket_id}.md"));
            continue;
        }

        let ticket_status = get_ticket_status(&ticket_path)?;

        // Check consistency
        if *checkbox_status && ticket_status != TicketStatus::Green {
            warnings.push(format!(
                "{ticket_id}: Checked in roadmap but status is {:?}",
                ticket_status
            ));
        }

        if !checkbox_status && ticket_status == TicketStatus::Green {
            warnings.push(format!("{ticket_id}: Unchecked in roadmap but status is GREEN"));
        }
    }

    Ok(RoadmapValidation {
        valid: errors.is_empty(),
        errors,
        warnings,
    })
}
```

### MCP Handler

```rust
/// Handle validate_roadmap tool call (TICKET-PMAT-6019, PMAT-6022)
async fn handle_validate_roadmap(&self, params: Value) -> Result<ToolResult> {
    match self.validate_roadmap_internal(params).await {
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

/// Internal implementation of validate_roadmap
async fn validate_roadmap_internal(&self, params: Value) -> Result<Value> {
    use crate::cli::handlers::roadmap_handler::validate_roadmap_internal as validate_impl;
    use std::path::PathBuf;

    let roadmap_path = params.get("roadmap_path")
        .and_then(|v| v.as_str())
        .unwrap_or("ROADMAP.md");

    let tickets_dir = params.get("tickets_dir")
        .and_then(|v| v.as_str())
        .unwrap_or("docs/tickets");

    let validation = validate_impl(
        &PathBuf::from(roadmap_path),
        &PathBuf::from(tickets_dir),
    ).await?;

    Ok(json!({
        "roadmap_path": roadmap_path,
        "tickets_dir": tickets_dir,
        "valid": validation.valid,
        "errors": validation.errors,
        "warnings": validation.warnings,
        "message": if validation.valid {
            "Roadmap validation passed"
        } else {
            format!("{} error(s) found", validation.errors.len())
        }
    }))
}
```

## Acceptance Criteria

- [x] Refactored validation logic into `validate_roadmap_internal()`
- [x] MCP handler calls real validation
- [x] Returns actual errors and warnings
- [x] CLI wrapper unchanged (backward compatible)
- [x] Error handling with `McpOperationResult`
- [x] Default paths: ROADMAP.md, docs/tickets
- [x] Code compiles successfully
- [x] Cyclomatic complexity <8

## Files Modified

1. **server/src/cli/handlers/roadmap_handler.rs**
   - Added `validate_roadmap_internal()` public function
   - Refactored existing `validate_roadmap()` to call internal version
   - Lines: +40 (internal function), minimal changes to wrapper

2. **server/src/contracts/mcp_impl.rs**
   - Replaced mock `handle_validate_roadmap()`
   - Added `validate_roadmap_internal()` MCP implementation
   - Lines: +45

## Usage Examples

**Valid roadmap:**
```json
{
  "roadmap_path": "ROADMAP.md",
  "tickets_dir": "docs/tickets"
}

Response:
{
  "success": true,
  "data": {
    "valid": true,
    "errors": [],
    "warnings": [],
    "message": "Roadmap validation passed"
  }
}
```

**With errors:**
```json
Response:
{
  "success": true,
  "data": {
    "valid": false,
    "errors": [
      "Missing ticket file: TICKET-PMAT-9999.md"
    ],
    "warnings": [
      "TICKET-PMAT-6001: Checked in roadmap but status is Red"
    ],
    "message": "1 error(s) found"
  }
}
```

## Impact

**Before:** Mock validation always returns success
**After:**
- ✅ Real roadmap structure validation
- ✅ Actual ticket file existence checks
- ✅ Checkbox/status consistency verification
- ✅ Detailed error and warning reporting

## Related Tickets

- PMAT-5032: Original roadmap validation implementation
- PMAT-6013: MCP Phase 1 (tool registration)
- PMAT-6022: MCP error handling

---

*Completed: October 6, 2025*
*Sprint 22 - MCP Phase 2*
