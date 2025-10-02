# Documentation Link Validator - GitHub Issues

This file contains all GitHub issue templates for the doc-validate feature implementation.

---

## Phase 1: Core Link Extraction

### PMAT-1001: Implement regex-based markdown link parser

**Labels**: enhancement, P0, phase-1
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Implement a robust regex-based parser to extract all markdown links from file content.

**Acceptance Criteria**:
- [ ] Parse all standard markdown link syntax: `[text](url)`
- [ ] Extract link text, target, and source location
- [ ] Handle inline code blocks (don't parse links inside backticks)
- [ ] Property tests verify all valid markdown links are extracted
- [ ] Unit tests for edge cases (nested brackets, special characters)
- [ ] Doctests with examples
- [ ] Code coverage ≥ 80%

**Test Requirements**:
```rust
// Property test
proptest! {
    fun test_link_extraction_completeness(text: String, url: String) { ... }
}

// Unit test
#[test]
fun test_extract_single_link() { ... }

// Doctest
/// # Examples
/// ```
/// let links = extract_links("[example](https://example.com)", Path::new("test.md"));
/// assert_eq!(links.len(), 1);
/// ```
```

**Related**:
- Spec: `docs/specifications/doc-validate.md` §3.2.1
- Roadmap: `docs/execution/doc-validate-roadmap.md` Phase 1

---

### PMAT-1002: Add link classification (Internal/HTTP/Anchor/Email)

**Labels**: enhancement, P0, phase-1
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Classify extracted links into types: Internal, ExternalHttp, Anchor, Email, Other.

**Acceptance Criteria**:
- [ ] Classify HTTP/HTTPS links as ExternalHttp
- [ ] Classify file paths as Internal
- [ ] Classify #anchor as Anchor
- [ ] Classify mailto: as Email
- [ ] Property tests verify classification determinism
- [ ] Unit tests for all link types
- [ ] Doctests with examples
- [ ] Code coverage ≥ 80%

**Test Requirements**:
```rust
// Property test
proptest! {
    fun test_link_classification_determinism(target: String) { ... }
}

// Unit tests
#[test]
fun test_classify_http_link() { ... }
#[test]
fun test_classify_internal_link() { ... }
```

**Related**:
- Depends on: PMAT-1001
- Spec: `docs/specifications/doc-validate.md` §3.2.1

---

### PMAT-1003: Extract line numbers for error reporting

**Labels**: enhancement, P0, phase-1
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Track line numbers where links appear for better error reporting.

**Acceptance Criteria**:
- [ ] Extract accurate line numbers for each link
- [ ] Handle multi-line link syntax
- [ ] Unit tests verify line number accuracy
- [ ] Integration test with multi-line markdown

**Related**:
- Depends on: PMAT-1001

---

### PMAT-1004: Handle malformed markdown link syntax

**Labels**: enhancement, P0, phase-1
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Gracefully handle malformed or edge-case markdown link syntax.

**Acceptance Criteria**:
- [ ] Handle unmatched brackets
- [ ] Handle empty link text or target
- [ ] Handle special characters in URLs
- [ ] Handle whitespace in links
- [ ] Property tests for malformed input
- [ ] Unit tests for each edge case

**Test Requirements**:
```rust
proptest! {
    fun test_malformed_links_dont_panic(input: String) { ... }
}
```

**Related**:
- Depends on: PMAT-1001

---

### PMAT-1005: Add doctests for link extraction functions

**Labels**: documentation, P0, phase-1
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Add comprehensive doctests to all public link extraction functions.

**Acceptance Criteria**:
- [ ] Doctests for `extract_links()`
- [ ] Doctests for `classify_link()`
- [ ] All doctests passing with `cargo test --doc`
- [ ] Examples in documentation are runnable

**Related**:
- Depends on: PMAT-1001, PMAT-1002

---

## Phase 2: Internal Link Validation

### PMAT-1006: Implement file existence checking for internal links

**Labels**: enhancement, P0, phase-2
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Validate that internal file links point to existing files.

**Acceptance Criteria**:
- [ ] Check file existence for relative paths
- [ ] Check file existence for absolute paths
- [ ] Return ValidationStatus::Valid for existing files
- [ ] Return ValidationStatus::NotFound for missing files
- [ ] Unit tests with temp files
- [ ] Integration tests with real files

**Test Requirements**:
```rust
#[tokio::test]
async fun test_validate_existing_internal_link() { ... }

#[tokio::test]
async fun test_validate_missing_internal_link() { ... }
```

**Related**:
- Spec: `docs/specifications/doc-validate.md` §3.2.2

---

### PMAT-1007: Add relative path resolution from source file

**Labels**: enhancement, P0, phase-2
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Resolve relative link targets based on source file location.

**Acceptance Criteria**:
- [ ] Resolve `./file.md` relative to source directory
- [ ] Resolve `../parent.md` correctly
- [ ] Handle nested directory structures
- [ ] Property tests verify resolution correctness
- [ ] Unit tests for various relative paths

**Test Requirements**:
```rust
proptest! {
    fun test_relative_path_resolution(filename: String) { ... }
}
```

**Related**:
- Depends on: PMAT-1006

---

### PMAT-1008: Handle path normalization (../, ./, etc.)

**Labels**: enhancement, P0, phase-2
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Normalize paths to handle `../`, `./`, and redundant separators.

**Acceptance Criteria**:
- [ ] Normalize `./` and `../` in paths
- [ ] Handle redundant path separators
- [ ] Handle Windows vs Unix paths
- [ ] Property tests for normalization
- [ ] Unit tests for edge cases

**Related**:
- Depends on: PMAT-1007

---

### PMAT-1009: Validate anchor links within documents

**Labels**: enhancement, P1, phase-2
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Validate that anchor links (#section) point to valid headers.

**Acceptance Criteria**:
- [ ] Parse markdown headers from target file
- [ ] Generate valid anchor IDs from headers
- [ ] Validate anchor exists in target
- [ ] Unit tests for anchor validation

**Related**:
- Depends on: PMAT-1006

---

### PMAT-1010: Add support for case-insensitive filesystems

**Labels**: enhancement, P1, phase-2
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Handle case-insensitive filesystems (macOS, Windows).

**Acceptance Criteria**:
- [ ] Detect filesystem case sensitivity
- [ ] Validate links correctly on case-insensitive systems
- [ ] Unit tests for both filesystem types

---

## Phase 3: HTTP Link Validation

### PMAT-1011: Set up reqwest HTTP client with timeout

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Configure reqwest HTTP client with timeouts and basic settings.

**Acceptance Criteria**:
- [ ] Create HTTP client with configurable timeout
- [ ] Set user-agent header
- [ ] Enable connection pooling
- [ ] Unit tests for client configuration

**Related**:
- Spec: `docs/specifications/doc-validate.md` §3.2.2

---

### PMAT-1012: Implement HTTP HEAD request validation

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Use HTTP HEAD requests to validate external links efficiently.

**Acceptance Criteria**:
- [ ] Send HEAD request to external links
- [ ] Parse HTTP status codes
- [ ] Return appropriate ValidationStatus
- [ ] Integration tests with mock HTTP server
- [ ] Unit tests for status code handling

**Test Requirements**:
```rust
#[tokio::test]
async fun test_validate_http_200() { ... }

#[tokio::test]
async fun test_validate_http_404() { ... }
```

**Related**:
- Depends on: PMAT-1011

---

### PMAT-1013: Add retry logic with exponential backoff

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Implement retry logic with exponential backoff for transient failures.

**Acceptance Criteria**:
- [ ] Retry on network errors up to max_retries
- [ ] Exponential backoff between retries
- [ ] Property tests verify backoff increases
- [ ] Unit tests for retry logic
- [ ] Integration tests with flaky mock server

**Test Requirements**:
```rust
proptest! {
    fun test_exponential_backoff(base_delay: u64, retry: u32) { ... }
}

#[tokio::test]
async fun test_http_retry_logic() { ... }
```

**Related**:
- Depends on: PMAT-1012

---

### PMAT-1014: Implement concurrent HTTP request handling

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Validate multiple HTTP links concurrently for performance.

**Acceptance Criteria**:
- [ ] Use tokio to validate links concurrently
- [ ] Limit concurrent requests (configurable)
- [ ] Handle results from all concurrent requests
- [ ] Integration tests with 100+ links
- [ ] Benchmark concurrent vs sequential

**Test Requirements**:
```rust
#[tokio::test]
async fun test_concurrent_validation() { ... }
```

**Related**:
- Depends on: PMAT-1012

---

### PMAT-1015: Add connection pooling and keep-alive

**Labels**: enhancement, P1, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Optimize HTTP performance with connection pooling.

**Acceptance Criteria**:
- [ ] Enable connection pooling in reqwest
- [ ] Configure keep-alive settings
- [ ] Performance tests show improvement

**Related**:
- Depends on: PMAT-1011

---

### PMAT-1016: Handle HTTP redirects (301, 302, 307, 308)

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Follow HTTP redirects and validate final destination.

**Acceptance Criteria**:
- [ ] Configure redirect following in reqwest
- [ ] Limit max redirects
- [ ] Unit tests for redirect handling
- [ ] Integration tests with redirect server

**Related**:
- Depends on: PMAT-1012

---

### PMAT-1017: Detect and report 404 errors

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Specifically detect and report HTTP 404 errors.

**Acceptance Criteria**:
- [ ] Return ValidationStatus::NotFound for 404
- [ ] Include HTTP status code in result
- [ ] Unit tests for 404 detection
- [ ] Integration tests with mock 404 server

**Related**:
- Depends on: PMAT-1012

---

### PMAT-1018: Handle network errors gracefully

**Labels**: enhancement, P0, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Handle network errors (timeout, connection refused, DNS failure).

**Acceptance Criteria**:
- [ ] Return ValidationStatus::NetworkError
- [ ] Include error message in result
- [ ] Unit tests for different error types
- [ ] Integration tests with unreachable hosts

**Related**:
- Depends on: PMAT-1012

---

### PMAT-1019: Add user-agent and custom headers support

**Labels**: enhancement, P1, phase-3
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Support custom user-agent and HTTP headers.

**Acceptance Criteria**:
- [ ] Configurable user-agent string
- [ ] Support for custom headers
- [ ] Unit tests for header configuration

**Related**:
- Depends on: PMAT-1011

---

## Phase 4: CLI Integration

### PMAT-1020: Add `validate-docs` subcommand to PMAT CLI

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Add new `validate-docs` subcommand to PMAT CLI.

**Acceptance Criteria**:
- [ ] Add ValidateDocsCmd to CLI
- [ ] Register subcommand in main CLI
- [ ] Integration test for CLI invocation
- [ ] Help text is clear and complete

**Related**:
- Spec: `docs/specifications/doc-validate.md` §3.3

---

### PMAT-1021: Implement command-line argument parsing

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Parse command-line arguments using clap.

**Acceptance Criteria**:
- [ ] `--root` for root directory
- [ ] `--config` for config file
- [ ] `--fail-on-error` flag
- [ ] `--output` for format selection
- [ ] `--max-concurrent` for concurrency limit
- [ ] `--timeout` for HTTP timeout
- [ ] Unit tests for argument parsing

**Related**:
- Depends on: PMAT-1020

---

### PMAT-1022: Add text output formatter

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Format validation results as human-readable text.

**Acceptance Criteria**:
- [ ] Summary with counts (total, valid, broken)
- [ ] List broken links with file:line
- [ ] Color-coded output (green/red)
- [ ] Unit tests for text formatting

**Related**:
- Depends on: PMAT-1020

---

### PMAT-1023: Add JSON output formatter

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Format validation results as JSON for programmatic consumption.

**Acceptance Criteria**:
- [ ] Serialize ValidationSummary to JSON
- [ ] Include all result details
- [ ] Valid JSON schema
- [ ] Unit tests for JSON output

**Related**:
- Depends on: PMAT-1020

---

### PMAT-1024: Add JUnit XML output formatter for CI

**Labels**: enhancement, P1, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Format validation results as JUnit XML for CI integration.

**Acceptance Criteria**:
- [ ] Generate valid JUnit XML
- [ ] Map broken links to test failures
- [ ] Unit tests for XML output

**Related**:
- Depends on: PMAT-1020

---

### PMAT-1025: Implement configuration file support (.toml)

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Support loading configuration from .toml file.

**Acceptance Criteria**:
- [ ] Parse .pmat/doc-validator.toml
- [ ] Override with CLI arguments
- [ ] Unit tests for config parsing
- [ ] Integration tests with config file

**Related**:
- Spec: `docs/specifications/doc-validate.md` §7.1

---

### PMAT-1026: Add exclude patterns (glob support)

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Support excluding files/directories with glob patterns.

**Acceptance Criteria**:
- [ ] Parse glob patterns from config
- [ ] Exclude matching files during scan
- [ ] Property tests for glob matching
- [ ] Unit tests for exclusion logic

**Related**:
- Depends on: PMAT-1025

---

### PMAT-1027: Add progress indicator for long validations

**Labels**: enhancement, P1, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Show progress during long validation runs.

**Acceptance Criteria**:
- [ ] Display progress bar or percentage
- [ ] Update as files are processed
- [ ] Manual testing for UX

**Related**:
- Depends on: PMAT-1020

---

### PMAT-1028: Exit with error code on broken links

**Labels**: enhancement, P0, phase-4
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Exit with non-zero code when broken links found (for CI).

**Acceptance Criteria**:
- [ ] Exit code 1 when broken links found
- [ ] Exit code 0 when all links valid
- [ ] Respect `--fail-on-error` flag
- [ ] Integration test for exit codes

**Related**:
- Depends on: PMAT-1020

---

## Phase 5: Quality & Performance

### PMAT-1029: Run all property tests and verify coverage

**Labels**: testing, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Run all property tests and ensure they pass.

**Acceptance Criteria**:
- [ ] All property tests passing
- [ ] Property tests cover key invariants
- [ ] Proptest runs with sufficient iterations

---

### PMAT-1030: Run all unit tests and integration tests

**Labels**: testing, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Verify all unit and integration tests pass.

**Acceptance Criteria**:
- [ ] `cargo test` passes 100%
- [ ] No flaky tests
- [ ] All integration tests pass

---

### PMAT-1031: Run all doctests

**Labels**: testing, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Verify all documentation examples work.

**Acceptance Criteria**:
- [ ] `cargo test --doc` passes 100%
- [ ] All examples are correct and runnable

---

### PMAT-1032: Run `pmat quality-gate` and fix violations

**Labels**: quality, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Run PMAT quality gate and fix any violations.

**Acceptance Criteria**:
- [ ] `pmat quality-gate` passes
- [ ] No quality violations
- [ ] Complexity within limits

---

### PMAT-1033: Run `cargo llvm-cov` and achieve 80%+ coverage

**Labels**: testing, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Generate coverage report and ensure ≥80% coverage.

**Acceptance Criteria**:
- [ ] `cargo llvm-cov` runs successfully
- [ ] Line coverage ≥ 80%
- [ ] Critical paths have 100% coverage
- [ ] Coverage report generated

**Commands**:
```bash
cargo llvm-cov --all-features --lcov --output-path lcov.info
cargo llvm-cov report
```

---

### PMAT-1034: Add benchmark suite with criterion

**Labels**: performance, P1, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Create benchmarks for performance-critical paths.

**Acceptance Criteria**:
- [ ] Benchmark link extraction
- [ ] Benchmark concurrent HTTP validation
- [ ] Benchmark full directory validation
- [ ] Document baseline performance

---

### PMAT-1035: Profile and optimize hot paths

**Labels**: performance, P1, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Profile code and optimize performance bottlenecks.

**Acceptance Criteria**:
- [ ] Profile with cargo-flamegraph
- [ ] Identify hot paths
- [ ] Optimize if needed
- [ ] Re-benchmark after optimization

---

### PMAT-1036: Run clippy and fix all warnings

**Labels**: quality, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Run clippy and fix all warnings.

**Acceptance Criteria**:
- [ ] `cargo clippy -- -D warnings` passes
- [ ] No clippy warnings remain

---

### PMAT-1037: Run rustfmt on all code

**Labels**: quality, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Format all code with rustfmt.

**Acceptance Criteria**:
- [ ] `cargo fmt` applied to all files
- [ ] `cargo fmt -- --check` passes

---

### PMAT-1038: Update documentation and examples

**Labels**: documentation, P0, phase-5
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Complete all documentation and examples.

**Acceptance Criteria**:
- [ ] All public APIs documented
- [ ] README.md updated with usage
- [ ] Specification up to date
- [ ] Examples are comprehensive

---

## Phase 6: Release

### PMAT-1039: Bump version to 0.6.0 in Cargo.toml

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Update version number in Cargo.toml.

**Acceptance Criteria**:
- [ ] Version updated to 0.6.0
- [ ] Committed to master

---

### PMAT-1040: Update CHANGELOG.md with release notes

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Write comprehensive release notes.

**Acceptance Criteria**:
- [ ] CHANGELOG.md updated with v0.6.0 section
- [ ] Lists all new features
- [ ] Lists breaking changes (if any)
- [ ] Committed to master

---

### PMAT-1041: Update README.md with validate-docs usage

**Labels**: documentation, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Add validate-docs command to README.

**Acceptance Criteria**:
- [ ] README.md includes validate-docs section
- [ ] Usage examples provided
- [ ] Committed to master

---

### PMAT-1042: Run final quality checks before release

**Labels**: quality, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Run all quality checks one final time.

**Acceptance Criteria**:
- [ ] All tests passing
- [ ] Quality gate passing
- [ ] Coverage ≥ 80%
- [ ] No clippy warnings
- [ ] Code formatted

---

### PMAT-1043: Create git tag for v0.6.0

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Create annotated git tag for release.

**Acceptance Criteria**:
- [ ] Tag created: `git tag -a v0.6.0 -m "Release v0.6.0"`
- [ ] Tag pushed: `git push origin v0.6.0`

---

### PMAT-1044: Publish to crates.io with `cargo publish`

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Publish package to crates.io.

**Acceptance Criteria**:
- [ ] `cargo publish` succeeds
- [ ] Package visible on crates.io
- [ ] Documentation published

**Commands**:
```bash
cargo publish --dry-run
cargo publish
```

---

### PMAT-1045: Create GitHub release with notes

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Create GitHub release from tag.

**Acceptance Criteria**:
- [ ] GitHub release created for v0.6.0
- [ ] Release notes from CHANGELOG included
- [ ] Binary artifacts attached (if applicable)

**Commands**:
```bash
gh release create v0.6.0 --title "v0.6.0 - Documentation Link Validation" --notes-file CHANGELOG.md
```

---

### PMAT-1046: Push commits and tags to GitHub

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Push all commits and tags to GitHub.

**Acceptance Criteria**:
- [ ] `git push origin master` succeeds
- [ ] `git push origin v0.6.0` succeeds
- [ ] All changes visible on GitHub

---

### PMAT-1047: Verify crates.io publication

**Labels**: release, P0, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Verify package is correctly published.

**Acceptance Criteria**:
- [ ] Package visible on crates.io
- [ ] `cargo install pmat` works
- [ ] Documentation renders correctly

---

### PMAT-1048: Announce release (Discord/Twitter/Docs)

**Labels**: release, P2, phase-6
**Assignee**: TBD
**Milestone**: v0.6.0

**Description**:
Announce the release to community.

**Acceptance Criteria**:
- [ ] Community announcement posted
- [ ] Documentation site updated
