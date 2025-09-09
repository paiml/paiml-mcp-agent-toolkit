# First-Class Ruchy Language Support Specification for PMAT

**Version**: 1.0.0  
**Status**: Active  
**Owner**: PMAT Development Team  
**Created**: 2025-09-09  

## Executive Summary

This specification defines the requirements for first-class Ruchy programming language support in the PMAT (PragMatic Agents Toolkit) system. Ruchy is a self-hosting programming language (v1.88.0) with 95.6% book compatibility that combines Swift/Kotlin ergonomics with Rust performance through transpilation.

## 1. Language Overview

### 1.1 Ruchy Language Profile
- **Version**: v1.88.0 (September 2025)
- **Type System**: Strong, static typing with inference
- **Paradigm**: Multi-paradigm (functional, object-oriented, actor model)
- **Target**: Transpiles to Rust for compilation
- **File Extensions**: `.ruchy`, `.rh`
- **Self-Hosting**: Bootstrap compiler written in Ruchy

### 1.2 Core Language Features
- **Functions**: `fun name(params) -> ReturnType { ... }`
- **Pattern Matching**: With guards and destructuring
- **Pipeline Operator**: `data |> transform |> filter`
- **Lambda Syntax**: Both `|x| x + 1` and `x => x + 1`
- **Error Handling**: Result/Option types with `?` operator
- **Control Flow**: `if`, `while`, `for`, `match`, `when`
- **Classes**: `class Name { ... }`
- **Actors**: `actor Name { ... }` for concurrency
- **Traits**: `trait Name { ... }` for interfaces
- **Module System**: `use`, `mod`, and `::` path resolution

### 1.3 Type System
```ruchy
// Primitive Types
i8, i16, i32, i64, i128
u8, u16, u32, u64, u128
f32, f64
bool, char, String, ()

// Composite Types
[T]                    // Arrays/Lists
(T1, T2, ...)         // Tuples  
T1 -> T2              // Functions
Option<T>             // Nullable types
Result<T, E>          // Error handling
&T, &mut T            // References

// Collections
HashMap<K, V>
HashSet<T>
DataFrame             // Tabular data (Polars)
Series               // Column data

// Type Aliases
type UserId = i64
type Callback = fun(i32) -> bool
```

### 1.4 Syntax Examples

#### Functions and Pattern Matching
```ruchy
// Function with pattern matching
fun classify_number(x: i32) -> String {
    match x {
        n if n < 0 => "negative",
        0 => "zero",
        1..=10 => "small positive",
        _ => "large positive"
    }
}

// Lambda expressions
let transform = |data| data |> filter(|x| x > 0) |> map(|x| x * 2)

// Generic functions
fun map<T, U>(list: [T], f: fun(T) -> U) -> [U] {
    list.iter().map(f).collect()
}
```

#### Control Flow
```ruchy
// For loops with ranges
for i in 0..5 {
    println(i);
}

// While loops
while condition {
    process_data();
}

// When expressions (Swift-style)
when {
    x < 0 -> "negative",
    x == 0 -> "zero", 
    x > 0 -> "positive"
}
```

#### Classes and Actors
```ruchy
class Point {
    x: f64,
    y: f64,
    
    fun distance_to(other: Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}

actor Counter {
    state: { count: i32 },
    
    receive increment() {
        self.count += 1;
    }
    
    receive get() -> i32 {
        self.count
    }
}
```

## 2. PMAT Integration Requirements

### 2.1 Language Detection
- **File Extensions**: Detect `.ruchy` and `.rh` files
- **Priority**: Ruchy detection should take precedence over other languages
- **Integration Point**: `src/services/languages/mod.rs::Language::from_extension()`

### 2.2 AST Parsing and Analysis
- **Parser Integration**: Extend existing Ruchy parser in `src/services/languages/ruchy.rs`
- **AST Node Types**: Support all Ruchy constructs (functions, classes, actors, traits)
- **Token Types**: 35+ token types including pipeline operators, pattern matching
- **Integration Point**: `RuchyComplexityAnalyzer::analyze_program()`

### 2.3 Complexity Analysis

#### Cyclomatic Complexity
- **Functions**: Base complexity = 1
- **Control Flow**: `if`, `while`, `for` each +1
- **Pattern Matching**: Each arm in `match` +1
- **Logical Operators**: `&&`, `||` each +1
- **Actor Handlers**: Each `receive` block +1

#### Cognitive Complexity
- **Nesting**: Additional cognitive load for nested structures
- **Control Flow**: Base cognitive complexity with nesting penalties
- **Pattern Matching**: Complex patterns increase cognitive load
- **Pipeline Operators**: Chained operations may increase complexity

#### Ruchy-Specific Complexity
- **Actor Concurrency**: Message passing patterns
- **Pipeline Chains**: Long pipeline sequences
- **Pattern Guards**: Guard expressions in match arms
- **Generic Functions**: Type parameter complexity

### 2.4 Dead Code Analysis
- **Unused Functions**: Exclude `main` and exported functions
- **Unused Variables**: Track variable usage across scopes
- **Unreachable Code**: After `return`, `panic!`, `throw`
- **Actor Analysis**: Unused message handlers and spawned actors

### 2.5 Halstead Metrics
- **Operators**: All Ruchy operators including `|>`, `=>`, `match`, `when`
- **Operands**: Identifiers, literals, function names
- **Volume**: Calculated using distinct and total counts
- **Difficulty**: Based on operator/operand ratios
- **Effort, Time, Bugs**: Standard Halstead formulas

## 3. TDG (Technical Debt Grading) Integration

### 3.1 TDG Score Components
1. **Structural Complexity** (25%): Cyclomatic complexity analysis
2. **Semantic Complexity** (25%): Cognitive complexity with Ruchy-specific patterns
3. **Duplication Ratio** (15%): Code duplication detection
4. **Coupling Score** (15%): Module and dependency analysis
5. **Documentation Coverage** (10%): Comment and docstring analysis
6. **Consistency Score** (10%): Code style and pattern consistency

### 3.2 Ruchy-Specific TDG Adjustments
- **Actor Complexity**: Special handling for actor concurrency patterns
- **Pipeline Complexity**: Pipeline operator chains evaluated for readability
- **Pattern Matching**: Complex patterns weighted appropriately
- **Generic Usage**: Type parameter usage and constraints

### 3.3 Quality Thresholds
- **A+ Grade**: ≤5 complexity, 90% coverage, minimal duplication
- **A Grade**: ≤10 complexity, 80% coverage, moderate duplication  
- **B Grade**: ≤15 complexity, 70% coverage, acceptable duplication
- **C Grade**: ≤20 complexity, 60% coverage, high duplication

## 4. Entropy Analysis Integration

### 4.1 Pattern Types for Ruchy
- **Error Handling**: Result/Option usage patterns, `?` operator chains
- **Data Validation**: Input validation and sanitization patterns
- **Resource Management**: Memory and file handle management
- **Control Flow**: Loop and conditional patterns
- **Data Transformation**: Pipeline and map/filter/reduce patterns
- **API Calls**: External service interaction patterns
- **Actor Patterns**: Message passing and state management

### 4.2 Entropy Violations
- **Repetitive Error Handling**: Similar error handling code blocks
- **Duplicated Validation**: Repeated input validation logic
- **Copy-Paste Functions**: Nearly identical function implementations
- **Inconsistent Patterns**: Mixed paradigms within modules
- **Actor Antipatterns**: Poor message passing design

### 4.3 Actionable Recommendations
- **Extract Common Patterns**: Suggest function or macro extraction
- **Standardize Idioms**: Recommend consistent Ruchy patterns
- **Reduce Duplication**: Identify refactoring opportunities
- **Improve Actor Design**: Better message flow organization

## 5. Quality-Driven Development (QDD) Integration

### 5.1 QDD Profile Extensions
- **Extreme Profile**: ≤5 complexity, 0 entropy violations, actor best practices
- **Standard Profile**: ≤10 complexity, ≤5 entropy violations, good patterns
- **Relaxed Profile**: ≤20 complexity, ≤15 entropy violations, legacy support

### 5.2 Code Generation Templates
- **Function Templates**: Standard Ruchy function patterns with error handling
- **Class Templates**: Proper class structure with common methods
- **Actor Templates**: Well-designed actor patterns with proper message handling
- **Module Templates**: Standard module organization and exports

### 5.3 Refactoring Patterns
- **Complexity Reduction**: Break down high-complexity functions
- **Actor Refactoring**: Improve message passing and state design
- **Pipeline Optimization**: Simplify complex pipeline chains
- **Pattern Matching**: Optimize match expressions for readability

## 6. MCP (Model Context Protocol) Integration

### 6.1 MCP Tools for Ruchy
- **analyze_ruchy_complexity**: Ruchy-specific complexity analysis
- **ruchy_dead_code**: Dead code detection with actor awareness
- **ruchy_refactor**: Automated refactoring suggestions
- **ruchy_quality_gate**: Quality threshold enforcement

### 6.2 Tool Specifications
```typescript
// MCP Tool: analyze_ruchy_complexity
{
  "name": "analyze_ruchy_complexity",
  "description": "Analyze complexity metrics for Ruchy source files",
  "inputSchema": {
    "type": "object",
    "properties": {
      "file_path": { "type": "string" },
      "include_actors": { "type": "boolean", "default": true },
      "include_halstead": { "type": "boolean", "default": true }
    },
    "required": ["file_path"]
  }
}
```

## 7. Implementation Plan

### 7.1 Phase 1: Core Integration (Sprint 83)
- [ ] Enhance language detection for Ruchy files
- [ ] Extend AST parser for complete Ruchy syntax support
- [ ] Implement Ruchy-specific complexity analysis
- [ ] Add Ruchy to TDG scoring system
- [ ] Basic entropy analysis for Ruchy patterns

### 7.2 Phase 2: Advanced Features (Sprint 84)
- [ ] Actor-specific analysis and metrics
- [ ] Pipeline operator complexity handling
- [ ] Advanced pattern matching analysis
- [ ] Ruchy-specific refactoring suggestions
- [ ] MCP tool integration

### 7.3 Phase 3: Quality Integration (Sprint 85)
- [ ] QDD profile extensions for Ruchy
- [ ] Code generation templates
- [ ] Quality gate integration
- [ ] Comprehensive testing with ruchy-book examples
- [ ] Documentation and examples

## 8. Testing Strategy

### 8.1 Test Data Sources
- **Ruchy Repository**: `/home/noah/src/ruchy` (v1.88.0, 361 files)
- **Ruchy Book**: `/home/noah/src/ruchy-book` (219+ examples, 95.6% compatibility)
- **Rosetta Ruchy**: `/home/noah/src/rosetta-ruchy` (Algorithm implementations)
- **Test Suites**: `/home/noah/src/ruchyruchy` (Comprehensive test cases)

### 8.2 Validation Criteria
- **Syntax Parsing**: 100% successful parsing of valid Ruchy code
- **Complexity Analysis**: Accurate complexity metrics for all constructs
- **Dead Code Detection**: Correct identification with actor awareness
- **TDG Scoring**: Consistent grading across Ruchy projects
- **Entropy Analysis**: Meaningful pattern detection and recommendations

### 8.3 Quality Gates
- All PMAT tools must handle Ruchy files without errors
- Complexity analysis must match or exceed existing Rust analysis quality
- TDG scores must be consistent with manual evaluation
- Entropy analysis must provide actionable recommendations
- MCP tools must integrate seamlessly with existing workflows

## 9. Success Metrics

### 9.1 Functional Metrics
- **Parser Coverage**: 100% of Ruchy syntax constructs supported
- **Analysis Accuracy**: Complexity metrics within ±5% of manual analysis
- **Tool Integration**: All existing PMAT commands work with Ruchy files
- **Performance**: Analysis speed comparable to Rust analysis

### 9.2 Quality Metrics
- **Test Coverage**: ≥95% test coverage for Ruchy-specific code
- **Bug Rate**: <1% error rate on valid Ruchy code
- **User Experience**: Seamless integration with existing workflows
- **Documentation**: Complete specification and usage examples

## 10. Maintenance and Evolution

### 10.1 Version Compatibility
- Support for Ruchy v1.88.0+ with backward compatibility
- Automatic detection of Ruchy version features
- Graceful handling of unsupported syntax

### 10.2 Future Enhancements
- Support for Ruchy 2.0 features as they become available
- Integration with Ruchy LSP for real-time analysis
- Advanced actor concurrency analysis
- Machine learning-based code quality predictions

## 11. Appendices

### 11.1 Ruchy Language Resources
- **Main Repository**: https://github.com/paiml/ruchy
- **Language Book**: https://github.com/paiml/ruchy-book
- **Specification**: `/home/noah/src/ruchy/docs/SPECIFICATION.md`
- **Quality Score**: 94.0/100 TDG score (A grade)

### 11.2 Implementation References
- **Current Implementation**: `src/services/languages/ruchy.rs`
- **Language Detection**: `src/services/languages/mod.rs`
- **Complexity Analyzer**: `RuchyComplexityAnalyzer`
- **Test Examples**: `tests/` directories in ruchy-book

### 11.3 Related Specifications
- **PMAT TDG Specification**: `docs/specifications/tdg-spec.md`
- **QDD Specification**: `docs/specifications/qdd-spec.md`
- **Entropy Analysis**: `docs/specifications/entropy-spec.md`

---

**Document Control**
- **Version**: 1.0.0
- **Last Updated**: 2025-09-09
- **Next Review**: 2025-10-09
- **Approval**: PMAT Architecture Review Board

**Implementation Status**: ✅ Ready for Development
**Priority**: High (Sprint 83 deliverable)
**Dependencies**: None