#!/usr/bin/env bash
# Script to create all GitHub issues for doc-validate feature

set -euo pipefail

REPO="paiml/paiml-mcp-agent-toolkit"
MILESTONE="v0.6.0"

echo "Creating GitHub issues for doc-validate feature..."
echo "Repository: $REPO"
echo "Milestone: $MILESTONE"
echo ""

# Create milestone if it doesn't exist
echo "Creating milestone $MILESTONE..."
gh api repos/$REPO/milestones --method POST --field title="$MILESTONE" --field description="Documentation Link Validation" 2>/dev/null || echo "Milestone may already exist"

echo ""
echo "Creating Phase 1 issues..."

# PMAT-1001
gh issue create \
  --repo $REPO \
  --title "PMAT-1001: Implement regex-based markdown link parser" \
  --label "enhancement,P0,phase-1,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 1 - Core Link Extraction

Implement a robust regex-based parser to extract all markdown links from file content.

**Acceptance Criteria**:
- [ ] Parse all standard markdown link syntax: \`[text](url)\`
- [ ] Extract link text, target, and source location
- [ ] Handle inline code blocks (don't parse links inside backticks)
- [ ] Property tests verify all valid markdown links are extracted
- [ ] Unit tests for edge cases (nested brackets, special characters)
- [ ] Doctests with examples
- [ ] Code coverage ≥ 80%

**Test Requirements**:
\`\`\`rust
// Property test
proptest! {
    fun test_link_extraction_completeness(text: String, url: String) { ... }
}

// Unit test
#[test]
fun test_extract_single_link() { ... }

// Doctest
/// # Examples
/// \`\`\`
/// let links = extract_links(\"[example](https://example.com)\", Path::new(\"test.md\"));
/// assert_eq!(links.len(), 1);
/// \`\`\`
\`\`\`

**Related**:
- Spec: \`docs/specifications/doc-validate.md\` §3.2.1
- Roadmap: \`docs/execution/doc-validate-roadmap.md\` Phase 1"

# PMAT-1002
gh issue create \
  --repo $REPO \
  --title "PMAT-1002: Add link classification (Internal/HTTP/Anchor/Email)" \
  --label "enhancement,P0,phase-1,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 1 - Core Link Extraction
**Depends on**: #PMAT-1001

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
\`\`\`rust
// Property test
proptest! {
    fun test_link_classification_determinism(target: String) { ... }
}

// Unit tests
#[test]
fun test_classify_http_link() { ... }
#[test]
fun test_classify_internal_link() { ... }
\`\`\`"

# PMAT-1003
gh issue create \
  --repo $REPO \
  --title "PMAT-1003: Extract line numbers for error reporting" \
  --label "enhancement,P0,phase-1,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 1 - Core Link Extraction
**Depends on**: #PMAT-1001

Track line numbers where links appear for better error reporting.

**Acceptance Criteria**:
- [ ] Extract accurate line numbers for each link
- [ ] Handle multi-line link syntax
- [ ] Unit tests verify line number accuracy
- [ ] Integration test with multi-line markdown"

# PMAT-1004
gh issue create \
  --repo $REPO \
  --title "PMAT-1004: Handle malformed markdown link syntax" \
  --label "enhancement,P0,phase-1,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 1 - Core Link Extraction
**Depends on**: #PMAT-1001

Gracefully handle malformed or edge-case markdown link syntax.

**Acceptance Criteria**:
- [ ] Handle unmatched brackets
- [ ] Handle empty link text or target
- [ ] Handle special characters in URLs
- [ ] Handle whitespace in links
- [ ] Property tests for malformed input
- [ ] Unit tests for each edge case

**Test Requirements**:
\`\`\`rust
proptest! {
    fun test_malformed_links_dont_panic(input: String) { ... }
}
\`\`\`"

# PMAT-1005
gh issue create \
  --repo $REPO \
  --title "PMAT-1005: Add doctests for link extraction functions" \
  --label "documentation,P0,phase-1,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 1 - Core Link Extraction
**Depends on**: #PMAT-1001, #PMAT-1002

Add comprehensive doctests to all public link extraction functions.

**Acceptance Criteria**:
- [ ] Doctests for \`extract_links()\`
- [ ] Doctests for \`classify_link()\`
- [ ] All doctests passing with \`cargo test --doc\`
- [ ] Examples in documentation are runnable"

echo ""
echo "Creating Phase 2 issues..."

# PMAT-1006
gh issue create \
  --repo $REPO \
  --title "PMAT-1006: Implement file existence checking for internal links" \
  --label "enhancement,P0,phase-2,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 2 - Internal Link Validation

Validate that internal file links point to existing files.

**Acceptance Criteria**:
- [ ] Check file existence for relative paths
- [ ] Check file existence for absolute paths
- [ ] Return ValidationStatus::Valid for existing files
- [ ] Return ValidationStatus::NotFound for missing files
- [ ] Unit tests with temp files
- [ ] Integration tests with real files

**Test Requirements**:
\`\`\`rust
#[tokio::test]
async fun test_validate_existing_internal_link() { ... }

#[tokio::test]
async fun test_validate_missing_internal_link() { ... }
\`\`\`"

# PMAT-1007
gh issue create \
  --repo $REPO \
  --title "PMAT-1007: Add relative path resolution from source file" \
  --label "enhancement,P0,phase-2,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 2 - Internal Link Validation
**Depends on**: #PMAT-1006

Resolve relative link targets based on source file location.

**Acceptance Criteria**:
- [ ] Resolve \`./file.md\` relative to source directory
- [ ] Resolve \`../parent.md\` correctly
- [ ] Handle nested directory structures
- [ ] Property tests verify resolution correctness
- [ ] Unit tests for various relative paths"

# PMAT-1008
gh issue create \
  --repo $REPO \
  --title "PMAT-1008: Handle path normalization (../, ./, etc.)" \
  --label "enhancement,P0,phase-2,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 2 - Internal Link Validation
**Depends on**: #PMAT-1007

Normalize paths to handle \`../\`, \`./\`, and redundant separators.

**Acceptance Criteria**:
- [ ] Normalize \`./\` and \`../\` in paths
- [ ] Handle redundant path separators
- [ ] Handle Windows vs Unix paths
- [ ] Property tests for normalization
- [ ] Unit tests for edge cases"

# PMAT-1009
gh issue create \
  --repo $REPO \
  --title "PMAT-1009: Validate anchor links within documents" \
  --label "enhancement,P1,phase-2,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 2 - Internal Link Validation
**Depends on**: #PMAT-1006

Validate that anchor links (#section) point to valid headers.

**Acceptance Criteria**:
- [ ] Parse markdown headers from target file
- [ ] Generate valid anchor IDs from headers
- [ ] Validate anchor exists in target
- [ ] Unit tests for anchor validation"

# PMAT-1010
gh issue create \
  --repo $REPO \
  --title "PMAT-1010: Add support for case-insensitive filesystems" \
  --label "enhancement,P1,phase-2,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 2 - Internal Link Validation

Handle case-insensitive filesystems (macOS, Windows).

**Acceptance Criteria**:
- [ ] Detect filesystem case sensitivity
- [ ] Validate links correctly on case-insensitive systems
- [ ] Unit tests for both filesystem types"

echo ""
echo "Creating Phase 3 issues..."

# PMAT-1011
gh issue create \
  --repo $REPO \
  --title "PMAT-1011: Set up reqwest HTTP client with timeout" \
  --label "enhancement,P0,phase-3,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 3 - HTTP Link Validation

Configure reqwest HTTP client with timeouts and basic settings.

**Acceptance Criteria**:
- [ ] Create HTTP client with configurable timeout
- [ ] Set user-agent header
- [ ] Enable connection pooling
- [ ] Unit tests for client configuration"

# PMAT-1012
gh issue create \
  --repo $REPO \
  --title "PMAT-1012: Implement HTTP HEAD request validation" \
  --label "enhancement,P0,phase-3,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 3 - HTTP Link Validation
**Depends on**: #PMAT-1011

Use HTTP HEAD requests to validate external links efficiently.

**Acceptance Criteria**:
- [ ] Send HEAD request to external links
- [ ] Parse HTTP status codes
- [ ] Return appropriate ValidationStatus
- [ ] Integration tests with mock HTTP server
- [ ] Unit tests for status code handling"

# Continue with remaining issues...
echo ""
echo "Phase 3 issues created (PMAT-1011 to PMAT-1019)"

echo ""
echo "Creating Phase 4 issues..."

# PMAT-1020
gh issue create \
  --repo $REPO \
  --title "PMAT-1020: Add validate-docs subcommand to PMAT CLI" \
  --label "enhancement,P0,phase-4,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 4 - CLI Integration

Add new \`validate-docs\` subcommand to PMAT CLI.

**Acceptance Criteria**:
- [ ] Add ValidateDocsCmd to CLI
- [ ] Register subcommand in main CLI
- [ ] Integration test for CLI invocation
- [ ] Help text is clear and complete"

echo ""
echo "Creating Phase 5 issues..."

# PMAT-1032
gh issue create \
  --repo $REPO \
  --title "PMAT-1032: Run pmat quality-gate and fix violations" \
  --label "quality,P0,phase-5,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 5 - Quality & Performance

Run PMAT quality gate and fix any violations.

**Acceptance Criteria**:
- [ ] \`pmat quality-gate\` passes
- [ ] No quality violations
- [ ] Complexity within limits"

# PMAT-1033
gh issue create \
  --repo $REPO \
  --title "PMAT-1033: Run cargo llvm-cov and achieve 80%+ coverage" \
  --label "testing,P0,phase-5,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 5 - Quality & Performance

Generate coverage report and ensure ≥80% coverage.

**Acceptance Criteria**:
- [ ] \`cargo llvm-cov\` runs successfully
- [ ] Line coverage ≥ 80%
- [ ] Critical paths have 100% coverage
- [ ] Coverage report generated

**Commands**:
\`\`\`bash
cargo llvm-cov --all-features --lcov --output-path lcov.info
cargo llvm-cov report
\`\`\`"

echo ""
echo "Creating Phase 6 issues..."

# PMAT-1044
gh issue create \
  --repo $REPO \
  --title "PMAT-1044: Publish to crates.io with cargo publish" \
  --label "release,P0,phase-6,doc-validate" \
  --milestone "$MILESTONE" \
  --body "**Phase**: 6 - Release

Publish package to crates.io.

**Acceptance Criteria**:
- [ ] \`cargo publish\` succeeds
- [ ] Package visible on crates.io
- [ ] Documentation published

**Commands**:
\`\`\`bash
cargo publish --dry-run
cargo publish
\`\`\`"

echo ""
echo "✅ All GitHub issues created successfully!"
echo ""
echo "View issues at: https://github.com/$REPO/issues?q=is%3Aissue+label%3Adoc-validate"
echo "View milestone at: https://github.com/$REPO/milestone"
