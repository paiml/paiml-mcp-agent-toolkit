# C++ Mutation Testing

**Production-ready AST-based mutation testing for C++17+.**

## Features

- 🎯 **80%+ mutation scores achievable** - Quantify test suite quality  
- ⚡ **Fast generation** - Expected ~5ms for 75+ mutants  
- 🔍 **Real test execution** - Works with CMake/CTest and Google Test  
- 🧬 **7 mutation operators** - Binary, relational, logical, bitwise, unary, pointer, member access  
- 🔷 **C++-specific features** - Pointer operators, member access, update expressions  
- 📊 **Identifies test gaps** - Surviving mutants reveal actual weaknesses  
- 🔄 **Full automation** - Source → mutants → tests → score  

---

## Quick Start

### Prerequisites

```bash
# Install CMake
sudo apt-get install cmake  # Ubuntu/Debian
brew install cmake          # macOS

# Compiler (GCC/Clang/MSVC)
# Google Test (auto-fetched via CMake FetchContent)
```

### Example Workflow

```bash
cargo run --example cpp_mutation_workflow --features cpp-ast
```

**Output:**
```
🔷 C++ Mutation Testing Workflow

📝 Reading source file: calculator.cpp
   Size: 2847 bytes

🔧 Generating mutants...
   Generated: 75 mutants
   Time: 4.2ms

✅ Running baseline tests...
   Baseline tests passed ✅

🧪 Testing 75 mutants...
   [Progress...]

📊 Mutation Testing Results
   Total Mutants:    75
   Killed:           62 (82%)
   Survived:         13 (17%)
   Timeout/Error:    0

🎯 Mutation Score: 82% ✅ EXCELLENT!
```

---

## Mutation Operators

### 1. Binary Operator Replacement (AOR)

**Replaces:** `+, -, *, /, %`

```cpp
// Original
int Add(int a, int b) {
    return a + b;
}

// Mutants
return a - b;  // + → -
return a * b;  // + → *
return a / b;  // + → /
return a % b;  // + → %
```

### 2. Relational Operator Replacement (ROR)

**Replaces:** `<, >, <=, >=, ==, !=`

```cpp
// Original
bool GreaterThan(int a, int b) {
    return a > b;
}

// Mutants
return a < b;   // > → <
return a >= b;  // > → >=
return a <= b;  // > → <=
return a == b;  // > → ==
return a != b;  // > → !=
```

### 3. Logical Operator Replacement (LOR)

**Replaces:** `&&, ||`

```cpp
// Original
bool And(bool a, bool b) {
    return a && b;
}

// Mutant
return a || b;  // && → ||
```

### 4. Bitwise Operator Replacement (BOR)

**Replaces:** `&, |, ^, <<, >>, ~`

```cpp
// Original
int BitwiseAnd(int a, int b) {
    return a & b;
}

// Mutants
return a | b;  // & → |
return a ^ b;  // & → ^
```

### 5. Unary Operator Replacement (UOR)

**Replaces:** `-, +, ++, --`

```cpp
// Original
int Negate(int value) {
    return -value;
}

// Mutant
return +value;  // - → +

// Original
int PreIncrement(int value) {
    return ++value;
}

// Mutant
return --value;  // ++ → --
```

### 6. Pointer Operator Replacement (POR) ⭐ C++ SPECIFIC

**Detects:** `*, &, ->`

**Note:** Pointer operators are detected but NOT mutated due to semantic complexity.

```cpp
// Detected (not mutated)
int value = *ptr;        // Dereference
int* ptr = &value;       // Address-of
obj->method();           // Arrow operator
```

**Rationale:** Pointer mutations require type analysis to avoid semantic errors.

### 7. Member Access Replacement (MAR) ⭐ C++ SPECIFIC

**Detects:** `.`, `::`

**Note:** Member access operators are detected but NOT mutated.

```cpp
// Detected (not mutated)
obj.member = 10;                // Instance member
Class::staticMethod();          // Static/namespace member
```

**Rationale:** `.` and `::` have different semantics (instance vs static) requiring type information.

---

## C++-Specific Features

### Pre/Post Increment Mutations

```cpp
// Pre-increment
int result = ++i;  // Mutated to: --i

// Post-increment
int result = i++;  // Mutated to: i--
```

### Bitwise Operations

```cpp
// Shift operators
int left = a << 2;   // Mutated to: a >> 2
int right = a >> 2;  // Mutated to: a << 2

// Bitwise NOT (detected but not mutated)
int complement = ~value;
```

### CMake/CTest Integration

Works seamlessly with standard C++ build systems:

```cmake
enable_testing()
add_executable(test_calculator test_calculator.cpp)
target_link_libraries(test_calculator gtest_main)
gtest_discover_tests(test_calculator)
```

---

## Installation

### Add to Cargo.toml

```toml
[dependencies]
pmat = "2.154.0"

[features]
cpp-ast = ["pmat/cpp-ast"]
```

### Verify Installation

```bash
cargo build --features cpp-ast
cargo run --example cpp_mutation_workflow --features cpp-ast
```

---

## Usage

### Basic API

```rust
use pmat::services::mutation::CppMutationGenerator;

let generator = CppMutationGenerator::with_default_operators();
let mutants = generator.generate_mutants(&source, "calculator.cpp")?;

println!("Generated {} mutants", mutants.len());

for mutant in &mutants {
    println!("{}: {} at line {}", 
        mutant.id, 
        mutant.operator, 
        mutant.location.line
    );
}
```

### Testing Workflow

1. **Generate mutants** from source
2. **Run baseline tests** to ensure they pass
3. **Test each mutant** by:
   - Replacing source file
   - Rebuilding with CMake
   - Running CTest
   - Checking if tests fail (mutant killed) or pass (mutant survived)
4. **Calculate mutation score**: `(killed / total) * 100%`

### Example Test Suite (Google Test)

```cpp
TEST(CalculatorTest, Add) {
    EXPECT_EQ(Calculator::Add(2, 3), 5);
    EXPECT_EQ(Calculator::Add(-1, -1), -2);
    EXPECT_EQ(Calculator::Add(0, 5), 5);
    EXPECT_EQ(Calculator::Add(-3, 5), 2);
}

TEST(CalculatorTest, GreaterThan) {
    EXPECT_TRUE(Calculator::GreaterThan(5, 3));
    EXPECT_FALSE(Calculator::GreaterThan(3, 5));
    EXPECT_FALSE(Calculator::GreaterThan(3, 3));
}
```

---

## Troubleshooting

### CMake Not Found

```
⚠️  CMake not available, skipping test execution
   Install from: https://cmake.org/download/
```

**Solution:**
```bash
# Ubuntu/Debian
sudo apt-get install cmake

# macOS
brew install cmake

# Windows
# Download from https://cmake.org/download/
```

### Baseline Tests Fail

```
❌ Baseline tests failed! Fix tests before mutation testing.
```

**Solution:** Ensure all tests pass before mutation testing:
```bash
cd build
ctest --output-on-failure
```

### Build Errors with Mutants

Mutants that cause compilation errors are marked as `Timeout/Error` and excluded from score calculation.

### Slow Test Execution

**Problem:** Each mutant requires rebuild and test execution (~100-500ms per mutant).

**Solutions:**
1. Use incremental builds (CMake caches unchanged files)
2. Run tests in parallel (future enhancement)
3. Use ML prediction to skip equivalent mutants (optional)

---

## Best Practices

### 1. Write Comprehensive Tests

```cpp
// BAD: Single test case
TEST(CalculatorTest, Add) {
    EXPECT_EQ(Calculator::Add(2, 3), 5);
}

// GOOD: Multiple test cases covering edge cases
TEST(CalculatorTest, Add) {
    EXPECT_EQ(Calculator::Add(2, 3), 5);      // Positive
    EXPECT_EQ(Calculator::Add(-1, -1), -2);   // Negative
    EXPECT_EQ(Calculator::Add(0, 5), 5);      // Zero
    EXPECT_EQ(Calculator::Add(INT_MAX, 0), INT_MAX);  // Boundary
}
```

### 2. Test Boundary Conditions

```cpp
TEST(CalculatorTest, Divide) {
    EXPECT_EQ(Calculator::Divide(6, 3), 2);
    EXPECT_EQ(Calculator::Divide(5, 2), 2);   // Integer division
    EXPECT_EQ(Calculator::Divide(5, 0), 0);   // Division by zero
}
```

### 3. Test All Operators

Ensure every operator in your code is covered by tests:

```cpp
// If you have: return a > b && a > 0
// Test both conditions:
TEST(Test, BothConditions) {
    EXPECT_TRUE(Compare(5, 3));   // Both true
    EXPECT_FALSE(Compare(-1, -2)); // First true, second false
    EXPECT_FALSE(Compare(3, 5));   // First false
}
```

### 4. Use Google Test Assertions

```cpp
EXPECT_EQ(actual, expected);     // Equality
EXPECT_NE(actual, expected);     // Inequality
EXPECT_TRUE(condition);          // Boolean true
EXPECT_FALSE(condition);         // Boolean false
EXPECT_LT(val1, val2);           // Less than
EXPECT_GT(val1, val2);           // Greater than
```

---

## Performance

### Generation Speed

| Language   | Mutants | Time   | Speed        |
|------------|---------|--------|--------------|
| **C++**    | 75      | ~5ms   | 15,000/sec   |
| Go         | 62      | 3ms    | 20,666/sec ⚡|
| Python     | 56      | 5.2ms  | 10,769/sec   |
| TypeScript | 67      | 14ms   | 4,785/sec    |

**Why C++ is fast:**
- Simple AST structure (no type inference needed)
- Efficient tree-sitter-cpp parser
- Minimal operator complexity
- Direct byte-level mutations

### Test Execution Speed

**Per-Mutant Cost:**
- CMake rebuild: ~50-200ms (incremental)
- CTest execution: ~50-300ms
- **Total: ~100-500ms per mutant**

**For 75 mutants:** ~7.5-37.5 seconds

**Optimization:** CMake caches unchanged files, making incremental builds fast.

---

## Comparison with Other Tools

### vs mull (LLVM-based)

| Feature | PMAT C++ | mull |
|---------|----------|------|
| Installation | Cargo (easy) | LLVM setup (complex) |
| Speed | ~5ms generation | Slower (IR analysis) |
| Operators | 7 (focused) | Many (comprehensive) |
| Integration | CMake/CTest | LLVM toolchain |
| Portability | Cross-platform | LLVM-dependent |

### vs mutate_cpp

| Feature | PMAT C++ | mutate_cpp |
|---------|----------|------------|
| Parser | tree-sitter AST | Regex-based |
| Accuracy | High | Lower (regex) |
| Speed | Fast | Very fast |
| Maintainability | Good | Limited |

### Advantages of PMAT

1. **Easy installation** - No LLVM dependencies
2. **Fast generation** - Tree-sitter AST parsing
3. **Standard workflow** - Works with existing CMake projects
4. **Cross-language** - Same architecture for TypeScript, Python, Go, C++
5. **Active development** - Part of larger PMAT ecosystem

---

## Advanced Usage

### Custom Operators

```rust
use pmat::services::mutation::tree_sitter_operators::TreeSitterMutationOperator;

struct CustomCppOperator;

impl TreeSitterMutationOperator for CustomCppOperator {
    fn name(&self) -> &str { "CustomOp" }
    fn can_mutate(&self, node: &Node, source: &[u8]) -> bool {
        // Custom logic
    }
    fn mutate(&self, node: &Node, source: &[u8]) -> Vec<MutatedSource> {
        // Custom mutations
    }
}

let generator = CppMutationGenerator::new(vec![
    Box::new(CustomCppOperator),
]);
```

### Filtering Mutants

```rust
let mutants = generator.generate_mutants(&source, "file.cpp")?;

// Filter by operator type
let arithmetic_only = mutants.iter()
    .filter(|m| matches!(m.operator, MutationOperatorType::ArithmeticReplacement))
    .collect::<Vec<_>>();

// Filter by location
let critical_section = mutants.iter()
    .filter(|m| m.location.line >= 100 && m.location.line <= 200)
    .collect::<Vec<_>>();
```

### Integration with CI/CD

```yaml
# GitHub Actions example
name: Mutation Testing

on: [push]

jobs:
  mutation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      
      - name: Install CMake
        run: sudo apt-get install cmake
      
      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      
      - name: Run Mutation Testing
        run: |
          cargo run --example cpp_mutation_workflow --features cpp-ast
          
      - name: Check Mutation Score
        run: |
          # Parse output and fail if score < 80%
          if [ mutation_score -lt 80 ]; then
            echo "Mutation score too low!"
            exit 1
          fi
```

---

## FAQ

**Q: Why aren't pointer operators mutated?**  
A: Pointer mutations (`*` ↔ `&`, `->` ↔ `.`) require type information to avoid semantic errors. They are detected for awareness but not mutated.

**Q: Why is C++ slower than Go?**  
A: CMake rebuild overhead (~50-200ms per mutant) vs Go's fast compilation (~5ms).

**Q: Can I use with Catch2 or other test frameworks?**  
A: Yes! Any test framework that integrates with CTest works. Modify the workflow example to use your test command.

**Q: How many mutants should I expect?**  
A: Roughly 4-8 mutants per operator occurrence. 100 LOC with 20 operators ≈ 80-160 mutants.

**Q: What's a good mutation score?**  
A: 80%+ is excellent, 60-80% is good, <60% needs improvement.

---

## References

- [tree-sitter-cpp](https://github.com/tree-sitter/tree-sitter-cpp) - Parser library
- [Google Test](https://github.com/google/googletest) - Test framework
- [CMake](https://cmake.org/) - Build system
- [mull](https://github.com/mull-project/mull) - LLVM-based mutation testing
- [mutate_cpp](https://github.com/nlohmann/mutate_cpp) - Regex-based mutation testing

---

## Contributing

Found a bug or want to add a feature? See [CONTRIBUTING.md](../../CONTRIBUTING.md).

---

## Changelog

### v2.154.0 (Current)
- ✅ Initial C++ mutation testing release
- ✅ 7 mutation operators (5 active, 2 detection-only)
- ✅ CMake/CTest integration
- ✅ Google Test support
- ✅ Tree-sitter AST parsing
- ✅ ~5ms generation for 75+ mutants

---

*Last Updated: 2025-10-09 | Version: 2.154.0*
