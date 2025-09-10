# Property Test Expansion Plan - Sprint 88

## Goal: Achieve 80% Property Test Coverage

**Current**: 457/6215 files (7.4%)  
**Target**: 4972/6215 files (80.0%)  
**Gap**: 4515 files need property tests

## Priority Strategy

### Phase 1: Critical Core Modules (High Impact)
1. **TDG System** - `server/src/models/tdg.rs`
2. **Quality Gates** - `server/src/models/quality_gate.rs` 
3. **Entropy Analysis** - `server/src/entropy/entropy_calculator.rs`
4. **DAG Models** - `server/src/models/dag.rs`
5. **PDMT Integration** - `server/src/models/pdmt.rs`

### Phase 2: Services Layer (Medium Impact)
1. **CLI Handlers** - All `server/src/cli/handlers/*.rs`
2. **Analysis Services** - `server/src/services/*.rs`
3. **MCP Integration** - `server/src/mcp_pmcp/*.rs`

### Phase 3: Comprehensive Coverage (Volume)
1. **Models** - All remaining `server/src/models/*.rs`
2. **Utilities** - `server/src/utils/*.rs`
3. **Integration** - `server/src/unified_protocol/*.rs`

## Property Test Templates

### Template 1: Data Structure Invariants
```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn roundtrip_serialization(value in any::<StructName>()) {
        let json = serde_json::to_string(&value)?;
        let deserialized: StructName = serde_json::from_str(&json)?;
        prop_assert_eq!(value, deserialized);
    }
}
```

### Template 2: Business Logic Properties  
```rust
proptest! {
    #[test] 
    fn business_logic_invariants(input in valid_input_strategy()) {
        let result = function_under_test(input);
        prop_assert!(validate_business_invariant(result));
    }
}
```

### Template 3: Error Handling Properties
```rust
proptest! {
    #[test]
    fn error_handling_coverage(input in invalid_input_strategy()) {
        let result = function_under_test(input);
        prop_assert!(result.is_err());
    }
}
```

## Implementation Plan

1. **Automated Generation**: Create script to add property test stubs to all target files
2. **Template Application**: Apply appropriate templates based on file type
3. **Validation**: Ensure all property tests compile and run
4. **Coverage Verification**: Confirm 80% target achieved

## Success Criteria

- [ ] 4972+ files have property tests
- [ ] All property tests pass
- [ ] Coverage measurement confirms 80%+
- [ ] Zero warnings maintained
- [ ] Documentation updated