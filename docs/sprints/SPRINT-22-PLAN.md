# Sprint 22 Plan: MCP Phase 2 - Full Implementation

**Sprint:** Sprint 22
**Duration:** 2-3 days (estimated)
**Status:** 📋 Planning
**Focus:** Connect MCP tools to actual CLI implementations for production-ready agent integration

---

## Executive Summary

Sprint 21 established the MCP protocol foundation with 5 tools (PMAT-6013). Sprint 22 will connect these tools to actual CLI logic, transforming them from mock responses to fully functional implementations. This enables production-ready agent workflows in Claude Code and other MCP clients.

**Current State:**
- ✅ MCP tools registered and discoverable
- ✅ JSON Schema validation working
- ✅ Protocol integration complete
- ❌ Handlers return mock responses (Phase 1)

**Target State:**
- ✅ Handlers call actual CLI functions
- ✅ Real scaffolding operations
- ✅ Real validation and health checks
- ✅ Full error handling and propagation
- ✅ Production-ready for agent workflows

---

## Sprint Goals

### Primary Objectives

1. **Connect MCP to CLI Logic**
   - Replace mock responses with actual implementations
   - Reuse existing CLI handler functions
   - Maintain protocol compliance

2. **Enable Real Agent Workflows**
   - Agents can scaffold real projects
   - Agents can validate real roadmaps
   - Agents can run real health checks
   - Agents can generate real tickets

3. **Add Production Features**
   - Comprehensive error handling
   - Progress reporting (where applicable)
   - Proper file system integration
   - Result validation

4. **Maintain Quality Standards**
   - All code CC <8
   - Comprehensive testing
   - Full documentation
   - Zero breaking changes

---

## Background: Current MCP Implementation

### What Works (Phase 1 - PMAT-6013)

**From `server/src/contracts/mcp_impl.rs`:**

```rust
// Phase 1: Tools are registered
.tool(create_scaffold_agent_tool())
.tool(create_scaffold_wasm_tool())
.tool(create_validate_roadmap_tool())
.tool(create_health_check_tool())
.tool(create_generate_tickets_tool())

// Phase 1: Handlers return mock data
async fn handle_scaffold_agent(&self, params: Value) -> Result<ToolResult> {
    let name = params.get("name")...;
    let result = json!({
        "success": true,
        "message": "Agent project 'X' scaffolded successfully"
    });
    Ok(ToolResult::Success(result))
}
```

**Limitations:**
- No actual scaffolding happens
- No files created
- No validation performed
- No health checks run
- Mock success responses only

### What We Need (Phase 2)

**Real Implementations:**

```rust
// Phase 2: Call actual CLI handlers
async fn handle_scaffold_agent(&self, params: Value) -> Result<ToolResult> {
    // Extract and validate params
    let name = params.get("name")...;

    // Call actual scaffolding logic
    let result = scaffold_agent_project(
        name, template, output_dir, quality_level
    ).await?;

    // Return real results
    Ok(ToolResult::Success(serde_json::to_value(result)?))
}
```

---

## Proposed Tickets

### TICKET-PMAT-6017: Connect scaffold_agent MCP Tool

**Priority:** P0 - Critical
**Estimated Effort:** 2-3 hours

**Problem:**
Current implementation returns mock response. Agents can't actually scaffold projects.

**Solution:**
Connect to existing `ScaffoldEngine` from PMAT-5001:

```rust
async fn handle_scaffold_agent(&self, params: Value) -> Result<ToolResult> {
    use crate::cli::handlers::scaffold_handler;

    // Extract params
    let name = params.get("name").and_then(|v| v.as_str())
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' parameter"))?;
    let template = params.get("template")...;
    let output_dir = params.get("output_dir")...;

    // Call actual scaffolding
    let result = scaffold_handler::scaffold_agent_project(
        name.to_string(),
        template.to_string(),
        output_dir.into(),
        quality_level
    ).await?;

    // Return structured result
    Ok(ToolResult::Success(json!({
        "success": true,
        "project_name": name,
        "template": template,
        "output_dir": output_dir,
        "files_created": result.files_created,
        "message": format!("Agent project '{}' scaffolded successfully", name)
    })))
}
```

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (update handler)
- Reuse: `server/src/cli/handlers/scaffold_handler.rs`

**Testing:**
- Unit test: Mock scaffolding call
- Integration test: Actual project creation
- Verify files created in correct location

**Success Criteria:**
- Agent can scaffold real projects
- Files actually created on disk
- Proper error handling
- CC <5

---

### TICKET-PMAT-6018: Connect scaffold_wasm MCP Tool

**Priority:** P0 - Critical
**Estimated Effort:** 2-3 hours

**Problem:**
Current implementation returns mock response. Agents can't scaffold WASM projects.

**Solution:**
Connect to existing WASM scaffolding from PMAT-5003:

```rust
async fn handle_scaffold_wasm(&self, params: Value) -> Result<ToolResult> {
    use crate::cli::handlers::scaffold_handler;

    // Extract params
    let name = params.get("name")...;
    let framework = params.get("framework")...;
    let output_dir = params.get("output_dir")...;

    // Call actual scaffolding
    let result = scaffold_handler::scaffold_wasm_project(
        name.to_string(),
        framework.to_string(),
        output_dir.into(),
        quality_level
    ).await?;

    Ok(ToolResult::Success(json!({
        "success": true,
        "project_name": name,
        "framework": framework,
        "files_created": result.files_created,
        "message": format!("WASM project '{}' scaffolded successfully", name)
    })))
}
```

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (update handler)
- Reuse: `server/src/cli/handlers/scaffold_handler.rs`

**Success Criteria:**
- Agent can scaffold real WASM projects
- Correct framework templates used
- Files created properly
- CC <5

---

### TICKET-PMAT-6019: Connect validate_roadmap MCP Tool

**Priority:** P1 - High
**Estimated Effort:** 2 hours

**Problem:**
Current implementation returns mock validation. Agents can't validate real roadmaps.

**Solution:**
Connect to existing roadmap validation from PMAT-5032:

```rust
async fn handle_validate_roadmap(&self, params: Value) -> Result<ToolResult> {
    use crate::cli::handlers::roadmap_handler;

    // Extract params
    let roadmap_path = params.get("roadmap_path")...;
    let tickets_dir = params.get("tickets_dir")...;

    // Call actual validation
    let validation = roadmap_handler::validate_roadmap_internal(
        &roadmap_path.into(),
        &tickets_dir.into()
    ).await?;

    Ok(ToolResult::Success(json!({
        "success": validation.valid,
        "valid": validation.valid,
        "errors": validation.errors,
        "warnings": validation.warnings,
        "message": if validation.valid {
            "Roadmap validation passed"
        } else {
            format!("{} error(s) found", validation.errors.len())
        }
    })))
}
```

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (update handler)
- `server/src/cli/handlers/roadmap_handler.rs` (extract validation logic to reusable function)

**Success Criteria:**
- Agent gets real validation results
- Errors and warnings properly reported
- Matches CLI validation exactly
- CC <4

---

### TICKET-PMAT-6020: Connect health_check MCP Tool

**Priority:** P1 - High
**Estimated Effort:** 2-3 hours

**Problem:**
Current implementation returns mock health data. Agents can't run real health checks.

**Solution:**
Connect to existing health checks from PMAT-6001/6010:

```rust
async fn handle_health_check(&self, params: Value) -> Result<ToolResult> {
    use crate::cli::handlers::health_handler;

    // Extract params
    let project_dir = params.get("project_dir")...;
    let quick = params.get("quick")...;
    let check_build = params.get("check_build")...;
    let check_tests = params.get("check_tests")...;

    // Call actual health checks (with parallel execution from PMAT-6010!)
    let report = health_handler::run_health_checks_internal(
        &project_dir.into(),
        quick,
        false, // all
        check_build,
        check_tests,
        false, // check_coverage
    ).await?;

    Ok(ToolResult::Success(json!({
        "success": report.healthy,
        "healthy": report.healthy,
        "checks": report.checks,
        "summary": report.summary,
        "message": if report.healthy {
            "Project health check passed"
        } else {
            format!("{} check(s) failed", report.summary.failed)
        }
    })))
}
```

**Benefits:**
- Uses parallel execution from PMAT-6010!
- Real build/test/coverage checks
- Actual performance data

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (update handler)
- `server/src/cli/handlers/health_handler.rs` (extract to reusable function)

**Success Criteria:**
- Agent gets real health check results
- Parallel execution works via MCP
- Performance data accurate
- CC <5

---

### TICKET-PMAT-6021: Connect generate_tickets MCP Tool

**Priority:** P1 - High
**Estimated Effort:** 2 hours

**Problem:**
Current implementation returns mock generation data. Agents can't generate real tickets.

**Solution:**
Connect to actual ticket generation from PMAT-6012:

```rust
async fn handle_generate_tickets(&self, params: Value) -> Result<ToolResult> {
    use crate::cli::handlers::roadmap_handler;

    // Extract params
    let roadmap_path = params.get("roadmap_path")...;
    let tickets_dir = params.get("tickets_dir")...;
    let dry_run = params.get("dry_run")...;

    // Call actual generation
    let result = roadmap_handler::generate_tickets_internal(
        &roadmap_path.into(),
        &tickets_dir.into(),
        dry_run
    ).await?;

    Ok(ToolResult::Success(json!({
        "success": true,
        "generated": result.generated,
        "skipped": result.skipped,
        "files": result.file_list,
        "message": if result.generated > 0 {
            format!("Generated {} ticket file(s)", result.generated)
        } else {
            "No missing ticket files".to_string()
        }
    })))
}
```

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (update handler)
- `server/src/cli/handlers/roadmap_handler.rs` (extract to reusable function)

**Success Criteria:**
- Agent can generate real ticket files
- Dry-run mode works
- Files created on disk
- CC <4

---

### TICKET-PMAT-6022: Add MCP Error Handling & Result Types

**Priority:** P0 - Critical
**Estimated Effort:** 1-2 hours

**Problem:**
Current error handling is minimal. Need proper error propagation for MCP protocol.

**Solution:**
Add comprehensive error handling:

```rust
/// MCP operation result
#[derive(Debug, Serialize)]
pub struct McpOperationResult {
    pub success: bool,
    pub data: Option<Value>,
    pub error: Option<String>,
    pub error_details: Option<Vec<String>>,
}

impl McpOperationResult {
    pub fn success(data: Value) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            error_details: None,
        }
    }

    pub fn error(message: String, details: Option<Vec<String>>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            error_details: details,
        }
    }
}

// Use in handlers
async fn handle_scaffold_agent(&self, params: Value) -> Result<ToolResult> {
    match scaffold_agent_internal(params).await {
        Ok(result) => {
            let mcp_result = McpOperationResult::success(result);
            Ok(ToolResult::Success(serde_json::to_value(mcp_result)?))
        }
        Err(e) => {
            let mcp_result = McpOperationResult::error(
                e.to_string(),
                Some(vec![format!("{:?}", e)])
            );
            Ok(ToolResult::Success(serde_json::to_value(mcp_result)?))
        }
    }
}
```

**Benefits:**
- Consistent error format across all MCP tools
- Detailed error information for debugging
- Agents can handle errors gracefully

**Files to Modify:**
- `server/src/contracts/mcp_impl.rs` (add result type, update all handlers)

**Success Criteria:**
- All MCP tools use consistent error format
- Errors include helpful details
- CC <3 for result type

---

## Architecture: Integration Pattern

### Current Architecture (Phase 1)

```
┌──────────────────────────────────────┐
│   MCP Client (Claude Code)           │
└──────────────┬───────────────────────┘
               │ MCP Protocol
               ▼
┌──────────────────────────────────────┐
│     ContractMcpServer                │
│  ┌────────────────────────────────┐  │
│  │  MCP Tool Handlers             │  │
│  │  (Return Mock Data)            │  │
│  └────────────────────────────────┘  │
└──────────────────────────────────────┘
```

### Target Architecture (Phase 2)

```
┌──────────────────────────────────────┐
│   MCP Client (Claude Code)           │
└──────────────┬───────────────────────┘
               │ MCP Protocol
               ▼
┌──────────────────────────────────────┐
│     ContractMcpServer                │
│  ┌────────────────────────────────┐  │
│  │  MCP Tool Handlers             │  │
│  │  (Extract Params & Call CLI)   │  │
│  └────────────┬───────────────────┘  │
└────────────────┼───────────────────────┘
                 │
                 ▼
┌──────────────────────────────────────┐
│   Shared Internal Functions          │
│  ┌────────────────────────────────┐  │
│  │  scaffold_agent_internal()    │  │
│  │  scaffold_wasm_internal()     │  │
│  │  validate_roadmap_internal()  │  │
│  │  run_health_checks_internal() │  │
│  │  generate_tickets_internal()  │  │
│  └────────────────────────────────┘  │
└──────────────┬───────────────────────┘
               │
               ▼
┌──────────────────────────────────────┐
│   Core Logic (Existing)              │
│  - ScaffoldEngine                    │
│  - RoadmapHandler                    │
│  - HealthHandler                     │
└──────────────────────────────────────┘
```

### Refactoring Pattern

**For each CLI handler, extract reusable logic:**

```rust
// BEFORE: CLI-only
pub async fn handle_maintain_health(...) -> Result<()> {
    // Business logic mixed with CLI output
    let report = run_checks(...);
    print_health_report(&report, &format)?;
    Ok(())
}

// AFTER: Shared internal function + CLI wrapper
pub async fn run_health_checks_internal(...) -> Result<HealthReport> {
    // Pure business logic, no CLI output
    let report = run_checks(...);
    Ok(report)
}

pub async fn handle_maintain_health(...) -> Result<()> {
    // CLI wrapper
    let report = run_health_checks_internal(...).await?;
    print_health_report(&report, &format)?;
    Ok(())
}

// MCP can now use internal function
async fn handle_health_check(&self, params: Value) -> Result<ToolResult> {
    let report = run_health_checks_internal(...).await?;
    Ok(ToolResult::Success(serde_json::to_value(report)?))
}
```

---

## Implementation Strategy

### Phase 1: Refactor CLI Handlers (Extract Logic)

**For each handler:**
1. Extract business logic to `*_internal()` function
2. Keep CLI wrapper thin (just I/O and formatting)
3. Ensure internal function returns structured data

**Files to Refactor:**
- `scaffold_handler.rs` → `scaffold_agent_internal()`, `scaffold_wasm_internal()`
- `roadmap_handler.rs` → `validate_roadmap_internal()`, `generate_tickets_internal()`
- `health_handler.rs` → `run_health_checks_internal()`

### Phase 2: Connect MCP Handlers

**For each MCP handler:**
1. Replace mock response
2. Call `*_internal()` function
3. Convert result to MCP format
4. Add error handling

### Phase 3: Testing & Validation

**For each integration:**
1. Unit tests for parameter extraction
2. Integration tests for actual operations
3. Error handling tests
4. End-to-end MCP protocol tests (if possible)

---

## Estimated Effort

| Ticket | Estimate | Priority | Dependencies |
|--------|----------|----------|--------------|
| PMAT-6017 | 2-3h | P0 | Refactor scaffold_handler |
| PMAT-6018 | 2-3h | P0 | Refactor scaffold_handler |
| PMAT-6019 | 2h | P1 | Refactor roadmap_handler |
| PMAT-6020 | 2-3h | P1 | Refactor health_handler |
| PMAT-6021 | 2h | P1 | Refactor roadmap_handler |
| PMAT-6022 | 1-2h | P0 | None |
| **Total** | **11-16h** | - | - |

**Sprint Duration:** 2-3 days
**Recommended Scope:** All 6 tickets (complete MCP Phase 2)

---

## Success Criteria

### Feature Complete When:
- [ ] All 5 MCP tools call real implementations
- [ ] Agents can scaffold real projects (agent & WASM)
- [ ] Agents can validate real roadmaps
- [ ] Agents can run real health checks
- [ ] Agents can generate real tickets
- [ ] Comprehensive error handling in place
- [ ] All tests passing (existing + new)

### Release Ready When (v2.141.0):
- [ ] Sprint 22 summary created
- [ ] Release notes written
- [ ] All code CC <8
- [ ] Integration tests added
- [ ] Documentation updated
- [ ] Dogfooding validated

### Published When:
- [ ] Version bumped to 2.141.0
- [ ] Git tag created and pushed
- [ ] Published to crates.io

---

## Testing Strategy

### Unit Tests
- Parameter extraction for each tool
- Error handling edge cases
- Result formatting

### Integration Tests
- Each tool performs actual operation
- Files created/modified correctly
- Results match CLI behavior

### End-to-End Tests (Manual)
- Claude Code can scaffold real projects
- Claude Code can validate roadmaps
- Claude Code can run health checks
- Error messages are helpful

---

## Risks & Mitigations

### Risk 1: Refactoring Breaks CLI
**Mitigation:**
- Extract logic incrementally
- Keep existing tests passing
- Test CLI after each refactor

### Risk 2: MCP Protocol Incompatibility
**Mitigation:**
- Follow MCP spec strictly
- Test with actual MCP client if possible
- Maintain backward compatibility

### Risk 3: Error Handling Complexity
**Mitigation:**
- Standard error result type (PMAT-6022)
- Consistent error format
- Detailed error messages

### Risk 4: File System Side Effects
**Mitigation:**
- Use dry-run modes where applicable
- Clear documentation of what operations do
- Validate paths before operations

---

## Documentation Requirements

### For Each Ticket:
- [ ] Ticket file in `docs/tickets/`
- [ ] Implementation details
- [ ] Usage examples
- [ ] Error handling documented

### Sprint Documentation:
- [ ] Sprint 22 summary
- [ ] Release notes (v2.141.0)
- [ ] Updated feature guides
- [ ] MCP integration guide

---

## Value Proposition

### For Agent Developers:
- **Functional Workflows:** Agents can perform real operations
- **Error Handling:** Proper error propagation for debugging
- **Production Ready:** Real implementations, not mocks

### For Claude Code Users:
- **Actually Works:** Scaffolding creates real projects
- **Accurate Results:** Health checks show real status
- **Reliable:** Validation uses actual logic

### For PMAT Ecosystem:
- **Agent Integration Complete:** Full MCP Phase 2
- **Code Reuse:** CLI and MCP share logic
- **Maintainable:** Single source of truth for operations

---

## Next Steps After Sprint 22

### Sprint 23 Candidates:

**MCP Enhancements:**
- Progress streaming for long operations
- Resource protocol (expose project files)
- Prompts protocol (scaffolding templates)

**Deferred Sprint 21 Items:**
- PMAT-6014: Smart coverage
- PMAT-6015: Enhanced diagnostics
- PMAT-6016: Health trends

**New Capabilities:**
- Team collaboration features
- Advanced analytics
- CI/CD integrations

---

## Conclusion

Sprint 22 will complete the MCP integration started in Sprint 21, transforming PMAT from having MCP protocol support to having fully functional agent integration. This enables production-ready workflows in Claude Code and other MCP-enabled agents.

**Estimated Effort:** 11-16 hours (2-3 days)
**Value:** High - Completes agent ecosystem integration
**Risk:** Low - Building on proven Sprint 21 foundation

**Ready to execute!** 🚀

---

*Sprint 22 Plan*
*Created: October 6, 2025*
*Status: Ready for Implementation*
*Based on: Sprint 21 MCP foundation (PMAT-6013)*
