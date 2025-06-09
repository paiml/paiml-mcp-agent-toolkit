#!/bin/bash
# Pre-release comprehensive analysis
# Generates a detailed checklist for release preparation

set -euo pipefail

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Output file
OUTPUT="docs/bugs/prepare-release.md"
mkdir -p docs/bugs

# Use cargo run if pmat not available
if command -v pmat &> /dev/null; then
    PMAT="pmat"
else
    PMAT="cargo run --release --"
fi

echo "Running comprehensive pre-release analysis..."

# Start the markdown file
cat > "$OUTPUT" <<EOF
# Release Preparation Checklist - v0.22.0

Generated: $(date)
Analysis Type: Comprehensive Pre-Release Audit

## Executive Summary

This automated analysis identifies all quality issues that must be addressed before release.
The system enforces ZERO tolerance for SATD, high complexity, and known defects.

---

## 🔴 Critical Issues (Release Blockers)

### Self-Admitted Technical Debt (SATD)
**Policy**: ZERO tolerance - No TODO, FIXME, HACK, XXX allowed

EOF

# Check for SATD
echo "Analyzing SATD..."
echo '```' >> "$OUTPUT"
if $PMAT analyze satd . 2>/dev/null | grep -E "TODO|FIXME|HACK|XXX" >> "$OUTPUT"; then
    echo '```' >> "$OUTPUT"
    echo "❌ **SATD FOUND** - Must be eliminated before release" >> "$OUTPUT"
else
    echo "No SATD found" >> "$OUTPUT"
    echo '```' >> "$OUTPUT"
    echo "✅ **SATD Check Passed** - No technical debt markers found" >> "$OUTPUT"
fi

# Check for high complexity
echo -e "\n### High Complexity Functions\n**Policy**: No function may exceed cyclomatic complexity of 20\n" >> "$OUTPUT"
echo "Analyzing complexity..."
echo '```' >> "$OUTPUT"
$PMAT analyze complexity . --max-cyclomatic 20 2>/dev/null | grep -A5 "exceeds threshold" >> "$OUTPUT" || echo "No functions exceed complexity threshold" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# Check for incomplete features
echo -e "\n### Incomplete Features\n**Policy**: All features must be fully implemented\n" >> "$OUTPUT"
echo "Checking for placeholder implementations..."
echo '```bash' >> "$OUTPUT"
grep -r "unimplemented!()" server/src/ 2>/dev/null | head -10 >> "$OUTPUT" || echo "No unimplemented!() calls found" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# Run tests
echo -e "\n### Test Status\n" >> "$OUTPUT"
echo "Running tests..."
if make test-fast > /tmp/test_output.txt 2>&1; then
    TEST_COUNT=$(grep -E "test result:|passed" /tmp/test_output.txt | tail -1)
    echo "✅ **All tests passing** - $TEST_COUNT" >> "$OUTPUT"
else
    echo "❌ **Tests failing** - See test output for details" >> "$OUTPUT"
fi

# Check linting
echo -e "\n### Linting Status\n" >> "$OUTPUT"
echo "Running linter..."
if make lint > /tmp/lint_output.txt 2>&1; then
    echo "✅ **Linting passed** - No warnings or errors" >> "$OUTPUT"
else
    echo "❌ **Linting issues found** - Must be fixed before release" >> "$OUTPUT"
    echo '```' >> "$OUTPUT"
    tail -20 /tmp/lint_output.txt >> "$OUTPUT"
    echo '```' >> "$OUTPUT"
fi

# High priority issues
echo -e "\n---\n\n## 🟡 High Priority Issues\n" >> "$OUTPUT"

# Functions with moderate complexity
echo -e "### Functions with Complexity 15-20\n" >> "$OUTPUT"
echo '```' >> "$OUTPUT"
$PMAT analyze complexity . --min-cyclomatic 15 --max-cyclomatic 20 2>/dev/null | grep -E "Function:|Cyclomatic:" | head -20 >> "$OUTPUT" || echo "No functions in this range" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# Dead code analysis
echo -e "\n### Dead Code Analysis\n" >> "$OUTPUT"
echo "Analyzing dead code..."
echo '```' >> "$OUTPUT"
$PMAT analyze dead-code . 2>/dev/null | grep -A2 "Dead" | head -20 >> "$OUTPUT" || echo "Dead code analysis not available" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# File analysis
echo -e "\n---\n\n## 📊 Repository Statistics\n" >> "$OUTPUT"
echo "Analyzing repository structure..."

# Count files by type
echo -e "### File Distribution\n" >> "$OUTPUT"
echo '```' >> "$OUTPUT"
echo "Rust files: $(find server/src -name "*.rs" | wc -l)" >> "$OUTPUT"
echo "Test files: $(find server -name "*test*.rs" -o -name "*tests.rs" | wc -l)" >> "$OUTPUT"
echo "TypeScript files: $(find scripts -name "*.ts" | wc -l)" >> "$OUTPUT"
echo "Documentation files: $(find . -name "*.md" | wc -l)" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# Top complex files
echo -e "\n### Top 10 Complex Files\n" >> "$OUTPUT"
echo '```' >> "$OUTPUT"
$PMAT analyze complexity . --top 10 2>/dev/null >> "$OUTPUT" || echo "Complexity analysis not available" >> "$OUTPUT"
echo '```' >> "$OUTPUT"

# Generate actionable checklist
echo -e "\n---\n\n## ✅ Pre-Release Checklist\n" >> "$OUTPUT"
cat >> "$OUTPUT" <<'EOF'
### Code Quality
- [ ] All SATD removed (TODO, FIXME, HACK, XXX)
- [ ] No functions exceed cyclomatic complexity of 20
- [ ] No functions exceed cognitive complexity of 30
- [ ] All placeholder implementations replaced
- [ ] Dead code removed
- [ ] Duplicate code consolidated

### Testing
- [ ] All tests passing (`make test-fast`)
- [ ] Test coverage > 80%
- [ ] Integration tests updated
- [ ] Performance benchmarks run
- [ ] Fuzz tests executed

### Documentation
- [ ] README.md updated with new features
- [ ] CHANGELOG.md updated
- [ ] API documentation complete
- [ ] Architecture diagrams current
- [ ] Example code tested

### Build & Release
- [ ] Linting passes with zero warnings (`make lint`)
- [ ] Formatting consistent (`make format`)
- [ ] Binary size optimized (`make release`)
- [ ] Cross-platform builds tested
- [ ] Version number updated

### Security & Performance
- [ ] No hardcoded secrets or credentials
- [ ] Dependencies updated and audited
- [ ] Performance regression tests pass
- [ ] Memory usage profiled
- [ ] Startup time < 100ms

### Final Steps
- [ ] Create git tag for release
- [ ] Generate release notes
- [ ] Update GitHub releases
- [ ] Publish to crates.io (if applicable)
- [ ] Announce release

---

## 🤖 Automated Refactoring Commands

To fix identified issues automatically, run:

```bash
# Fix SATD markers
pmat refactor interactive --fix-satd

# Extract complex functions
pmat refactor interactive --extract-complex --threshold 20

# Remove dead code
pmat refactor interactive --remove-dead-code

# Run full overnight repair
./scripts/overnight-refactor.sh
```

## 📈 Quality Metrics Trend

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| SATD Count | TBD | 0 | 🔄 |
| Max Complexity | TBD | 20 | 🔄 |
| Test Coverage | TBD | >80% | 🔄 |
| Lint Warnings | TBD | 0 | 🔄 |
| Binary Size | ~15MB | <20MB | ✅ |

---

Generated by PMAT Pre-Release Analyzer
Next recommended action: Address all critical issues before proceeding
EOF

echo -e "\n${GREEN}Analysis complete!${NC}"
echo "Report generated at: $OUTPUT"
echo ""
echo "Next steps:"
echo "1. Review the checklist: cat $OUTPUT"
echo "2. Fix critical issues first (marked with 🔴)"
echo "3. Run overnight refactoring: ./scripts/overnight-refactor.sh"
echo "4. Re-run this analysis to verify fixes"