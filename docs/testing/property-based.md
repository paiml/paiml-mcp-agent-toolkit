# Property-Based Testing Guide

*Reference: SPECIFICATION.md Section 28*

## Overview

PMAT employs comprehensive property-based testing using the `proptest` framework to ensure correctness across a wide range of inputs. This guide documents our property testing approach and standards.

## Core Properties

### 1. Complexity Analysis Properties

Tests verify that complexity metrics are:
- **Deterministic**: Same input always produces same complexity score
- **Monotonic**: Adding complexity increases scores
- **Bounded**: Scores stay within expected ranges (0-100 for most metrics)

Reference implementation: `src/tests/property_tests.rs`

### 2. SATD Detection Properties

Properties ensure SATD detection:
- **Completeness**: All known patterns are detected
- **Accuracy**: No false positives for clean code
- **Stability**: Detection is consistent across runs

### 3. Refactoring Properties

Critical properties for refactoring:
- **Semantic Preservation**: Refactored code maintains behavior
- **Quality Improvement**: Complexity decreases or stays same
- **Syntactic Validity**: Output is valid Rust code

## Test Strategy

### Input Generation

```rust
proptest! {
    #[test]
    fn test_complexity_bounds(
        code in any::<String>().prop_filter("valid rust", |s| s.len() < 10000)
    ) {
        let complexity = analyze_complexity(&code);
        prop_assert!(complexity.cyclomatic >= 0);
        prop_assert!(complexity.cyclomatic <= 1000);
    }
}
```

### Shrinking

When a property fails, proptest automatically shrinks the input to find minimal failing case:
- Start with complex input that fails
- Systematically reduce complexity
- Find simplest input that still fails

### Coverage

Property tests cover:
- **64+ core properties** across all major components
- **10,000+ test cases** per property by default
- **Edge cases** through targeted generators

## Running Property Tests

```bash
# Run all property tests
make test-property

# Run with more iterations (slower but more thorough)
PROPTEST_CASES=100000 cargo test --test property

# Run specific property test
cargo test --test property test_complexity_bounds
```

## Writing New Properties

Follow these patterns when adding property tests:

1. **Define the property clearly**
   - What invariant should hold?
   - What are the bounds?

2. **Generate appropriate inputs**
   - Use custom generators for domain types
   - Apply filters for valid inputs

3. **Assert the property**
   - Check invariants hold
   - Verify postconditions

4. **Document the property**
   - Explain what it tests
   - Reference specification section

## Integration with CI

Property tests run in CI with:
- Standard iteration count for PR checks
- Extended iteration count for nightly builds
- Seed fixation for reproducibility

See `.github/workflows/test.yml` for configuration.

## Related Documentation

- [Integration Testing](./integration.md)
- [Performance Testing](./performance.md)
- [SPECIFICATION.md Section 28](../SPECIFICATION.md#28-property-based-testing)