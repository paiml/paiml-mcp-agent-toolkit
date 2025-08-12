# Unified pmcp MCP Server Implementation

## Problem Statement

The current codebase violates the DRY principle and the "ONE implementation" rule by having THREE separate MCP server implementations:

1. **Standard MCP Server** (`run_mcp_server` in lib.rs) - Handles template/scaffold tools
2. **Refactor MCP Server** (`mcp_server::McpServer`) - Handles refactoring tools  
3. **pmcp-based Server** (`mcp_pmcp::PmcpServer`) - High-performance implementation with all tools

This creates:
- Code duplication and maintenance burden
- Confusion about which server to use
- Inconsistent feature availability (quality_proxy only in pmcp)
- Multiple code paths to test and maintain

## Solution: One Unified pmcp Implementation

Consolidate everything into a single pmcp-based MCP server that:
- Uses the latest pmcp SDK from crates.io (currently 1.0.0)
- Provides ALL tools in one place
- Works with the new quality_proxy feature
- Has comprehensive testing (unit, doc, property, examples)
- Maintains backward compatibility

## Detailed Implementation Tasks

### Phase 1: Analysis and Planning

#### Task 1.1: Inventory Current Tools
- [ ] List all tools in standard MCP server (run_mcp_server)
- [ ] List all tools in refactor MCP server (mcp_server::McpServer)
- [ ] List all tools in pmcp server (mcp_pmcp::PmcpServer)
- [ ] Identify any missing tools in pmcp implementation
- [ ] Document tool dependencies and shared services

#### Task 1.2: Analyze Handler Differences
- [ ] Compare handler implementations between servers
- [ ] Identify any functionality unique to each server
- [ ] Document parameter differences for same tools
- [ ] Check response format compatibility

#### Task 1.3: Review pmcp SDK Capabilities
- [ ] Check latest pmcp version on crates.io
- [ ] Verify all required features are available
- [ ] Review pmcp transport support (stdio, websocket, HTTP/SSE)
- [ ] Confirm error handling and logging capabilities

### Phase 2: Unification Implementation

#### Task 2.1: Migrate Missing Tools to pmcp
- [ ] Port template tools (generate_template, list_templates, etc.) to pmcp handlers
- [ ] Port scaffold_project tool to pmcp handler
- [ ] Port search_templates tool to pmcp handler
- [ ] Port validate_template tool to pmcp handler
- [ ] Ensure all tools use consistent parameter validation

#### Task 2.2: Consolidate Handler Logic
- [ ] Create unified handler module for pmcp
- [ ] Move all handler implementations to pmcp format
- [ ] Ensure quality_proxy is properly integrated
- [ ] Remove duplicate handler code
- [ ] Update all service dependencies

#### Task 2.3: Remove Legacy Implementations
- [ ] Delete run_mcp_server function from lib.rs
- [ ] Remove mcp_server module entirely
- [ ] Clean up old MCP models if not needed
- [ ] Update all imports and references
- [ ] Remove environment variable switches (PMAT_PMCP_MCP, PMAT_REFACTOR_MCP)

#### Task 2.4: Update Binary Entry Point
- [ ] Modify bin/pmat.rs to always use pmcp server
- [ ] Remove execution mode detection for MCP variants
- [ ] Simplify MCP server initialization
- [ ] Ensure proper error handling
- [ ] Add startup diagnostics

### Phase 3: Quality Integration

#### Task 3.1: Verify Quality Proxy Integration
- [ ] Ensure quality_proxy tool is registered in unified server
- [ ] Test all quality_proxy modes (Strict, Advisory, AutoFix)
- [ ] Verify quality_proxy works with all operations (Write, Edit, Append)
- [ ] Check quality gate integration
- [ ] Test refactoring engine integration

#### Task 3.2: Add Quality Gates to All Tools
- [ ] Add quality checks to template generation
- [ ] Add quality checks to scaffold operations
- [ ] Ensure consistent quality enforcement
- [ ] Add configuration options for quality levels
- [ ] Document quality requirements

### Phase 4: Testing

#### Task 4.1: Unit Tests
- [ ] Write unit tests for unified pmcp server initialization
- [ ] Test each tool handler individually
- [ ] Test error handling and edge cases
- [ ] Test parameter validation
- [ ] Test response formatting
- [ ] Achieve 80%+ code coverage

#### Task 4.2: Doctests
- [ ] Add doctests to pmcp server module
- [ ] Add doctests to each handler
- [ ] Document usage examples in docstrings
- [ ] Ensure all public APIs have doctests
- [ ] Run `make test-doc` and fix any failures

#### Task 4.3: Property Tests
- [ ] Create property tests for tool parameter generation
- [ ] Test handler idempotency
- [ ] Test error recovery
- [ ] Test concurrent request handling
- [ ] Test large input handling
- [ ] Add at least 10 property tests

#### Task 4.4: Integration Tests
- [ ] Test full MCP protocol flow
- [ ] Test initialize/shutdown sequence
- [ ] Test tool discovery (tools/list)
- [ ] Test each tool end-to-end
- [ ] Test quality_proxy with real code
- [ ] Test error responses

#### Task 4.5: Examples
- [ ] Create example: basic MCP client
- [ ] Create example: quality_proxy usage
- [ ] Create example: template generation via MCP
- [ ] Create example: full refactoring session
- [ ] Create example: concurrent tool usage
- [ ] Ensure all examples are runnable with `cargo run --example`

### Phase 5: Documentation

#### Task 5.1: Update User Documentation
- [ ] Update README.md with unified server info
- [ ] Update CLI reference (docs/cli-reference.md)
- [ ] Update MCP protocol docs (docs/mcp-protocol.md)
- [ ] Update API guide (docs/api-guide.md)
- [ ] Remove references to multiple MCP servers

#### Task 5.2: Update Developer Documentation
- [ ] Document pmcp handler development
- [ ] Create migration guide from old handlers
- [ ] Document testing approach
- [ ] Update architecture diagrams
- [ ] Add troubleshooting guide

#### Task 5.3: Update Examples README
- [ ] Document new MCP examples
- [ ] Provide usage instructions
- [ ] Include expected outputs
- [ ] Add debugging tips
- [ ] Link to relevant documentation

### Phase 6: Quality Assurance

#### Task 6.1: Run Quality Gates
- [ ] Run `pmat quality-gate` on entire codebase
- [ ] Fix any complexity violations
- [ ] Remove any SATD comments
- [ ] Ensure all public items are documented
- [ ] Run `make lint` and fix issues
- [ ] Run `make test` for full test suite

#### Task 6.2: Performance Testing
- [ ] Benchmark unified server vs old implementations
- [ ] Test memory usage under load
- [ ] Verify startup time
- [ ] Test concurrent request handling
- [ ] Document performance improvements

#### Task 6.3: Backward Compatibility
- [ ] Ensure all existing tools still work
- [ ] Verify parameter compatibility
- [ ] Test with existing MCP clients
- [ ] Document any breaking changes
- [ ] Provide migration path if needed

### Phase 7: Release

#### Task 7.1: Version Bump
- [ ] Update version to 2.2.0 in Cargo.toml
- [ ] Update CHANGELOG.md with all changes
- [ ] Update version in documentation
- [ ] Tag release commit
- [ ] Create release notes

#### Task 7.2: GitHub Release
- [ ] Commit all changes with detailed message
- [ ] Push to GitHub master branch
- [ ] Create GitHub release v2.2.0
- [ ] Upload release artifacts
- [ ] Verify CI/CD passes

#### Task 7.3: Crates.io Publication
- [ ] Run `cargo publish --dry-run`
- [ ] Fix any publication issues
- [ ] Publish to crates.io
- [ ] Verify installation: `cargo install pmat --version 2.2.0`
- [ ] Test installed binary

#### Task 7.4: Binary Release Verification
- [ ] Download GitHub release binary
- [ ] Test MCP server functionality
- [ ] Verify quality_proxy works
- [ ] Test all tools via MCP protocol
- [ ] Confirm curl install script works

### Phase 8: Post-Release Verification

#### Task 8.1: End-to-End Testing
- [ ] Install from crates.io on clean system
- [ ] Test MCP server with all tools
- [ ] Verify quality_proxy enforcement
- [ ] Run example programs
- [ ] Check documentation accuracy

#### Task 8.2: Client Testing
- [ ] Test with Claude Desktop
- [ ] Test with other MCP clients
- [ ] Verify tool discovery works
- [ ] Test error handling
- [ ] Confirm backward compatibility

## Success Criteria

1. **Single Implementation**: Only ONE MCP server implementation using pmcp SDK
2. **Feature Complete**: All existing tools available in unified server
3. **Quality Integration**: quality_proxy fully functional via MCP
4. **Comprehensive Testing**: Unit, doc, property tests, and examples all passing
5. **Performance**: Equal or better performance than previous implementations
6. **Documentation**: Clear, updated documentation for users and developers
7. **Clean Release**: v2.2.0 successfully published to GitHub and crates.io
8. **Verified Installation**: Binary works correctly when installed from crates.io

## Risk Mitigation

- **Backup Plan**: Keep old implementations in git history for rollback
- **Gradual Migration**: Test each tool thoroughly before removing old code
- **Client Compatibility**: Test with multiple MCP clients before release
- **Performance Regression**: Benchmark before and after changes
- **Documentation**: Update all docs before code removal

## Timeline

- Phase 1-2: Core implementation (2 hours)
- Phase 3-4: Testing and quality (2 hours)  
- Phase 5-6: Documentation and QA (1 hour)
- Phase 7-8: Release and verification (1 hour)

Total estimated time: 6 hours

## Notes

- The pmcp SDK provides significant performance improvements (10x faster)
- This consolidation will reduce codebase size by ~30%
- Maintenance burden will be significantly reduced
- All future MCP tools should be added to the unified implementation
- This change enables consistent quality enforcement across all tools