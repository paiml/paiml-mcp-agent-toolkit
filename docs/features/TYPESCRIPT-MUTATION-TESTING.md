# TypeScript/JavaScript Mutation Testing

**Status:** ✅ Production Ready (v2.144.0+)
**Languages:** TypeScript, JavaScript (ES6+, JSX, TSX)
**Quality:** 80%+ mutation score achievable

---

## Overview

PMAT provides **AST-based mutation testing** for TypeScript and JavaScript projects, helping you validate test suite quality by introducing controlled bugs (mutations) and checking if your tests catch them.

**Key Benefits:**
- 🎯 Quantify test quality with mutation scores
- 🔍 Identify gaps in test coverage
- ⚡ Fast generation (14ms for 67 mutants)
- 🧬 5 mutation operators covering common bug patterns
- 🔄 Works with jest, vitest, and mocha

---

## Quick Start

### 1. Install Dependencies

```bash
cd fixtures/typescript
npm install  # Install vitest/jest
```

### 2. Run Mutation Testing

```bash
# From project root
cargo run --example typescript_mutation_workflow --features typescript-ast

# Or build the example first
cargo build --example typescript_mutation_workflow --features typescript-ast
./target/debug/examples/typescript_mutation_workflow
```

### 3. View Results

```
🧬 TypeScript Mutation Testing Workflow

📝 Reading source file: fixtures/typescript/calculator.ts
   Size: 1,776 bytes

🔧 Generating mutants...
   Generated: 67 mutants
   Time: 14ms

✅ Running baseline tests...
   Baseline tests passed ✅

🧪 Testing mutants (67 total)...
   [Progress updates...]

📊 Mutation Testing Results
   Total Mutants:    67
   Killed:           54 (80%)
   Survived:         13 (19%)
   Timeout/Error:    0

🎯 Mutation Score: 80% ✅ EXCELLENT!
```

---

## Mutation Operators

### 1. Arithmetic Operator Replacement (AOR)

**Replaces:** `+`, `-`, `*`, `/`, `%`

```typescript
// Original
function add(a: number, b: number) {
    return a + b;
}

// Mutants
return a - b;  // + → -
return a * b;  // + → *
return a / b;  // + → /
```

**Tests should fail** when arithmetic operators are changed.

### 2. Strict Equality Mutation

**Replaces:** `===`, `!==` with `==`, `!=`

```typescript
// Original
if (value === 0) {
    return true;
}

// Mutants
if (value == 0) {   // === → ==  (type coercion!)
if (value !== 0) {  // === → !== (negation)
```

**Tests should fail** when type-safe equality is weakened.

### 3. Optional Chaining Mutation

**Replaces:** `?.` with `.`

```typescript
// Original
return obj?.nested?.value;

// Mutant
return obj.nested.value;  // ?. → . (will throw if null!)
```

**Tests should fail** when null safety is removed.

### 4. Nullish Coalescing Mutation

**Replaces:** `??` with `||`

```typescript
// Original
return value ?? defaultValue;

// Mutant
return value || defaultValue;  // ?? → || (falsy vs nullish!)
```

**Tests should fail** when nullish behavior changes.

### 5. Async/Await Mutation

**Removes:** `async` and `await` keywords

```typescript
// Original
async function fetchValue(): Promise<number> {
    return await Promise.resolve(42);
}

// Mutants
function fetchValue(): Promise<number> {    // Remove async
    return Promise.resolve(42);             // Remove await
}
```

**Tests should fail** when Promise handling is broken.

---

## Understanding Mutation Scores

### Score Interpretation

| Score | Quality | Recommendation |
|-------|---------|----------------|
| **90-100%** | Excellent | Maintain current quality |
| **80-89%** | Good | Minor improvements needed |
| **70-79%** | Acceptable | Add targeted tests |
| **60-69%** | Weak | Significant gaps exist |
| **< 60%** | Poor | Major test suite overhaul needed |

### What Mutation Scores Tell You

**High Score (80%+):**
- ✅ Tests catch most bugs
- ✅ Good coverage of edge cases
- ✅ Type safety validated
- ✅ Error conditions tested

**Low Score (<70%):**
- ❌ Tests miss common bug patterns
- ❌ Weak edge case coverage
- ❌ Type coercion not tested
- ❌ Happy path bias

---

## Surviving Mutants (Test Weaknesses)

Surviving mutants indicate **real gaps** in your test suite:

### Example: Type Coercion Gap

**Mutant:** `===` → `==` (survives)

```typescript
// Code
if (b === 0) {
    throw new Error("Division by zero");
}

// Test (weak!)
expect(() => divide(10, 0)).toThrow();
// Passes even with b == 0 because all inputs are numbers!

// Better test
expect(() => divide(10, "0")).toThrow();  // Would fail with ==
```

### Example: Boundary Condition Gap

**Mutant:** `>` → `>=` (survives)

```typescript
// Code
return a > b ? a : b;

// Test (weak!)
expect(max(5, 3)).toBe(5);
// Passes even with >= because 5 > 3

// Better test
expect(max(5, 5)).toBe(5);  // Would fail with >= edge case
```

### Example: Async Testing Gap

**Mutant:** Remove `await` (survives)

```typescript
// Code
return await Promise.resolve(42);

// Test (weak!)
const result = fetchValue();
expect(result).resolves.toBeDefined();
// Passes even without await (returns Promise)

// Better test
const result = await fetchValue();
expect(typeof result).toBe('number');  // Would fail without await
```

---

## Example Project Structure

```
my-typescript-project/
├── src/
│   ├── calculator.ts          # Source code
│   └── calculator.test.ts     # Tests
├── package.json               # Test framework config
├── tsconfig.json
└── node_modules/              # Must run npm install
```

### Minimal package.json

```json
{
  "scripts": {
    "test": "vitest run"
  },
  "devDependencies": {
    "vitest": "^2.0.0",
    "typescript": "^5.0.0"
  }
}
```

---

## Advanced Usage

### Parallel Execution (Experimental)

```bash
cargo run --example typescript_mutation_workflow_parallel --features typescript-ast
```

**Benefits:**
- Uses multiple CPU cores
- Potential 8x speedup
- Same mutation score

**Note:** File locking prevents conflicts but serializes file I/O.

### Programmatic Usage

```rust
use pmat::services::mutation::TypeScriptMutationGenerator;

// Generate mutants
let generator = TypeScriptMutationGenerator::with_default_operators();
let mutants = generator.generate_mutants(&source, "test.ts")?;

// Process mutants
for mutant in mutants {
    println!("Mutant: {} at line {}", mutant.id, mutant.location.line);
}
```

---

## Limitations & Known Issues

### Current Limitations

1. **Single-file testing** - Multi-file projects not yet supported
2. **npm startup overhead** - Each mutant restarts test framework (~1.8s)
3. **No test selection** - Runs all tests for each mutant
4. **Sequential execution** - No true parallel testing yet

### Workarounds

**Speed up testing:**
```json
{
  "scripts": {
    "test": "vitest run --reporter=basic"  // Minimal output
  }
}
```

**Reduce mutants:**
- Focus on critical files
- Use smaller test suites during development
- Run full mutation testing in CI

### Future Enhancements

- [ ] Multi-file project support
- [ ] Test framework keep-alive (3-4x speedup)
- [ ] Smart test selection (2-5x speedup)
- [ ] Parallel execution (8x speedup)
- [ ] HTML reports
- [ ] CI/CD integration

---

## Troubleshooting

### "No package.json found"

**Problem:** Project root not detected

**Solution:**
```bash
# Ensure package.json exists
cd your-project
npm init -y
npm install --save-dev vitest

# Or specify project root explicitly in code
```

### "Baseline tests failed"

**Problem:** Tests fail on original code

**Solution:**
```bash
# Fix tests first
npm test

# Then run mutation testing
cargo run --example typescript_mutation_workflow
```

### "Test execution timeout"

**Problem:** Tests take too long

**Solution:**
```typescript
// Add timeout in test
test('slow operation', async () => {
    // ...
}, 30000);  // 30 second timeout
```

### "No mutants generated"

**Problem:** Code doesn't match mutation patterns

**Solution:**
- Ensure code has arithmetic operators (+, -, *, /)
- Check for === comparisons
- Verify TypeScript syntax is valid

---

## Best Practices

### 1. Start with Small Files

```bash
# Good: Single file, focused tests
calculator.ts + calculator.test.ts (67 mutants, 2 minutes)

# Avoid: Large files initially
entire-app.ts (1000+ mutants, 30+ minutes)
```

### 2. Interpret Surviving Mutants

**Don't just aim for 100%** - some mutants are equivalent:

```typescript
// These might be equivalent:
return a > b ? a : b;
return a >= b ? a : b;  // Same behavior when a > b
```

Focus on **meaningful survivors** that reveal test gaps.

### 3. Add Tests Iteratively

```typescript
// 1. Run mutation testing
// 2. Identify surviving mutants
// 3. Add tests to kill them
// 4. Re-run to verify

// Example: Add type coercion test
test('strict equality with type coercion', () => {
    expect(isEqual(0, "0")).toBe(false);  // Kills === → == mutant
});
```

### 4. Use in CI/CD

```yaml
# .github/workflows/mutation-testing.yml
name: Mutation Testing
on: [pull_request]

jobs:
  mutation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - run: npm install
      - run: cargo run --example typescript_mutation_workflow
      - run: |
          # Fail if mutation score < 70%
          if [ $MUTATION_SCORE -lt 70 ]; then exit 1; fi
```

---

## Performance Expectations

### Typical Performance

| Mutants | Sequential | Optimized (Future) |
|---------|------------|-------------------|
| 10 | ~18s | ~2s |
| 50 | ~90s | ~7s |
| 100 | ~180s | ~12s |
| 500 | ~900s (15min) | ~60s |

**Note:** Times assume ~1.8s per mutant (current) or ~0.12s (optimized)

### Scaling Recommendations

**Small projects (<50 mutants):** Run on every commit
**Medium projects (50-200 mutants):** Run on PRs
**Large projects (200+ mutants):** Run nightly or on main branch

---

## Related Documentation

- [Complete Implementation Summary](../tickets/PMAT-7010-COMPLETE-SUMMARY.md)
- [REFACTOR Phase Day 1](../tickets/PMAT-7010-REFACTOR-DAY1-COMPLETE.md)
- [Performance Optimization Plan](../tickets/PMAT-7010-REFACTOR-DAY2-PLAN.md)
- [Original Specification](../tickets/TICKET-PMAT-7010.md)

---

## Support & Contributing

### Getting Help

1. Check [Troubleshooting](#troubleshooting) section
2. Review [examples/](../../server/examples/) for working code
3. Open issue on GitHub with reproduction steps

### Contributing

Contributions welcome for:
- Additional mutation operators
- Performance optimizations
- Test framework integrations
- Documentation improvements

See [CONTRIBUTING.md](../../CONTRIBUTING.md) for guidelines.

---

## License

MIT OR Apache-2.0 (same as PMAT)

---

**Last Updated:** 2025-10-08
**Version:** 2.144.0
**Status:** Production Ready
**Maintainer:** PMAT Team

