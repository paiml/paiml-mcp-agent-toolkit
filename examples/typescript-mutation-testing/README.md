# TypeScript Mutation Testing Example

This example demonstrates how to use PMAT for mutation testing on a TypeScript project.

## Overview

Mutation testing is a technique to evaluate the quality of your test suite by introducing small changes (mutations) to your code and checking if your tests detect them.

**Mutation Score** = (Killed Mutants / Total Valid Mutants) × 100%

- **Killed**: Test suite detected the mutation (good!)
- **Survived**: Test suite did not detect the mutation (test gap!)
- **CompileError**: Mutation caused invalid syntax
- **Timeout**: Mutation caused infinite loop

## Project Structure

```
typescript-mutation-testing/
├── src/
│   └── calculator.ts         # Calculator module with 8 functions
├── tests/
│   └── calculator.test.ts    # Comprehensive test suite (24 tests)
├── package.json               # Node.js dependencies
├── tsconfig.json              # TypeScript configuration
├── jest.config.js             # Jest test configuration
└── README.md                  # This file
```

## Getting Started

### 1. Install Dependencies

First, install the required Node.js packages:

```bash
cd examples/typescript-mutation-testing
npm install
```

### 2. Run Tests

Verify that all tests pass:

```bash
npm test
```

You should see:

```
PASS  tests/calculator.test.ts
  Arithmetic Operations
    ✓ add should correctly add two numbers (2 ms)
    ✓ subtract should correctly subtract two numbers
    ✓ multiply should correctly multiply two numbers
    ✓ divide should correctly divide two numbers (1 ms)
  Logical Operations
    ✓ isEven should detect even numbers
    ✓ max should return maximum value
  Complex Operations
    ✓ factorial should calculate factorial correctly (1 ms)
    ✓ factorial should throw error for negative numbers
    ✓ isPrime should detect prime numbers
  Edge Cases
    ✓ add with large numbers
    ✓ multiply by zero
    ✓ divide by zero returns null
    ✓ isEven with negative numbers
    ✓ max with equal numbers
    ✓ factorial of zero
    ✓ isPrime for 2
    ✓ isPrime for large numbers

Test Suites: 1 passed, 1 total
Tests:       24 passed, 24 total
```

### 3. Run Mutation Testing

Use PMAT to perform mutation testing on the calculator module:

```bash
# Install pmat if not already installed
cargo install pmat

# Run mutation testing on the calculator module
pmat mutate --target src/calculator.ts
```

### 4. Analyze Results

PMAT will generate mutants and run your tests against each one. Example output:

```
Mutation Testing Results
========================
Total Mutants: 55
Killed: 50 (90.9%)
Survived: 3 (5.5%)
Compile Errors: 2 (3.6%)
Timeouts: 0 (0.0%)

Mutation Score: 90.9%
```

### 5. Review Survived Mutants (Test Gaps)

Use `--failures-only` to see only survived mutants:

```bash
pmat mutate --target src/calculator.ts --failures-only
```

This shows which mutations your tests failed to catch, indicating gaps in test coverage.

## Example: Detecting Test Gaps

### Original Code
```typescript
export function max(a: number, b: number): number {
  if (a > b) {  // Mutation: change > to >=
    return a;
  } else {
    return b;
  }
}
```

### Mutation
```typescript
export function max(a: number, b: number): number {
  if (a >= b) {  // Mutated: > changed to >=
    return a;
  } else {
    return b;
  }
}
```

### Test That Catches It
```typescript
test('max should return maximum value', () => {
  expect(max(5, 3)).toBe(5);  // Would pass with either > or >=
  expect(max(2, 8)).toBe(8);  // Would pass with either > or >=
  expect(max(4, 4)).toBe(4);  // CRITICAL: Catches the >= mutation!
});
```

Without the `max(4, 4)` test case, the `> to >=` mutation would **survive**, indicating a test gap.

## Output Formats

### Text (Default)
```bash
pmat mutate --target src/calculator.ts
```

Color-coded terminal output with summary and individual mutant details.

### JSON
```bash
pmat mutate --target src/calculator.ts --output-format json > results.json
```

Machine-readable JSON for CI/CD integration.

### Markdown
```bash
pmat mutate --target src/calculator.ts --output-format markdown > results.md
```

Human-readable report with tables and statistics.

## Advanced Usage

### Control Concurrency
```bash
# Use 4 parallel jobs
pmat mutate --target src/calculator.ts --jobs 4
```

### Set Timeout
```bash
# Timeout individual tests after 10 seconds
pmat mutate --target src/calculator.ts --timeout 10
```

### Enforce Mutation Score Threshold
```bash
# Fail if mutation score is below 85%
pmat mutate --target src/calculator.ts --threshold 85
```

This will exit with code 1 if the mutation score is below 85%, useful for CI/CD quality gates.

## CI/CD Integration

### GitHub Actions

Create `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on: [push, pull_request]

jobs:
  mutation-test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set up Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install dependencies
        run: |
          cd examples/typescript-mutation-testing
          npm install

      - name: Run tests
        run: |
          cd examples/typescript-mutation-testing
          npm test

      - name: Install Rust and pmat
        run: |
          curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
          source $HOME/.cargo/env
          cargo install pmat

      - name: Run mutation tests
        run: |
          cd examples/typescript-mutation-testing
          pmat mutate --target src/calculator.ts --failures-only --threshold 80
```

### GitLab CI

Add to `.gitlab-ci.yml`:

```yaml
mutation-testing:
  image: node:18
  stage: test
  before_script:
    - cd examples/typescript-mutation-testing
    - npm install
    - curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    - source $HOME/.cargo/env
    - cargo install pmat
  script:
    - npm test
    - pmat mutate --target src/calculator.ts --output-format json > mutation_results.json
  artifacts:
    reports:
      junit: mutation_results.json
```

## Best Practices

1. **Run mutation testing regularly** - Integrate into CI/CD pipeline
2. **Set reasonable thresholds** - 80-90% mutation score is excellent
3. **Focus on critical code** - Test business logic thoroughly
4. **Review survived mutants** - Each one represents a potential bug
5. **Use `--failures-only`** - Reduces noise in large codebases
6. **Test edge cases** - Boundary values catch more mutations
7. **Combine with coverage** - High code coverage + high mutation score = robust tests

## Understanding Mutation Operators

PMAT applies various mutation operators to TypeScript code:

### Arithmetic Operators
- `+` → `-`, `*`, `/`, `%`
- `-` → `+`, `*`, `/`, `%`
- `*` → `+`, `-`, `/`, `%`
- `/` → `+`, `-`, `*`, `%`
- `%` → `+`, `-`, `*`, `/`

### Comparison Operators
- `===` → `!==`, `<`, `>`, `<=`, `>=`
- `==` → `!=`, `<`, `>`, `<=`, `>=`
- `<` → `<=`, `>`, `>=`, `===`, `!==`
- `>` → `>=`, `<`, `<=`, `===`, `!==`
- `<=` → `<`, `>=`, `!==`
- `>=` → `>`, `<=`, `!==`
- `!==` → `===`, `<`, `>`, `<=`, `>=`
- `!=` → `==`, `<`, `>`, `<=`, `>=`

### Logical Operators
- `&&` → `||`
- `||` → `&&`
- `!` → (removed)

### Boundary Values
- `<` → `<=`
- `>` → `>=`
- `0` → `1`, `-1`
- `i++` → `i--`

### Return Values
- `return x` → `return null`, `return 0`, `return -1`
- `return true` → `return false`
- `return false` → `return true`

## Example Results

```
╔═══════════════════════════════════════════╗
║     Mutation Testing Summary              ║
╠═══════════════════════════════════════════╣
║ Total Mutants:        55                  ║
║ Killed:               50 (90.9%)          ║
║ Survived:             3 (5.5%)            ║
║ Compile Errors:       2 (3.6%)            ║
║ Timeouts:             0 (0.0%)            ║
╠═══════════════════════════════════════════╣
║ Mutation Score:       90.9% ✓             ║
╚═══════════════════════════════════════════╝
```

## Running Tests Manually

### With Jest
```bash
# Run all tests
npm test

# Run with coverage report
npm run test:coverage

# Run in watch mode
npm run test:watch
```

### Build TypeScript
```bash
npm run build
```

## Troubleshooting

### Module Resolution Errors

If you get `Cannot find module` errors, ensure your `tsconfig.json` is properly configured:

```json
{
  "compilerOptions": {
    "esModuleInterop": true,
    "resolveJsonModule": true
  }
}
```

### Jest Configuration Issues

If tests don't run, verify `jest.config.js` has the correct preset:

```javascript
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
};
```

### Mutation Testing Takes Too Long

Use the `--timeout` flag to limit test execution time:

```bash
pmat mutate --target src/calculator.ts --timeout 5
```

Or reduce concurrency if tests are CPU-bound:

```bash
pmat mutate --target src/calculator.ts --jobs 1
```

## Resources

- **PMAT Repository**: https://github.com/paiml/paiml-mcp-agent-toolkit
- **Crates.io**: https://crates.io/crates/pmat
- **Documentation**: `server/README.md`
- **Mutation Testing Paper**: [Mutation Testing: An Empirical Evaluation](https://dl.acm.org/doi/10.1145/3183440)

## License

This example is part of the PMAT project and is provided as-is for demonstration purposes.
