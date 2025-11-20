# Issue: Implement `pmat analyze defects` Command

**Priority**: HIGH
**Sprint**: 2
**Estimate**: 8 story points
**Labels**: enhancement, cli, known-defects

## Summary

Create new `pmat analyze defects` command to scan projects for known defect patterns with text, JSON, and JUnit output formats for CI/CD integration.

## User Story

As a **developer**
I want **to scan my entire project for known defects**
So that **I can identify and fix production defect patterns before deployment**

## Acceptance Criteria

- [ ] Command `pmat analyze defects` scans entire project
- [ ] Supports `--file <path>` flag to scan specific file
- [ ] Supports `--severity <level>` flag to filter by severity (critical, high, medium, low)
- [ ] Supports `--format <fmt>` flag with text (default), json, junit outputs
- [ ] JSON output compatible with CI/CD tools (GitHub Actions, GitLab CI)
- [ ] JUnit XML output for test result integration
- [ ] Exit code 1 if critical defects found, 0 otherwise
- [ ] Performance: >1000 files/second on modern hardware
- [ ] Test coverage ≥85% for CLI and formatters

## CLI Usage

```bash
# Scan entire project for known defects (default: text output)
$ pmat analyze defects

Known Defects Report
====================

📊 Summary
  Total Files Scanned: 542
  Files with Defects: 87
  Total Defects: 234
  Critical: 12
  High: 98
  Medium: 124
  Low: 0

🔴 CRITICAL Defects (12)

  RUST-UNWRAP-001: .unwrap() calls (12 instances)
    - src/main.rs:42
    - src/lib.rs:105
    - src/services/analyzer.rs:201
    - ... (9 more)

  Fix: Use .expect() with descriptive messages or proper error handling
  Evidence: Cloudflare outage 2025-11-18 (3+ hour network outage)

🟠 HIGH Defects (98)

  RUST-INDEXING-003: Direct array indexing (43 instances)
    - src/utils/cache.rs:78
    - ... (42 more)

  RUST-UNSAFE-004: Unsafe without SAFETY doc (55 instances)
    - src/memory/allocator.rs:12
    - ... (54 more)

🟡 MEDIUM Defects (124)

  RUST-CLONE-005: Excessive .clone() (124 instances)

Exit code: 1 (critical defects found)

# Scan specific file
$ pmat analyze defects --file src/main.rs

# Filter by severity
$ pmat analyze defects --severity critical

# JSON output for CI/CD
$ pmat analyze defects --format json > defects.json

# JUnit XML for test integration
$ pmat analyze defects --format junit > defects.xml
```

## JSON Output Format

```json
{
  "summary": {
    "total_files_scanned": 542,
    "files_with_defects": 87,
    "total_defects": 234,
    "by_severity": {
      "critical": 12,
      "high": 98,
      "medium": 124,
      "low": 0
    }
  },
  "defects": [
    {
      "id": "RUST-UNWRAP-001",
      "name": ".unwrap() calls",
      "severity": "critical",
      "language": "rust",
      "instances": [
        {
          "file": "src/main.rs",
          "line": 42,
          "column": 15,
          "code_snippet": "let config = File::open(\"config.toml\").unwrap();"
        }
      ],
      "fix_recommendation": "Use .expect() with descriptive messages or proper error handling",
      "evidence": {
        "type": "incident",
        "description": "Cloudflare outage 2025-11-18",
        "url": "https://blog.cloudflare.com/2025-01-18-outage"
      },
      "bad_example": "let x = result.unwrap();",
      "good_example": "let x = result.expect(\"Bot feature file must be valid\");"
    }
  ],
  "exit_code": 1,
  "has_critical_defects": true
}
```

## JUnit XML Output Format

```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites name="Known Defects Analysis" tests="234" failures="110" errors="0">
  <testsuite name="RUST-UNWRAP-001" tests="12" failures="12" errors="0">
    <testcase name="src/main.rs:42" classname="RUST-UNWRAP-001">
      <failure message=".unwrap() call detected">
File: src/main.rs:42
Pattern: RUST-UNWRAP-001
Severity: CRITICAL
Evidence: Cloudflare outage 2025-11-18
Fix: Use .expect() with descriptive messages
      </failure>
    </testcase>
  </testsuite>
</testsuites>
```

## Implementation Tasks

1. **CLI Handler** (2 days)
   - Create `server/src/cli/handlers/analyze_defects_handler.rs`
   - Implement file discovery and scanning
   - Add severity filtering logic
   - Implement exit code logic (1 if critical, 0 otherwise)

2. **Formatters** (2 days)
   - Text formatter with color-coded output
   - JSON formatter for CI/CD integration
   - JUnit XML formatter for test integration
   - Create `server/src/cli/formatters/defects_formatters.rs`

3. **Performance Optimization** (1 day)
   - Parallel file scanning using rayon
   - Optimize regex compilation (compile once, reuse)
   - Target: >1000 files/second

4. **Testing** (2 days)
   - Unit tests for CLI argument parsing
   - Unit tests for each formatter
   - Integration tests for end-to-end workflow
   - Property tests for edge cases

5. **Documentation** (1 day)
   - Update README.md with usage examples
   - Add CI/CD integration examples (GitHub Actions, GitLab CI)
   - Update pmat-book with `analyze defects` chapter

## Test Plan

### Unit Tests

```rust
#[test]
fn test_defects_command_critical_exit_code() {
    let result = analyze_defects(
        &project_with_unwraps,
        None,  // scan all files
        None,  // all severities
        OutputFormat::Text
    ).unwrap();

    assert_eq!(result.exit_code, 1);  // Critical defects found
    assert!(result.has_critical_defects);
}

#[test]
fn test_json_formatter() {
    let defects = vec![/* ... */];
    let json = format_defects_json(&defects).unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed["summary"]["total_defects"], 12);
}

#[test]
fn test_junit_formatter() {
    let defects = vec![/* ... */];
    let xml = format_defects_junit(&defects).unwrap();

    assert!(xml.contains("<testsuite"));
    assert!(xml.contains("RUST-UNWRAP-001"));
}
```

### Integration Tests

- Scan project with no defects → exit code 0
- Scan project with critical defects → exit code 1
- Filter by severity → only matching defects returned
- JSON output → valid JSON, parseable by jq
- JUnit output → valid XML, parseable by CI tools

## CI/CD Integration Examples

### GitHub Actions

```yaml
# .github/workflows/defects.yml
name: Known Defects Check

on: [push, pull_request]

jobs:
  defects:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install PMAT
        run: cargo install pmat

      - name: Scan for known defects
        run: pmat analyze defects --format junit > defects.xml

      - name: Publish Test Results
        uses: EnricoMi/publish-unit-test-result-action@v2
        if: always()
        with:
          files: defects.xml

      - name: Fail on critical defects
        run: pmat analyze defects --severity critical
```

### GitLab CI

```yaml
# .gitlab-ci.yml
defects:
  stage: test
  script:
    - cargo install pmat
    - pmat analyze defects --format json > defects.json
  artifacts:
    reports:
      junit: defects.xml
    paths:
      - defects.json
```

## Dependencies

- Specification: `docs/specifications/known-defects-languages-spec.md` ✅
- Issue #1: TDG integration (can be parallel)

## Success Metrics

- 100% detection rate for known defects
- <5% false positive rate
- >1000 files/second scan performance
- Zero crashes on valid Rust projects
- 85%+ test coverage

## Out of Scope (Future Work)

- Auto-fix mode (`--fix` flag) - Phase 4
- Custom defect patterns - Phase 5
- IDE integration - Phase 6

## References

- Specification: `docs/specifications/known-defects-languages-spec.md`
- Cloudflare incident: https://blog.cloudflare.com/2025-01-18-outage
