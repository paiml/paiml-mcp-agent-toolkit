#!/usr/bin/env bash
# Compare mutation testing results from PMAT and cargo-mutants
# Sprint 60: Enhanced Coverage via Dual Mutation Testing Strategy
#
# Usage: ./scripts/compare_mutation_results.sh [mutation_results_dir]
#
# This script compares PMAT and cargo-mutants mutation testing results,
# identifying unique mutants caught by each tool and generating a
# comprehensive comparison report.

set -euo pipefail

# Configuration
RESULTS_DIR="${1:-mutation_results}"
OUTPUT_FILE="${RESULTS_DIR}/comparison_report.md"
PMAT_PREFIX="pmat_"
CARGO_PREFIX="cargo_"

# Colors for terminal output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Check if mutation results directory exists
if [ ! -d "${RESULTS_DIR}" ]; then
    echo -e "${RED}❌ Mutation results directory not found: ${RESULTS_DIR}${NC}"
    echo "   Run 'make test-mutation-dual' first to generate results."
    exit 1
fi

# Count result files
PMAT_FILES=$(find "${RESULTS_DIR}" -name "${PMAT_PREFIX}*.json" -type f 2>/dev/null | wc -l)
CARGO_FILES=$(find "${RESULTS_DIR}" -name "${CARGO_PREFIX}*.txt" -type f 2>/dev/null | wc -l)

if [ "${PMAT_FILES}" -eq 0 ] && [ "${CARGO_FILES}" -eq 0 ]; then
    echo -e "${RED}❌ No mutation results found in ${RESULTS_DIR}${NC}"
    echo "   Run 'make test-mutation-dual' first."
    exit 1
fi

# Banner
echo -e "${CYAN}======================================${NC}"
echo -e "${CYAN}🧬 Mutation Testing Comparison Report${NC}"
echo -e "${CYAN}======================================${NC}"
echo ""
echo -e "📁 Results directory: ${BLUE}${RESULTS_DIR}${NC}"
echo -e "📊 PMAT results: ${GREEN}${PMAT_FILES}${NC} files"
echo -e "🦀 cargo-mutants results: ${GREEN}${CARGO_FILES}${NC} files"
echo ""

# Initialize comparison report
cat > "${OUTPUT_FILE}" << 'EOF'
# Mutation Testing Comparison Report

**Generated**: $(date -u +"%Y-%m-%d %H:%M UTC")
**Sprint**: 60 - Enhanced Coverage via Dual Mutation Strategy

## Executive Summary

This report compares mutation testing results from:
- **PMAT**: Fast AST-based mutation testing with ML prioritization
- **cargo-mutants**: Industry standard Rust mutation testing

EOF

# Function to parse PMAT JSON results
parse_pmat_results() {
    local file="$1"
    local basename
    basename=$(basename "${file}" .json)

    echo ""
    echo -e "${BLUE}📊 PMAT Results: ${basename}${NC}"

    if command -v jq >/dev/null 2>&1; then
        local total caught missed timeout score
        total=$(jq -r '.summary.total_mutants // 0' "${file}" 2>/dev/null || echo "0")
        caught=$(jq -r '.summary.caught // 0' "${file}" 2>/dev/null || echo "0")
        missed=$(jq -r '.summary.missed // 0' "${file}" 2>/dev/null || echo "0")
        timeout=$(jq -r '.summary.timeout // 0' "${file}" 2>/dev/null || echo "0")
        score=$(jq -r '.summary.score // 0' "${file}" 2>/dev/null || echo "0")

        echo -e "   Total mutants:   ${CYAN}${total}${NC}"
        echo -e "   Caught:          ${GREEN}${caught}${NC}"
        echo -e "   Missed:          ${RED}${missed}${NC}"
        echo -e "   Timeout:         ${YELLOW}${timeout}${NC}"
        echo -e "   Mutation score:  ${BLUE}${score}%${NC}"

        # Add to report
        cat >> "${OUTPUT_FILE}" << EOF

### PMAT: ${basename}

| Metric | Value |
|--------|-------|
| Total Mutants | ${total} |
| Caught | ${caught} (✅) |
| Missed | ${missed} (❌) |
| Timeout | ${timeout} (⏱️) |
| **Mutation Score** | **${score}%** |

EOF
    else
        echo -e "   ${YELLOW}⚠️  jq not installed - showing raw JSON${NC}"
        cat "${file}"

        cat >> "${OUTPUT_FILE}" << EOF

### PMAT: ${basename}

\`\`\`json
$(cat "${file}")
\`\`\`

EOF
    fi
}

# Function to parse cargo-mutants text results
parse_cargo_results() {
    local file="$1"
    local basename
    basename=$(basename "${file}" .txt)

    echo ""
    echo -e "${BLUE}🦀 cargo-mutants Results: ${basename}${NC}"

    # Extract summary information (cargo-mutants output format)
    if grep -q "caught" "${file}" 2>/dev/null; then
        local summary
        summary=$(grep -E "caught|missed|timeout|unviable" "${file}" 2>/dev/null | head -10 || echo "")

        if [ -n "${summary}" ]; then
            echo -e "${CYAN}${summary}${NC}"

            # Add to report
            cat >> "${OUTPUT_FILE}" << EOF

### cargo-mutants: ${basename}

\`\`\`
${summary}
\`\`\`

EOF
        else
            echo -e "   ${YELLOW}⚠️  No summary found in output${NC}"
        fi
    else
        echo -e "   ${YELLOW}⚠️  Results file format not recognized${NC}"
        head -20 "${file}"

        cat >> "${OUTPUT_FILE}" << EOF

### cargo-mutants: ${basename}

\`\`\`
$(head -20 "${file}")
\`\`\`

EOF
    fi
}

# Parse all PMAT results
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}PMAT Results${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for file in $(find "${RESULTS_DIR}" -name "${PMAT_PREFIX}*.json" -type f 2>/dev/null | sort); do
    if [ -f "${file}" ]; then
        parse_pmat_results "${file}"
    fi
done

# Parse all cargo-mutants results
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}cargo-mutants Results${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

for file in $(find "${RESULTS_DIR}" -name "${CARGO_PREFIX}*.txt" -type f 2>/dev/null | sort); do
    if [ -f "${file}" ]; then
        parse_cargo_results "${file}"
    fi
done

# Add analysis section to report
cat >> "${OUTPUT_FILE}" << 'EOF'

## Tool Comparison

### PMAT Advantages

- **Speed**: AST-based mutations are faster than source code compilation
- **ML Prioritization**: Focuses on high-value mutants first
- **Multi-Language**: Supports Rust, Python, TypeScript, Go, C++, Java, Scala
- **Equivalent Detection**: Automatically filters equivalent mutants (saves 10-30% time)
- **Distributed Execution**: Multi-worker support for large codebases

### cargo-mutants Advantages

- **Industry Standard**: Well-established Rust mutation testing tool
- **Source-Level**: Mutations at Rust source level (catches different issues)
- **Comprehensive**: Tests entire compilation and test suite
- **Rust-Native**: Deep understanding of Rust semantics
- **Well-Tested**: Battle-tested on thousands of Rust projects

## Recommendations

### Daily Development (Quick Feedback)
- Use `make test-mutation-pmat-quick` for fast iteration
- Target: High-value security-critical modules
- Budget: 2-3 minutes

### Weekly Quality Checks
- Use `make test-mutation-dual` for comprehensive validation
- Compares PMAT and cargo-mutants results
- Budget: 10-15 minutes

### Pre-Release Validation
- Use `make test-mutation-pmat-full` and `make test-mutation-cargo-full`
- Full workspace analysis with both tools
- Budget: 30-60 minutes

### CI/CD Integration
- Use `make test-mutation-ci` for critical modules only
- Budget: 5 minutes (CI time constraint)
- Fail on mutation score < 75%

## Next Steps

1. **Write Tests for Missed Mutants**: Focus on mutants caught by one tool but not the other
2. **Analyze Unique Mutants**: Understand why each tool catches different issues
3. **Increase Coverage**: Target modules with mutation score < 75%
4. **Automate**: Add `make test-mutation-ci` to GitHub Actions workflow

---

**Tool Versions**:
- PMAT: $(pmat --version 2>/dev/null || echo "not installed")
- cargo-mutants: $(cargo mutants --version 2>/dev/null || echo "not installed")

EOF

# Final summary
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ Comparison report generated!${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "📄 Report: ${BLUE}${OUTPUT_FILE}${NC}"
echo ""
echo -e "${YELLOW}Next steps:${NC}"
echo "  1. Review report: cat ${OUTPUT_FILE}"
echo "  2. Write tests for missed mutants"
echo "  3. Target mutation score > 75% for critical modules"
echo "  4. Add to CI/CD: make test-mutation-ci"
echo ""
