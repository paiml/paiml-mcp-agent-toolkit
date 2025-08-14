# pmcp 1.0 Rust MCP SDK Integration Plan

## Overview

This document outlines the integration of the pmcp 1.0 Rust MCP SDK into pmat. Each task is designed to be implemented independently with comprehensive testing (property tests, doctests, unit tests, and examples) while maintaining pmat quality gates and zero SATD.

## Phase 1: Core Transport Layer Migration

### Task 1.1: Create pmcp Transport Abstraction Layer
**File**: `server/src/transport/mod.rs`
**Dependencies**: pmcp = "1.0"
**Deliverables**:
- [ ] Implement `TransportAdapter` trait mapping pmcp Transport to pmat needs
- [ ] Add property tests for message serialization/deserialization invariants
- [ ] Create doctests demonstrating transport initialization
- [ ] Add unit tests for connection lifecycle (connect, send, receive, close)
- [ ] Create `examples/transport_demo.rs` showing stdio and mock transports
- [ ] Run `pmat quality-gate --file server/src/transport/mod.rs`

### Task 1.2: Migrate StdioTransport to pmcp
**File**: `server/src/transport/stdio.rs`
**Deliverables**:
- [ ] Replace custom stdio implementation with `pmcp::transport::StdioTransport`
- [ ] Add property tests for frame boundary preservation
- [ ] Create doctests for stdio message exchange patterns
- [ ] Add unit tests for error handling (broken pipe, EOF, invalid frames)
- [ ] Create `examples/stdio_server.rs` demonstrating pmcp stdio usage
- [ ] Verify with `make test-fast && make lint`

### Task 1.3: Add WebSocket Transport Support
**File**: `server/src/transport/websocket.rs`
**Deliverables**:
- [ ] Implement WebSocket transport using `pmcp::transport::WebSocketTransport`
- [ ] Add property tests for WebSocket frame fragmentation
- [ ] Create doctests for WebSocket connection upgrade
- [ ] Add unit tests for connection drops and reconnection
- [ ] Create `examples/websocket_server.rs` with browser client demo
- [ ] Run full test suite: `make test`

### Task 1.4: Add HTTP/SSE Transport Support
**File**: `server/src/transport/http_sse.rs`
**Deliverables**:
- [ ] Implement HTTP Server-Sent Events transport
- [ ] Add property tests for SSE event stream formatting
- [ ] Create doctests for HTTP request/response patterns
- [ ] Add unit tests for connection keep-alive and timeouts
- [ ] Create `examples/http_sse_server.rs` with curl client examples
- [ ] Verify zero SATD: `pmat analyze satd`

## Phase 2: Handler Trait Migration

### Task 2.1: Migrate ToolHandler Implementation
**File**: `server/src/handlers/tool_handler.rs`
**Deliverables**:
- [ ] Implement `pmcp::ToolHandler` for all existing tools
- [ ] Add `RequestHandlerExtra` support for cancellation
- [ ] Add property tests for tool argument validation
- [ ] Create doctests for each tool handler method
- [ ] Add unit tests for error propagation and cancellation
- [ ] Create `examples/tool_handlers.rs` demonstrating all tools
- [ ] Run `pmat quality-gate --file server/src/handlers/tool_handler.rs`

### Task 2.2: Add ResourceHandler Support
**File**: `server/src/handlers/resource_handler.rs`
**Deliverables**:
- [ ] Implement `pmcp::ResourceHandler` trait
- [ ] Add file system resource provider
- [ ] Add Git repository resource provider
- [ ] Add property tests for URI parsing and validation
- [ ] Create doctests for resource listing and reading
- [ ] Add unit tests for permission checks and access control
- [ ] Create `examples/resource_providers.rs` with file and git resources
- [ ] Verify complexity: `pmat analyze complexity --file server/src/handlers/resource_handler.rs`

### Task 2.3: Add PromptHandler Support
**File**: `server/src/handlers/prompt_handler.rs`
**Deliverables**:
- [ ] Implement `pmcp::PromptHandler` trait
- [ ] Add code review prompt templates
- [ ] Add refactoring guidance prompts
- [ ] Add property tests for prompt parameter substitution
- [ ] Create doctests for prompt generation
- [ ] Add unit tests for prompt validation
- [ ] Create `examples/prompt_templates.rs` with all prompt types
- [ ] Run doctests: `make test-doc`

### Task 2.4: Add SamplingHandler for LLM Support
**File**: `server/src/handlers/sampling_handler.rs`
**Deliverables**:
- [ ] Implement `pmcp::SamplingHandler` trait
- [ ] Add OpenAI provider integration
- [ ] Add Anthropic provider integration
- [ ] Add property tests for message format conversion
- [ ] Create doctests for sampling parameters
- [ ] Add unit tests for rate limiting and retries
- [ ] Create `examples/llm_sampling.rs` with mock provider
- [ ] Verify all tests: `make test`

## Phase 3: Server Builder Pattern Migration

### Task 3.1: Implement pmcp Server Builder
**File**: `server/src/builder/mod.rs`
**Deliverables**:
- [ ] Create `PmatServerBuilder` using pmcp builder pattern
- [ ] Add type-safe configuration methods
- [ ] Add property tests for builder state transitions
- [ ] Create doctests for builder pattern usage
- [ ] Add unit tests for invalid configurations
- [ ] Create `examples/server_builder.rs` with various configurations
- [ ] Run quality gate: `pmat quality-gate --file server/src/builder/mod.rs`

### Task 3.2: Migrate Server Initialization
**File**: `server/src/server.rs`
**Deliverables**:
- [ ] Replace current server with pmcp `Server` implementation
- [ ] Integrate with new builder pattern
- [ ] Add property tests for server lifecycle
- [ ] Create doctests for server start/stop
- [ ] Add unit tests for concurrent request handling
- [ ] Update `examples/mcp_server.rs` to use new server
- [ ] Verify no lint issues: `make lint`

### Task 3.3: Add Middleware Support
**File**: `server/src/middleware/mod.rs`
**Deliverables**:
- [ ] Implement authentication middleware
- [ ] Implement logging middleware
- [ ] Implement rate limiting middleware
- [ ] Add property tests for middleware chain ordering
- [ ] Create doctests for middleware composition
- [ ] Add unit tests for middleware error handling
- [ ] Create `examples/middleware_chain.rs` with all middleware types
- [ ] Run property tests: `make test-property`

## Phase 4: Request Context and Cancellation

### Task 4.1: Implement Request Cancellation
**File**: `server/src/context/cancellation.rs`
**Deliverables**:
- [ ] Add cancellation token support to all handlers
- [ ] Implement graceful shutdown for long operations
- [ ] Add property tests for cancellation propagation
- [ ] Create doctests for cancellation patterns
- [ ] Add unit tests for partial work rollback
- [ ] Create `examples/cancellation_demo.rs` with timeout scenarios
- [ ] Verify complexity: `pmat analyze complexity --file server/src/context/cancellation.rs`

### Task 4.2: Add Progress Notifications
**File**: `server/src/context/progress.rs`
**Deliverables**:
- [ ] Implement progress tracking for analysis operations
- [ ] Add progress callbacks to refactoring operations
- [ ] Add property tests for progress percentage calculations
- [ ] Create doctests for progress notification API
- [ ] Add unit tests for progress aggregation
- [ ] Create `examples/progress_tracking.rs` with real-time updates
- [ ] Check for SATD: `pmat analyze satd`

### Task 4.3: Add Request Metadata Tracking
**File**: `server/src/context/metadata.rs`
**Deliverables**:
- [ ] Implement request ID generation and tracking
- [ ] Add request timing and performance metrics
- [ ] Add property tests for request ID uniqueness
- [ ] Create doctests for metadata access patterns
- [ ] Add unit tests for metadata propagation
- [ ] Create `examples/request_tracking.rs` with correlation demos
- [ ] Run full test suite: `make test`

## Phase 5: Testing Infrastructure

### Task 5.1: Add Property-Based Protocol Tests
**File**: `server/src/tests/protocol_properties.rs`
**Deliverables**:
- [ ] Create property tests for all MCP message types
- [ ] Add invariant tests for protocol state machine
- [ ] Add roundtrip tests for serialization
- [ ] Create doctests explaining property strategies
- [ ] Add shrinking tests for minimal failing cases
- [ ] Create `examples/property_testing.rs` with proptest patterns
- [ ] Verify all pass: `make test-property`

### Task 5.2: Add Mock Transport for Testing
**File**: `server/src/transport/mock.rs`
**Deliverables**:
- [ ] Implement deterministic mock transport
- [ ] Add failure injection capabilities
- [ ] Add property tests for mock behavior
- [ ] Create doctests for mock configuration
- [ ] Add unit tests for various failure modes
- [ ] Create `examples/mock_transport.rs` with test scenarios
- [ ] Run quality gate: `pmat quality-gate --file server/src/transport/mock.rs`

### Task 5.3: Add Integration Test Suite
**File**: `server/tests/pmcp_integration.rs`
**Deliverables**:
- [ ] Create end-to-end MCP protocol tests
- [ ] Add multi-transport integration tests
- [ ] Add concurrent client tests
- [ ] Create doctests for test utilities
- [ ] Add performance benchmarks
- [ ] Create `examples/integration_tests.rs` with test patterns
- [ ] Verify all tests pass: `make test`

## Phase 6: Macro-Based Code Generation

### Task 6.1: Implement Tool Definition Macros
**File**: `server/src/macros/tool_macro.rs`
**Deliverables**:
- [ ] Create `#[tool]` attribute macro for tool handlers
- [ ] Add automatic argument parsing and validation
- [ ] Add property tests for macro expansion
- [ ] Create doctests for macro usage
- [ ] Add unit tests for error messages
- [ ] Create `examples/tool_macros.rs` with various tool types
- [ ] Check complexity: `pmat analyze complexity --file server/src/macros/tool_macro.rs`

### Task 6.2: Implement Resource Definition Macros
**File**: `server/src/macros/resource_macro.rs`
**Deliverables**:
- [ ] Create `#[resource]` attribute macro for resources
- [ ] Add automatic URI pattern matching
- [ ] Add property tests for URI template expansion
- [ ] Create doctests for resource macros
- [ ] Add unit tests for pattern conflicts
- [ ] Create `examples/resource_macros.rs` with URI patterns
- [ ] Verify no SATD: `pmat analyze satd`

## Phase 7: Performance Optimization

### Task 7.1: Add Connection Pooling
**File**: `server/src/pool/connection_pool.rs`
**Deliverables**:
- [ ] Implement connection pool for transports
- [ ] Add automatic reconnection logic
- [ ] Add property tests for pool sizing
- [ ] Create doctests for pool configuration
- [ ] Add unit tests for pool exhaustion
- [ ] Create `examples/connection_pooling.rs` with load testing
- [ ] Run benchmarks and verify performance

### Task 7.2: Add Request Batching
**File**: `server/src/batch/request_batcher.rs`
**Deliverables**:
- [ ] Implement request batching for efficiency
- [ ] Add automatic batch sizing
- [ ] Add property tests for batch ordering
- [ ] Create doctests for batching API
- [ ] Add unit tests for batch timeouts
- [ ] Create `examples/request_batching.rs` with batch scenarios
- [ ] Verify quality: `pmat quality-gate --file server/src/batch/request_batcher.rs`

## Phase 8: Documentation and Examples

### Task 8.1: Create Comprehensive Examples
**File**: `examples/pmcp_showcase.rs`
**Deliverables**:
- [ ] Create full-featured MCP server example
- [ ] Add all transport types
- [ ] Add all handler types
- [ ] Add middleware and authentication
- [ ] Add progress and cancellation
- [ ] Ensure example compiles and runs
- [ ] Verify with `cargo run --example pmcp_showcase`

### Task 8.2: Add Migration Guide
**File**: `docs/pmcp_migration_guide.md`
**Deliverables**:
- [ ] Document breaking changes
- [ ] Provide migration examples
- [ ] Add troubleshooting section
- [ ] Include performance comparisons
- [ ] Add FAQ section
- [ ] Verify all code examples compile

## Success Criteria

Each task must meet the following criteria before being marked complete:

1. **Zero SATD**: No TODO, FIXME, or stub implementations
2. **Quality Gate Pass**: `pmat quality-gate --file <file>` passes
3. **All Tests Pass**: Property tests, doctests, unit tests all passing
4. **Example Compiles**: Associated example runs successfully
5. **Lint Clean**: `make lint` shows no warnings
6. **Complexity Check**: Functions ≤20 complexity score
7. **Documentation**: All public APIs have doctests

## Testing Commands for Each Task

```bash
# After implementing each task, run:
make lint                                    # No warnings allowed
make test-fast                              # Unit and integration tests
make test-doc                               # All doctests must pass
make test-property                          # Property tests must pass
cargo run --example <example_name>         # Example must compile and run
pmat quality-gate --file <implemented_file> # Must pass quality gate
pmat analyze satd                          # Must show 0 SATD comments
pmat analyze complexity --file <file>      # No function >20 complexity
```

## Implementation Order

The phases should be implemented in order, but tasks within each phase can be parallelized:

1. **Phase 1** (Transport): Foundation for all communication
2. **Phase 2** (Handlers): Core functionality migration
3. **Phase 3** (Server): New architecture adoption
4. **Phase 4** (Context): Enhanced capabilities
5. **Phase 5** (Testing): Comprehensive test coverage
6. **Phase 6** (Macros): Developer experience improvements
7. **Phase 7** (Performance): Production optimizations
8. **Phase 8** (Documentation): User guidance

## Estimated Timeline

- Phase 1: 2-3 days (4 tasks)
- Phase 2: 3-4 days (4 tasks)
- Phase 3: 2-3 days (3 tasks)
- Phase 4: 2-3 days (3 tasks)
- Phase 5: 2-3 days (3 tasks)
- Phase 6: 1-2 days (2 tasks)
- Phase 7: 1-2 days (2 tasks)
- Phase 8: 1 day (2 tasks)

**Total**: 14-23 days for complete integration

## Risk Mitigation

1. **Breaking Changes**: All changes must be backward compatible or provide migration path
2. **Performance Regression**: Benchmark before and after each phase
3. **Test Coverage**: Never merge without 100% test passage
4. **Complexity Creep**: Run quality gate after every task
5. **Integration Issues**: Use feature flags for gradual rollout