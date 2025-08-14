# Doctest Coverage for PMAT's pmcp SDK Integration

This document outlines the plan to increase doctest coverage for PMAT's use of the pmcp SDK, including both the integration code and examples.

## Overview

PMAT uses the pmcp SDK to provide an experimental high-performance MCP server implementation. This todo list focuses on adding comprehensive doctests to ensure the integration is well-documented and tested.

## Current Status

- pmcp integration is behind the `pmcp-mcp` feature flag
- Basic implementation exists in `src/mcp_pmcp/` directory
- One example exists: `examples/test_pmcp_server.rs`
- No doctests currently exist for the pmcp integration

## Todo Items

### 1. Core Module Documentation (`src/mcp_pmcp/mod.rs`)

- [ ] Add module-level documentation explaining the pmcp integration
- [ ] Include doctest showing how to create and configure a PmcpServer
- [ ] Add example of feature flag usage
- [ ] Document environment variable activation (PMAT_PMCP_MCP=1)

### 2. Server Implementation (`src/mcp_pmcp/server.rs`)

- [ ] Document PmcpServer struct with usage examples
- [ ] Add doctest for `new()` method
- [ ] Add doctest for `run()` method showing stdio server setup
- [ ] Include example of building server with all tools
- [ ] Document error handling patterns

### 3. Tool Handlers Documentation

#### Analyze Handlers (`src/mcp_pmcp/analyze_handlers.rs`)
- [ ] Document each analyze tool struct
- [ ] Add doctests for tool creation
- [ ] Include examples of JSON arguments for each tool:
  - [ ] AnalyzeComplexityTool
  - [ ] AnalyzeSatdTool  
  - [ ] AnalyzeDeadCodeTool
  - [ ] AnalyzeDagTool
  - [ ] AnalyzeDeepContextTool
  - [ ] AnalyzeBigOTool

#### Refactor Handlers (`src/mcp_pmcp/handlers.rs`)
- [ ] Document RefactorStartTool with state machine example
- [ ] Add doctest for RefactorNextIterationTool
- [ ] Document RefactorGetStateTool with state serialization
- [ ] Add RefactorStopTool usage example
- [ ] Include complete refactoring session workflow

#### Quality Gate Handlers (`src/mcp_pmcp/quality_handlers.rs`)
- [ ] Document QualityGateTool with configuration examples
- [ ] Show different quality check combinations
- [ ] Include threshold configuration examples

#### Context Handlers (`src/mcp_pmcp/context_handlers.rs`)
- [ ] Document GenerateContextTool with various options
- [ ] Add GitTool operations examples
- [ ] Include ScaffoldProjectTool usage

### 4. Tool Functions (`src/mcp_pmcp/tool_functions.rs`)

- [ ] Document each placeholder function with expected behavior
- [ ] Add doctests showing input/output format
- [ ] Include examples of success responses
- [ ] Document future implementation plans

### 5. Examples Enhancement

#### Enhance `examples/test_pmcp_server.rs`
- [ ] Add comprehensive comments explaining each step
- [ ] Include example of connecting to the server
- [ ] Show how to send requests and handle responses
- [ ] Add error handling examples

#### Create New Examples
- [ ] `examples/pmcp_client.rs` - Client connecting to pmcp server
- [ ] `examples/pmcp_analyze_workflow.rs` - Complete analysis workflow
- [ ] `examples/pmcp_refactor_session.rs` - Full refactoring session
- [ ] `examples/pmcp_custom_tool.rs` - Adding custom tools

### 6. Integration Tests as Documentation

- [ ] Create `tests/pmcp_integration.rs` with well-documented tests
- [ ] Each test should serve as a usage example
- [ ] Cover all major use cases

### 7. README Examples

- [ ] Update main README.md with pmcp examples
- [ ] Add examples/README.md with pmcp-specific examples
- [ ] Include performance comparison examples

## Implementation Strategy

1. **Phase 1**: Core module and server documentation (items 1-2)
2. **Phase 2**: Handler documentation (item 3)
3. **Phase 3**: Tool functions and examples (items 4-5)
4. **Phase 4**: Integration tests and README updates (items 6-7)

## Doctest Format Guidelines

Each doctest should follow this format:

```rust
/// Brief description of the component
/// 
/// # Examples
/// 
/// ```rust
/// # use pmat::mcp_pmcp::*;
/// # #[cfg(feature = "pmcp-mcp")]
/// # {
/// let server = PmcpServer::new();
/// // Example usage here
/// # }
/// ```
/// 
/// # Errors
/// 
/// Returns an error if [conditions]
```

## Success Criteria

- [ ] All public APIs have at least one doctest
- [ ] `cargo test --doc --features pmcp-mcp` passes
- [ ] `cargo run --example [name] --features pmcp-mcp` works for all examples
- [ ] Documentation clearly explains when to use pmcp vs standard MCP
- [ ] Performance benefits are documented with examples

## Notes

- All doctests must be gated with `#[cfg(feature = "pmcp-mcp")]`
- Examples should demonstrate real-world usage patterns
- Focus on the most common use cases first
- Ensure examples are self-contained and runnable