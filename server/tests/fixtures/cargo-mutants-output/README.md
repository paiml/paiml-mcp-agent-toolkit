# cargo-mutants Test Fixtures

Test fixtures containing real cargo-mutants v25.3.1 output for integration testing.

## Fixtures

### `some-missed/`
**Source**: Real output from `/tmp/pmat-mutate-test` (simple add function)
**Mutants**: 5 total
- 4 Caught (80%)
- 1 Missed (20%)

**Usage**: Standard test case for normal mutation testing workflow

### `all-caught/`
**Source**: Modified from `some-missed/` (all MissedMutant → CaughtMutant)
**Mutants**: 5 total
- 5 Caught (100%)
- 0 Missed (0%)

**Usage**: Test perfect mutation score (100%)

### `empty/`
**Source**: Manually created (baseline only, no mutants)
**Mutants**: 0 total

**Usage**: Test handling of projects with no mutants found

### `with-timeout/`
**Source**: Modified from `some-missed/` (first CaughtMutant → Timeout)
**Mutants**: 5 total
- 3 Caught
- 1 Missed
- 1 Timeout

**Usage**: Test timeout mutant handling

### `unviable/`
**Source**: Modified from `some-missed/` (first CaughtMutant → Unviable)
**Mutants**: 5 total
- 3 Caught
- 1 Missed
- 1 Unviable

**Usage**: Test unviable (non-compiling) mutant handling

## File Structure

Each fixture contains:
- `outcomes.json` - Required, contains all mutation test results
- Format matches cargo-mutants v25.3.1 output

## Generating New Fixtures

To create new fixtures from real projects:

```bash
# Run cargo-mutants on target project
cd /path/to/project
cargo mutants --output /tmp/new-fixture --timeout 60

# Copy outcomes.json
cp /tmp/new-fixture/outcomes.json server/tests/fixtures/cargo-mutants-output/new-fixture/

# Sanitize absolute paths if needed
sed -i 's|/home/[^/]*/|/home/user/|g' server/tests/fixtures/cargo-mutants-output/new-fixture/outcomes.json
```

## Usage in Tests

```rust
use std::path::PathBuf;
use pmat::services::mutation::json_parser::CargoMutantsReport;

#[test]
fn test_with_fixture() {
    let fixture = PathBuf::from("tests/fixtures/cargo-mutants-output/some-missed");
    let report = CargoMutantsReport::from_output_dir(&fixture).unwrap();

    assert_eq!(report.mutants.len(), 5);
    assert_eq!(report.mutation_score(), 80.0);
}
```

## Maintenance

- **Version**: Fixtures created for cargo-mutants v25.3.1
- **Date**: October 29, 2025
- **Update Policy**: Regenerate if cargo-mutants format changes significantly

## Notes

- Fixtures contain sanitized paths (no absolute user paths)
- Duration and timestamp values are real but not significant for tests
- Phase results are included for completeness but not all tests use them
