# TICKET-PMAT-7014: Rust Mutation Testing

**Ticket ID:** PMAT-7014
**Sprint:** 110
**Status:** ACTIVE
**Priority:** HIGH
**Created:** 2025-10-09
**Assignee:** EXTREME TDD Implementation
**Related Tickets:** PMAT-7010 (TypeScript), PMAT-7011 (Python), PMAT-7012 (Go), PMAT-7013 (C++)

---

## 📋 TICKET SUMMARY

Implement production-ready **Rust mutation testing** using tree-sitter AST parsing and EXTREME TDD methodology (RED → GREEN → REFACTOR). This is the **5th and FINAL language** in the multi-language mutation testing initiative, completing the 100% goal.

**Special Significance:** Internal dogfooding - testing PMAT itself with its own mutation testing engine.

**Target:** 80%+ mutation score achievable on well-tested Rust codebases.

---

## 🎯 ACCEPTANCE CRITERIA

### RED Phase (Failing Tests)
- [ ] Create Rust test fixtures (`fixtures/rust/calculator.rs`, `lib.rs`)
- [ ] Create stub implementations with `#[ignore]` tests
  - [ ] `rust_tree_sitter_mutations.rs` - 8 stub operators
  - [ ] `rust_mutation_generator.rs` - stub generator
- [ ] Update type system for Rust operators (pattern matching, range)
- [ ] Verify compilation with `cargo build --features rust-ast`

### GREEN Phase (Minimal Implementation)
- [ ] Implement 8 Rust mutation operators:
  1. **Binary Operator Replacement (AOR)** - `+, -, *, /, %`
  2. **Relational Operator Replacement (ROR)** - `<, >, <=, >=, ==, !=`
  3. **Logical Operator Replacement (LOR)** - `&&, ||`
  4. **Bitwise Operator Replacement (BOR)** - `&, |, ^, <<, >>, !`
  5. **Range Operator Replacement (RANGEOR)** - `..`, `..=` (NEW)
  6. **Pattern Match Replacement (PMR)** - `Some/None`, `Ok/Err` (NEW)
  7. **Method Chain Replacement (MCR)** - `.map/.filter/.unwrap` (NEW)
  8. **Lifetime/Borrow Mutation (LBM)** - `&, &mut` (NEW - detection only)
- [ ] Implement `RustMutationGenerator` AST visitor
- [ ] Remove all `#[ignore]` markers
- [ ] Verify all tests pass

### REFACTOR Phase (Production Polish)
- [ ] Create workflow example (`examples/rust_mutation_workflow.rs`)
- [ ] Write comprehensive documentation (`docs/features/RUST-MUTATION-TESTING.md`)
- [ ] Update project documentation (README.md, roadmap.md)
- [ ] Commit all changes with proper message

---

## 🏗️ TECHNICAL DESIGN

### Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Rust Mutation Testing                       │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐  │
│  │         RustMutationGenerator                        │  │
│  │  - parse_rust() → tree-sitter AST                    │  │
│  │  - visit_node() → recursive traversal                │  │
│  │  - generate_mutants() → Vec<Mutant>                  │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │      TreeSitterMutationOperator Trait                │  │
│  │  - can_mutate(node) → bool                           │  │
│  │  - mutate(node) → Vec<MutatedSource>                 │  │
│  │  - name() → &str                                     │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │           8 Rust Mutation Operators                  │  │
│  ├──────────────────────────────────────────────────────┤  │
│  │  1. RustBinaryOpMutation      (+, -, *, /, %)       │  │
│  │  2. RustRelationalOpMutation  (<, >, <=, >=, ==, !=)│  │
│  │  3. RustLogicalOpMutation     (&&, ||)              │  │
│  │  4. RustBitwiseOpMutation     (&, |, ^, <<, >>, !)  │  │
│  │  5. RustRangeOpMutation       (.., ..=)    ⭐ NEW   │  │
│  │  6. RustPatternMutation       (Some/None)  ⭐ NEW   │  │
│  │  7. RustMethodChainMutation   (.map/.filter) ⭐ NEW │  │
│  │  8. RustBorrowMutation        (&, &mut)    ⭐ NEW   │  │
│  └──────────────────────────────────────────────────────┘  │
│                          ▼                                   │
│  ┌──────────────────────────────────────────────────────┐  │
│  │              Test Execution                          │  │
│  │  - cargo test --lib                                  │  │
│  │  - Mutation score calculation                        │  │
│  └──────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Dependencies

**Already available in Cargo.toml:**
```toml
[dependencies]
tree-sitter = "0.23"
tree-sitter-rust = "0.23"

[features]
rust-ast = ["tree-sitter-rust"]
```

### Rust-Specific Considerations

1. **Range Operators**: Rust has unique range syntax
   - `..` - exclusive range (0..10)
   - `..=` - inclusive range (0..=10)
   - Mutations: `..` ↔ `..=`

2. **Pattern Matching**: Core Rust feature
   - `Some(x)` ↔ `None` in Option
   - `Ok(x)` ↔ `Err(e)` in Result
   - Match arms can be swapped

3. **Method Chaining**: Functional style
   - `.map()` ↔ `.filter()`
   - `.unwrap()` ↔ `.unwrap_or_default()`
   - `.and_then()` ↔ `.or_else()`

4. **Borrowing/Ownership**: Unique to Rust
   - `&` ↔ `&mut` (detection only - semantic complexity)
   - Lifetime annotations (detection only)

5. **Macro Invocations**: Special handling
   - Skip mutation inside macros (println!, assert!, etc.)
   - Macro invocations treated as atomic

---

## 📁 FILE STRUCTURE

### New Files (RED Phase)

```
fixtures/rust/
├── lib.rs                          # Library root
├── calculator.rs                   # Implementation with 25 functions
└── tests/
    └── calculator_test.rs          # Integration tests

server/src/services/mutation/
├── rust_tree_sitter_mutations.rs   # 8 mutation operators (~1,100 LOC)
└── rust_mutation_generator.rs      # AST visitor (~200 LOC)

server/examples/
└── rust_mutation_workflow.rs       # End-to-end workflow (~280 LOC)

docs/features/
└── RUST-MUTATION-TESTING.md        # User guide (~700 LOC)
```

---

## 🧪 TEST FIXTURES

### fixtures/rust/lib.rs

```rust
// lib.rs
pub mod calculator;
```

### fixtures/rust/calculator.rs

```rust
// calculator.rs
// Rust Calculator - Implementation for mutation testing

/// Arithmetic operators (AOR)
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}

pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn divide(a: i32, b: i32) -> i32 {
    if b == 0 { 0 } else { a / b }
}

pub fn modulo(a: i32, b: i32) -> i32 {
    if b == 0 { 0 } else { a % b }
}

/// Relational operators (ROR)
pub fn greater_than(a: i32, b: i32) -> bool {
    a > b
}

pub fn less_than(a: i32, b: i32) -> bool {
    a < b
}

pub fn greater_or_equal(a: i32, b: i32) -> bool {
    a >= b
}

pub fn less_or_equal(a: i32, b: i32) -> bool {
    a <= b
}

pub fn equal(a: i32, b: i32) -> bool {
    a == b
}

pub fn not_equal(a: i32, b: i32) -> bool {
    a != b
}

/// Logical operators (LOR)
pub fn logical_and(a: bool, b: bool) -> bool {
    a && b
}

pub fn logical_or(a: bool, b: bool) -> bool {
    a || b
}

/// Bitwise operators (BOR)
pub fn bitwise_and(a: i32, b: i32) -> i32 {
    a & b
}

pub fn bitwise_or(a: i32, b: i32) -> i32 {
    a | b
}

pub fn bitwise_xor(a: i32, b: i32) -> i32 {
    a ^ b
}

pub fn left_shift(a: i32, shift: u32) -> i32 {
    a << shift
}

pub fn right_shift(a: i32, shift: u32) -> i32 {
    a >> shift
}

pub fn bitwise_not(a: i32) -> i32 {
    !a
}

/// Range operators (RANGEOR) - Rust-specific
pub fn exclusive_range_sum(start: i32, end: i32) -> i32 {
    (start..end).sum()  // Will mutate to: start..=end
}

pub fn inclusive_range_sum(start: i32, end: i32) -> i32 {
    (start..=end).sum()  // Will mutate to: start..end
}

/// Pattern matching (PMR) - Rust-specific
pub fn unwrap_option(value: Option<i32>) -> i32 {
    match value {
        Some(x) => x,  // Will mutate to: None => x
        None => 0,
    }
}

pub fn unwrap_result(value: Result<i32, String>) -> i32 {
    match value {
        Ok(x) => x,    // Will mutate to: Err(x) => x
        Err(_) => 0,
    }
}

/// Method chaining (MCR) - Rust-specific
pub fn map_filter_example(values: Vec<i32>) -> Vec<i32> {
    values
        .iter()
        .map(|x| x * 2)      // Will mutate to: .filter(|x| x * 2)
        .filter(|x| *x > 5)  // Will mutate to: .map(|x| *x > 5)
        .copied()
        .collect()
}

/// Borrow/reference operators (LBM) - Detection only
pub fn borrow_immutable(value: &i32) -> i32 {
    *value
}

pub fn borrow_mutable(value: &mut i32) {
    *value += 1;
}
```

### fixtures/rust/tests/calculator_test.rs

```rust
// tests/calculator_test.rs
use rust_fixtures::calculator::*;

#[test]
fn test_add() {
    assert_eq!(add(2, 3), 5);
    assert_eq!(add(-1, -1), -2);
    assert_eq!(add(0, 5), 5);
    assert_eq!(add(-3, 5), 2);
}

#[test]
fn test_subtract() {
    assert_eq!(subtract(5, 3), 2);
    assert_eq!(subtract(-1, -1), 0);
    assert_eq!(subtract(0, 5), -5);
    assert_eq!(subtract(-3, 5), -8);
}

#[test]
fn test_multiply() {
    assert_eq!(multiply(2, 3), 6);
    assert_eq!(multiply(-2, 3), -6);
    assert_eq!(multiply(0, 5), 0);
    assert_eq!(multiply(-2, -3), 6);
}

#[test]
fn test_divide() {
    assert_eq!(divide(6, 3), 2);
    assert_eq!(divide(-6, 3), -2);
    assert_eq!(divide(5, 2), 2);
    assert_eq!(divide(5, 0), 0); // Division by zero
}

#[test]
fn test_modulo() {
    assert_eq!(modulo(7, 3), 1);
    assert_eq!(modulo(10, 5), 0);
    assert_eq!(modulo(5, 2), 1);
    assert_eq!(modulo(5, 0), 0); // Modulo by zero
}

#[test]
fn test_greater_than() {
    assert!(greater_than(5, 3));
    assert!(!greater_than(3, 5));
    assert!(!greater_than(3, 3));
}

#[test]
fn test_less_than() {
    assert!(less_than(3, 5));
    assert!(!less_than(5, 3));
    assert!(!less_than(3, 3));
}

#[test]
fn test_logical_and() {
    assert!(logical_and(true, true));
    assert!(!logical_and(true, false));
    assert!(!logical_and(false, true));
    assert!(!logical_and(false, false));
}

#[test]
fn test_logical_or() {
    assert!(logical_or(true, true));
    assert!(logical_or(true, false));
    assert!(logical_or(false, true));
    assert!(!logical_or(false, false));
}

#[test]
fn test_bitwise_and() {
    assert_eq!(bitwise_and(5, 3), 1);  // 101 & 011 = 001
    assert_eq!(bitwise_and(12, 10), 8); // 1100 & 1010 = 1000
}

#[test]
fn test_bitwise_or() {
    assert_eq!(bitwise_or(5, 3), 7);   // 101 | 011 = 111
    assert_eq!(bitwise_or(12, 10), 14); // 1100 | 1010 = 1110
}

#[test]
fn test_exclusive_range_sum() {
    assert_eq!(exclusive_range_sum(0, 5), 10);  // 0+1+2+3+4
    assert_eq!(exclusive_range_sum(1, 4), 6);   // 1+2+3
}

#[test]
fn test_inclusive_range_sum() {
    assert_eq!(inclusive_range_sum(0, 5), 15);  // 0+1+2+3+4+5
    assert_eq!(inclusive_range_sum(1, 4), 10);  // 1+2+3+4
}

#[test]
fn test_unwrap_option() {
    assert_eq!(unwrap_option(Some(42)), 42);
    assert_eq!(unwrap_option(None), 0);
}

#[test]
fn test_unwrap_result() {
    assert_eq!(unwrap_result(Ok(42)), 42);
    assert_eq!(unwrap_result(Err("error".to_string())), 0);
}

#[test]
fn test_map_filter_example() {
    let input = vec![1, 2, 3, 4, 5];
    let result = map_filter_example(input);
    assert_eq!(result, vec![6, 8, 10]);  // [2,4,6,8,10] filtered to >5
}

#[test]
fn test_borrow_immutable() {
    let value = 42;
    assert_eq!(borrow_immutable(&value), 42);
}

#[test]
fn test_borrow_mutable() {
    let mut value = 42;
    borrow_mutable(&mut value);
    assert_eq!(value, 43);
}
```

---

## 🔧 MUTATION OPERATORS

### 1. Binary Operator Replacement (AOR)

**AST Node:** `binary_expression`
**Operators:** `+, -, *, /, %`

**Example:**
```rust
// Original
let result = a + b;

// Mutants
let result = a - b;  // + → -
let result = a * b;  // + → *
let result = a / b;  // + → /
let result = a % b;  // + → %
```

### 2. Relational Operator Replacement (ROR)

**AST Node:** `binary_expression`
**Operators:** `<, >, <=, >=, ==, !=`

**Example:**
```rust
// Original
if x > y { ... }

// Mutants
if x < y { ... }   // > → <
if x >= y { ... }  // > → >=
if x <= y { ... }  // > → <=
if x == y { ... }  // > → ==
if x != y { ... }  // > → !=
```

### 3. Logical Operator Replacement (LOR)

**AST Node:** `binary_expression`
**Operators:** `&&, ||`

**Example:**
```rust
// Original
if a && b { ... }

// Mutant
if a || b { ... }  // && → ||
```

### 4. Bitwise Operator Replacement (BOR)

**AST Node:** `binary_expression`, `unary_expression`
**Operators:** `&, |, ^, <<, >>, !`

**Example:**
```rust
// Original
let flags = a & b;

// Mutants
let flags = a | b;  // & → |
let flags = a ^ b;  // & → ^
```

### 5. Range Operator Replacement (RANGEOR) ⭐ NEW

**AST Node:** `range_expression`
**Operators:** `..`, `..=`

**Example:**
```rust
// Original
let sum: i32 = (0..10).sum();

// Mutant
let sum: i32 = (0..=10).sum();  // .. → ..=

// Original
let sum: i32 = (0..=10).sum();

// Mutant
let sum: i32 = (0..10).sum();  // ..= → ..
```

### 6. Pattern Match Replacement (PMR) ⭐ NEW

**AST Node:** `match_pattern`
**Patterns:** `Some/None`, `Ok/Err`

**Example:**
```rust
// Original
match value {
    Some(x) => x,
    None => 0,
}

// Mutant (swap arms)
match value {
    None => x,      // Swapped
    Some(x) => 0,   // Swapped
}
```

**Note:** Complex pattern matching may be detection-only due to type constraints.

### 7. Method Chain Replacement (MCR) ⭐ NEW

**AST Node:** `call_expression` with method receivers
**Methods:** `.map`, `.filter`, `.unwrap`, `.and_then`, etc.

**Example:**
```rust
// Original
values.iter().map(|x| x * 2)

// Potential mutations (semantic complexity)
values.iter().filter(|x| x * 2)  // map → filter (likely compile error)
```

**Note:** Method chain mutations are detection-only due to type safety requirements.

### 8. Lifetime/Borrow Mutation (LBM) ⭐ NEW

**AST Node:** `reference_expression`
**Operators:** `&`, `&mut`

**Example:**
```rust
// Detected (not mutated)
fn borrow(value: &i32) { ... }       // Immutable borrow
fn borrow_mut(value: &mut i32) { ... }  // Mutable borrow
```

**Rationale:** Borrow checker prevents unsafe mutations. Detection-only for awareness.

---

## 📊 EXPECTED METRICS

Based on previous implementations:

| Metric | Target | Rationale |
|--------|--------|-----------|
| Mutants Generated | ~80-90 | More operators than other languages |
| Generation Time | <6ms | Rust AST similar complexity to C++ |
| Test Execution Time | ~5-10ms/mutant | Fast compilation with cargo |
| Mutation Score | 80%+ | With comprehensive test suite |
| Code Coverage | 100% | All operators exercised |

**Performance Comparison:**
- TypeScript: 67 mutants in 14ms
- Python: 56 mutants in 5.2ms
- Go: 62 mutants in <3ms (FASTEST)
- C++: 75 mutants in ~5ms
- **Rust: Expected ~85 mutants in <6ms** (3rd fastest)

---

## 🚧 IMPLEMENTATION CHALLENGES

### 1. Range Operator Semantics
**Challenge:** `..` vs `..=` have subtle differences in loops and slicing.

**Solution:** Straightforward swap - tests will catch semantic errors.

### 2. Pattern Matching Complexity
**Challenge:** Match arms have type constraints and exhaustiveness checking.

**Solution:**
- Simple pattern swaps (Some/None, Ok/Err)
- Skip complex patterns with guards or nested matches

### 3. Method Chaining
**Challenge:** Type inference makes method chain mutations often fail compilation.

**Solution:** Detection-only for awareness, skip actual mutation.

### 4. Borrow Checker
**Challenge:** `&` vs `&mut` mutations violate borrow checker rules.

**Solution:** Detection-only - borrow checker is a safety feature, not test gap.

### 5. Macro Invocations
**Challenge:** Mutations inside macros can break macro expansion.

**Solution:** Skip mutations inside macro invocations (`println!`, `assert!`, etc.)

---

## 📝 DOCUMENTATION OUTLINE

### docs/features/RUST-MUTATION-TESTING.md

```markdown
# Rust Mutation Testing

Production-ready AST-based mutation testing for Rust (2021 edition+).

## Quick Start

## Mutation Operators (8 operators)
1. Binary Operator Replacement (AOR)
2. Relational Operator Replacement (ROR)
3. Logical Operator Replacement (LOR)
4. Bitwise Operator Replacement (BOR)
5. Range Operator Replacement (RANGEOR) - NEW
6. Pattern Match Replacement (PMR) - NEW
7. Method Chain Replacement (MCR) - NEW (detection-only)
8. Lifetime/Borrow Mutation (LBM) - NEW (detection-only)

## Rust-Specific Features
- Range operator mutations (.., ..=)
- Pattern matching (Some/None, Ok/Err)
- Method chain detection
- Borrow checker awareness
- Macro invocation handling

## Troubleshooting
- Borrow checker errors
- Type inference issues
- Macro expansion problems

## Best Practices
- Comprehensive test coverage
- Test edge cases in pattern matching
- Verify range boundary conditions

## Performance
- Generation: <6ms for ~85 mutants
- Comparison with cargo-mutants

## Internal Dogfooding
- Testing PMAT with PMAT
```

---

## 🔄 EXTREME TDD WORKFLOW

### RED Phase Checklist
```bash
# 1. Create test fixtures
mkdir -p fixtures/rust/tests
# Create lib.rs, calculator.rs, tests/calculator_test.rs

# 2. Create stub operators
touch server/src/services/mutation/rust_tree_sitter_mutations.rs
touch server/src/services/mutation/rust_mutation_generator.rs

# 3. Update type system
# Edit server/src/services/mutation/types.rs
# Add: RangeReplacement, PatternReplacement, MethodChainReplacement, BorrowReplacement

# 4. Update ML predictor
# Edit server/src/services/mutation/ml_predictor.rs
# Add numeric mappings

# 5. Update module exports
# Edit server/src/services/mutation/mod.rs

# 6. Verify compilation
cargo build --features rust-ast
```

### GREEN Phase Checklist
```bash
# 1. Implement all 8 operators in rust_tree_sitter_mutations.rs
# 2. Implement generator in rust_mutation_generator.rs
# 3. Remove all #[ignore] markers
# 4. Run tests
cargo test --features rust-ast rust_

# 5. Verify mutant generation
cargo run --example rust_mutation_workflow --features rust-ast
```

### REFACTOR Phase Checklist
```bash
# 1. Create workflow example
touch server/examples/rust_mutation_workflow.rs

# 2. Write documentation
touch docs/features/RUST-MUTATION-TESTING.md

# 3. Update README.md
# 4. Update roadmap.md (Sprint 110 - MULTI-LANGUAGE 100% COMPLETE!)

# 5. Commit
git add .
git commit -m "feat: Rust Mutation Testing (PMAT-7014) - Multi-Language 100% COMPLETE!"
```

---

## 🎯 SUCCESS CRITERIA

- [ ] All 8 mutation operators implemented
- [ ] 80+ mutants generated from calculator.rs
- [ ] <6ms generation time achieved
- [ ] All unit tests passing (20+)
- [ ] Workflow example functional
- [ ] Documentation complete (700+ LOC)
- [ ] 80%+ mutation score on test fixtures
- [ ] Zero compilation errors
- [ ] v2.155.0 ready for release
- [ ] **MULTI-LANGUAGE MUTATION TESTING 100% COMPLETE! 🎉**

---

## 📈 MULTI-LANGUAGE PROGRESS

- ✅ TypeScript/JavaScript (PMAT-7010) - v2.144.0
- ✅ Python (PMAT-7011) - v2.152.0
- ✅ Go (PMAT-7012) - v2.153.0
- ✅ C++ (PMAT-7013) - v2.154.0
- 🔄 **Rust (PMAT-7014)** - **v2.155.0** ⚡ (FINAL - 100% COMPLETE!)

---

## 🔗 REFERENCES

- [tree-sitter-rust](https://github.com/tree-sitter/tree-sitter-rust)
- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants)
- [mutagen](https://github.com/llogiq/mutagen)
- PMAT-7010: TypeScript Mutation Testing
- PMAT-7011: Python Mutation Testing
- PMAT-7012: Go Mutation Testing
- PMAT-7013: C++ Mutation Testing

---

## 🎉 SPECIAL SIGNIFICANCE

**Internal Dogfooding:** This completes the multi-language mutation testing initiative by enabling PMAT to test itself with its own mutation testing engine.

**Milestone:** First production-ready mutation testing framework with unified architecture across 5 major languages.

---

**END OF TICKET SPECIFICATION**

*Next Steps: Execute RED phase and complete the multi-language mutation testing initiative at 100%!*
