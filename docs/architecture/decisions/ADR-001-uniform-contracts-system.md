# ADR-001: Uniform Contracts System for Interface Consistency

## Status
**Accepted** - 2025-08-28

## Context

PMAT has multiple interfaces (CLI, MCP, HTTP) that were developed with inconsistent parameter naming and behavior:

### Problems Identified
1. **Parameter Inconsistencies**: 
   - CLI used `--project-path` while MCP used `path`
   - Some commands had `--file` vs `--files` variations
   - HTTP endpoints used different JSON field names
   
2. **No Single Source of Truth**: 
   - Each interface defined parameters independently
   - Changes required updates in multiple places
   - Easy to introduce inconsistencies
   
3. **User Confusion**:
   - Different parameter names for same functionality across interfaces
   - Inconsistent error messages and validation
   - Different response formats

4. **Maintenance Overhead**:
   - Duplicate validation logic
   - Multiple parameter parsing implementations
   - Difficult to ensure consistency during development

## Decision

We will implement a **Uniform Contracts System** that enforces identical parameter names and behaviors across all interfaces.

### Core Architecture

#### 1. Contract Definitions (`contracts/mod.rs`)
- `BaseAnalysisContract` with shared parameters
- Specialized contracts for each command type
- Built-in validation and type safety
- Serde serialization/deserialization support

#### 2. Service Layer Architecture
- Unified service processing all contracts
- Single implementation for business logic
- Consistent response formats
- Error handling standardization

#### 3. Interface Implementations
- **MCP Handler**: Direct contract usage with auto-generated schemas
- **HTTP Server**: JSON endpoints using contracts with validation
- **CLI Commands**: New uniform parameter definitions
- **Backward Compatibility**: Adapter layer for transition period

#### 4. Quality Assurance
- Contract validation in CI/CD pipeline
- Comprehensive test coverage
- Type safety enforcement at compile time
- Documentation synchronization requirements

### Contract Structure
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BaseAnalysisContract {
    pub path: PathBuf,              // ALWAYS "path", never "project_path"
    pub format: OutputFormat,       // ALWAYS same enum
    pub output: Option<PathBuf>,    // ALWAYS optional
    pub top_files: Option<usize>,   // ALWAYS same name
    pub include_tests: bool,        // ALWAYS same behavior
    pub timeout: u64,               // ALWAYS available
}
```

## Consequences

### Positive
1. **Parameter Consistency**: All interfaces use identical parameter names
2. **Single Source of Truth**: Contract definitions eliminate inconsistencies
3. **Type Safety**: Compile-time validation prevents parameter errors
4. **Maintainability**: Changes in one place propagate to all interfaces
5. **User Experience**: Consistent behavior across all interfaces
6. **Testing**: Unified test suite covers all interfaces
7. **Documentation**: Auto-generated schemas and consistent documentation

### Negative
1. **Breaking Changes**: Existing parameter names will be deprecated
2. **Migration Effort**: Need to update existing commands over time
3. **Complexity**: Additional abstraction layer in codebase
4. **Learning Curve**: Developers need to understand contract system

### Mitigation Strategies
1. **Backward Compatibility**: Adapter layer maps old parameters to contracts
2. **Phased Migration**: Gradual rollout with deprecation warnings
3. **Comprehensive Documentation**: Clear migration guides and examples
4. **Extensive Testing**: Full test coverage for all migration scenarios

## Implementation Status

### Phase 1: Foundation ✅ COMPLETE
- [x] Contract definitions implemented
- [x] Service layer with contract processing
- [x] MCP handler using contracts
- [x] HTTP endpoints with contract validation
- [x] Contract versioning system
- [x] Backward compatibility adapter
- [x] Comprehensive test suite (9/9 tests passing)
- [x] CI/CD enforcement workflow
- [x] Documentation and roadmap

### Phase 2: CLI Migration (Next)
- [ ] Migrate existing CLI commands to uniform parameters
- [ ] Update help text and documentation
- [ ] Add deprecation warnings for old parameters
- [ ] Full integration testing

### Phase 3: Legacy Cleanup (Future)
- [ ] Remove backward compatibility layer
- [ ] Clean up old parameter definitions
- [ ] Final validation and testing

## Alternatives Considered

### Alternative 1: Manual Consistency
- **Description**: Manually ensure parameter consistency across interfaces
- **Rejected**: Error-prone, high maintenance overhead, no compile-time guarantees

### Alternative 2: Configuration-Based Approach
- **Description**: Use configuration files to define parameter mappings
- **Rejected**: Runtime overhead, complex configuration management, no type safety

### Alternative 3: Code Generation
- **Description**: Generate interface code from shared definitions
- **Rejected**: Complex build process, limited flexibility, debugging difficulties

## References
- [Uniform Contracts Roadmap](../execution/roadmap.md)
- [Contract System Documentation](../../server/src/contracts/README.md)
- [Implementation Specifications](../../specifications/)

---
**Authors**: Claude Agent  
**Reviewers**: Project Team  
**Approval Date**: 2025-08-28  
**Next Review**: Sprint 1 Completion