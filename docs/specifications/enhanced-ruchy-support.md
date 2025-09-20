# Enhanced Ruchy Language Support Specification

## Executive Summary

This specification defines the comprehensive integration of Ruchy language support into PMAT (Pragmatic MCP Agent Toolkit). Ruchy is a modern ML-style functional systems programming language with advanced type inference, zero-cost abstractions, and WebAssembly compilation capabilities. The integration will provide AST extraction, complexity analysis, entropy-based refactoring patterns, and quality gate enforcement for Ruchy codebases.

## 1. Language Overview

### 1.1 Ruchy Language Characteristics
- **Paradigm**: Functional-first with ML-style syntax
- **Type System**: Advanced type inference with refinement types
- **Compilation Targets**: Native, WebAssembly, Python transpilation
- **Key Features**:
  - Pattern matching with exhaustiveness checking
  - Actor-based concurrency model
  - Dataframe operations with type safety
  - Formal verification capabilities
  - Notebook-based development environment
  - Zero-cost abstractions

### 1.2 File Extensions
- `.ruchy` - Primary source files
- `.rh` - Alternative extension
- `.ruchynb` - Notebook files

### 1.3 Language Constructs to Analyze
- **Functions**: `let fn_name(params) = expr`
- **Types**: Type aliases, enum variants, records
- **Modules**: Module declarations and imports
- **Actors**: Actor definitions with message handlers
- **Patterns**: Match expressions, guards, destructuring
- **Proofs**: Theorem statements and tactics

## 2. Entropy Analysis Results

Based on the entropy analysis of the Ruchy codebase (1,883 files analyzed):

### 2.1 Identified Refactoring Patterns
1. **DataValidation Pattern** (70% of violations)
   - Repeated validation logic across 10+ locations
   - Potential LOC reduction: 15,522 lines
   - Recommendation: Create centralized validation trait system

2. **DataTransformation Pattern** (8% of violations)
   - Duplicated transformation pipelines
   - Potential LOC reduction: 1,180 lines
   - Recommendation: Extract to composable transformation modules

3. **ResourceManagement Pattern** (5% of violations)
   - Manual resource handling without RAII
   - Potential LOC reduction: 806 lines
   - Recommendation: Implement guard types and RAII patterns

4. **ApiCall Pattern** (5% of violations)
   - Repeated API client code
   - Potential LOC reduction: 648 lines
   - Recommendation: Create unified API abstraction layer

### 2.2 Quality Metrics
- **Total Violations**: 37 (excellent - target is 10-50)
- **Potential LOC Reduction**: 21,546 lines (5.3%)
- **Pattern Diversity**: Good (4 distinct pattern types)
- **Cross-file Duplication**: Moderate (validation patterns shared across modules)

## 3. AST Extraction Requirements

### 3.1 Core AST Elements

```rust
pub struct RuchyAstExtractor {
    items: Vec<AstItem>,
    current_module: Option<String>,
    type_context: HashMap<String, TypeInfo>,
    actor_context: HashMap<String, ActorInfo>,
}

pub struct TypeInfo {
    name: String,
    kind: TypeKind, // Alias, Enum, Record, Refinement
    line_number: usize,
}

pub struct ActorInfo {
    name: String,
    messages: Vec<String>,
    handlers: Vec<String>,
    line_number: usize,
}
```

### 3.2 Extraction Priorities
1. **Module Structure**: Module declarations, imports, exports
2. **Function Definitions**: Name, parameters, return type hints
3. **Type Definitions**: Type aliases, enums, records
4. **Actor Definitions**: Actor names, message types, handlers
5. **Pattern Matches**: Complexity contribution from match arms
6. **Proof Constructs**: Theorem statements, lemmas, tactics

### 3.3 Complexity Calculation

Ruchy-specific complexity factors:
- **Base Complexity**: Standard cyclomatic complexity
- **Pattern Complexity**: +1 per match arm, +2 for guards
- **Actor Complexity**: +3 per message handler
- **Proof Complexity**: +5 per tactic application
- **Type Complexity**: +1 per type parameter, +2 for refinement types

## 4. Implementation Strategy

### 4.1 Phase 1: Parser Integration (Week 1)
- Integrate Ruchy's existing Logos-based lexer
- Use Ruchy AST types from `frontend::ast`
- Implement basic AST traversal

### 4.2 Phase 2: Complexity Analysis (Week 2)
- Calculate cyclomatic complexity for functions
- Add pattern matching complexity
- Implement actor complexity metrics

### 4.3 Phase 3: Quality Gates (Week 3)
- Zero SATD enforcement
- Complexity thresholds (≤10 standard, ≤5 extreme)
- Test coverage requirements (80% minimum)

### 4.4 Phase 4: Advanced Features (Week 4)
- Proof verification integration
- WebAssembly analysis for notebooks
- Dataframe operation optimization

## 5. Testing Requirements

### 5.1 Unit Tests (TDD RED-GREEN-REFACTOR)
```rust
#[cfg(test)]
mod tests {
    // RED: Write failing test first
    #[test]
    fn test_extract_ruchy_function() {
        let source = "let add(x, y) = x + y";
        let extractor = RuchyAstExtractor::new();
        let items = extractor.analyze(source).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].name, "add");
    }
}
```

### 5.2 Property Tests
```rust
proptest! {
    #[test]
    fn ruchy_ast_extraction_preserves_structure(
        source in ruchy_source_strategy()
    ) {
        let items = extract_ast(&source);
        prop_assert!(items.len() <= count_definitions(&source));
    }
}
```

### 5.3 Integration Tests
- Parse real Ruchy standard library
- Verify against known complexity baselines
- Cross-validate with Ruchy's own quality tools

## 6. Quality Standards

### 6.1 Toyota Way Compliance
- **Kaizen**: Incremental improvements to parser accuracy
- **Genchi Genbutsu**: Use actual Ruchy codebases for testing
- **Jidoka**: Automated quality gates for every commit

### 6.2 Zero Defect Requirements
- All tests must pass (unit, property, integration)
- Zero SATD in implementation
- Complexity ≤10 for all functions
- 80% test coverage minimum

### 6.3 Performance Targets
- Parse 1000 lines/second minimum
- Memory usage <100MB for large files
- Cache parsed results for repeated analysis

## 7. Integration Points

### 7.1 CLI Integration
```bash
pmat analyze complexity --language ruchy file.ruchy
pmat tdg analyze file.ruchy --include-actors
pmat refactor auto --file file.ruchy --profile functional
```

### 7.2 MCP Tool Integration
```json
{
  "tool": "analyze_ruchy",
  "params": {
    "file_path": "src/main.ruchy",
    "include_proofs": true,
    "actor_analysis": true
  }
}
```

### 7.3 Quality Gate Integration
- Add Ruchy to supported languages in quality gates
- Include actor complexity in thresholds
- Validate proof correctness when present

## 8. Deliverables

### 8.1 Code Artifacts
1. `server/src/services/languages/ruchy.rs` - Core implementation
2. `server/src/services/languages/ruchy_ast.rs` - AST extraction
3. `server/src/services/languages/ruchy_complexity.rs` - Complexity analysis
4. Tests with 80% coverage via property testing

### 8.2 Documentation
1. API documentation with examples
2. Integration guide for Ruchy developers
3. Complexity calculation methodology

### 8.3 Examples
1. Ruchy standard library analysis
2. Actor system complexity assessment
3. Proof verification integration

## 9. Success Criteria

### 9.1 Functional Requirements
- [x] Parse all valid Ruchy syntax
- [ ] Extract functions, types, modules, actors
- [ ] Calculate accurate complexity metrics
- [ ] Integrate with existing quality gates

### 9.2 Non-Functional Requirements
- [ ] Performance: 1000+ lines/second
- [ ] Accuracy: 95%+ AST extraction accuracy
- [ ] Coverage: 80%+ test coverage
- [ ] Quality: Zero SATD, complexity ≤10

### 9.3 Validation
- [ ] Parse Ruchy compiler source (self-hosting test)
- [ ] Analyze 100+ Ruchy notebooks
- [ ] Cross-validate with Ruchy quality tools
- [ ] User acceptance from Ruchy team

## 10. Risk Mitigation

### 10.1 Technical Risks
- **Parser Complexity**: Mitigate with incremental implementation
- **Performance**: Use caching and lazy evaluation
- **Accuracy**: Extensive testing against real codebases

### 10.2 Integration Risks
- **Version Compatibility**: Pin to Ruchy v1.89.0 initially
- **Feature Conflicts**: Use feature flags for optional compilation
- **Maintenance**: Automated tests for regression detection

## 11. Timeline

### Week 1: Foundation
- [ ] Create ruchy.rs with basic structure
- [ ] Implement lexer integration
- [ ] Write initial TDD tests

### Week 2: Core Features
- [ ] AST extraction for functions and types
- [ ] Complexity calculation
- [ ] Property test suite

### Week 3: Advanced Features
- [ ] Actor analysis
- [ ] Pattern matching metrics
- [ ] Proof verification hooks

### Week 4: Integration & Polish
- [ ] CLI integration
- [ ] MCP tool creation
- [ ] Documentation and examples

## 12. Appendix: Ruchy Code Samples

### 12.1 Function Definition
```ruchy
let fibonacci(n) =
  match n with
  | 0 -> 0
  | 1 -> 1
  | n -> fibonacci(n - 1) + fibonacci(n - 2)
```

### 12.2 Actor Definition
```ruchy
actor Counter {
  state count = 0

  message Increment -> {
    count := count + 1
  }

  message GetCount -> Int {
    return count
  }
}
```

### 12.3 Type Definition with Refinement
```ruchy
type PositiveInt = { x: Int | x > 0 }

let safe_divide(a: Int, b: PositiveInt) -> Float =
  a.to_float() / b.to_float()
```

## 13. References

- Ruchy Language Repository: `/home/noah/src/ruchy`
- Ruchy Documentation: `docs/` in Ruchy repository
- PMAT Integration Guide: `docs/integration-guide.md`
- Toyota Way Principles: `CLAUDE.md`