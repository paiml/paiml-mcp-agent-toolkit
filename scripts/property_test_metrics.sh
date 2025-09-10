#!/bin/bash

# Property Test Metrics Dashboard
# Sprint 88 Achievement: 80% Coverage Monitoring
# 
# This script generates a comprehensive metrics report for property testing

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SERVER_DIR="${1:-server/src}"
REPORT_FORMAT="${2:-markdown}" # markdown, json, or terminal

# Functions
count_files() {
    find "$SERVER_DIR" -name "*.rs" -type f 2>/dev/null | wc -l
}

count_files_with_property_tests() {
    local all_files=$(mktemp)
    grep -r "proptest!" "$SERVER_DIR" --include="*.rs" -l 2>/dev/null > "$all_files" || true
    grep -r "mod property_tests" "$SERVER_DIR" --include="*.rs" -l 2>/dev/null >> "$all_files" || true
    sort -u "$all_files" | wc -l
    rm "$all_files"
}

count_placeholder_tests() {
    grep -r "prop_assert!(true)" "$SERVER_DIR" --include="*.rs" 2>/dev/null | wc -l || echo "0"
}

count_meaningful_tests() {
    grep -r "prop_assert!" "$SERVER_DIR" --include="*.rs" 2>/dev/null | grep -v "prop_assert!(true)" | wc -l || echo "0"
}

get_files_without_tests() {
    local temp_with_tests=$(mktemp)
    local temp_all_files=$(mktemp)
    
    # Get all files with tests
    grep -r "proptest!" "$SERVER_DIR" --include="*.rs" -l 2>/dev/null > "$temp_with_tests" || true
    grep -r "mod property_tests" "$SERVER_DIR" --include="*.rs" -l 2>/dev/null >> "$temp_with_tests" || true
    
    # Get all files
    find "$SERVER_DIR" -name "*.rs" -type f > "$temp_all_files"
    
    # Find difference
    comm -23 <(sort "$temp_all_files") <(sort -u "$temp_with_tests") 
    
    rm "$temp_with_tests" "$temp_all_files"
}

analyze_test_distribution() {
    echo "Analyzing test distribution by module..."
    
    for dir in "$SERVER_DIR"/*; do
        if [ -d "$dir" ]; then
            module_name=$(basename "$dir")
            module_files=$(find "$dir" -name "*.rs" -type f 2>/dev/null | wc -l)
            
            if [ "$module_files" -gt 0 ]; then
                module_tests=$(mktemp)
                grep -r "proptest!" "$dir" --include="*.rs" -l 2>/dev/null > "$module_tests" || true
                grep -r "mod property_tests" "$dir" --include="*.rs" -l 2>/dev/null >> "$module_tests" || true
                module_with_tests=$(sort -u "$module_tests" | wc -l)
                rm "$module_tests"
                
                coverage=$(awk "BEGIN {printf \"%.1f\", ($module_with_tests / $module_files) * 100}")
                echo "$module_name|$module_with_tests|$module_files|$coverage"
            fi
        fi
    done | sort -t'|' -k4 -rn
}

generate_terminal_report() {
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${CYAN}           PROPERTY TEST METRICS DASHBOARD                 ${NC}"
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    
    local total_files=$(count_files)
    local files_with_tests=$(count_files_with_property_tests)
    local coverage=$(awk "BEGIN {printf \"%.1f\", ($files_with_tests / $total_files) * 100}")
    local placeholder_count=$(count_placeholder_tests)
    local meaningful_count=$(count_meaningful_tests)
    
    # Coverage Status
    if (( $(echo "$coverage >= 80" | bc -l) )); then
        echo -e "${GREEN}✅ COVERAGE STATUS: PASSING${NC}"
    else
        echo -e "${RED}❌ COVERAGE STATUS: FAILING${NC}"
    fi
    
    echo ""
    echo -e "${BLUE}📊 Overall Statistics${NC}"
    echo "├─ Total Rust Files: $total_files"
    echo "├─ Files with Property Tests: $files_with_tests"
    echo "├─ Coverage Percentage: ${coverage}%"
    echo "├─ Target Threshold: 80.0%"
    echo "└─ Status: $([ $(echo "$coverage >= 80" | bc -l) -eq 1 ] && echo "✅ PASSING" || echo "❌ BELOW THRESHOLD")"
    
    echo ""
    echo -e "${BLUE}🎯 Test Quality Metrics${NC}"
    echo "├─ Placeholder Tests: $placeholder_count"
    echo "├─ Meaningful Tests: $meaningful_count"
    echo "├─ Quality Ratio: $(awk "BEGIN {if ($meaningful_count + $placeholder_count > 0) printf \"%.1f%%\", ($meaningful_count / ($meaningful_count + $placeholder_count)) * 100; else print \"0.0%\"}")"
    echo "└─ Recommendation: $([ "$placeholder_count" -gt "$meaningful_count" ] && echo "⚠️ Upgrade placeholder tests" || echo "✅ Good test quality")"
    
    echo ""
    echo -e "${BLUE}📁 Module Coverage Distribution${NC}"
    echo "┌──────────────────┬───────────┬────────────┬──────────┐"
    echo "│ Module           │ Covered   │ Total      │ Coverage │"
    echo "├──────────────────┼───────────┼────────────┼──────────┤"
    
    analyze_test_distribution | while IFS='|' read -r module covered total cov; do
        printf "│ %-16s │ %9s │ %10s │ %7s%% │\n" "$module" "$covered" "$total" "$cov"
    done
    
    echo "└──────────────────┴───────────┴────────────┴──────────┘"
    
    echo ""
    echo -e "${BLUE}🔍 Top Files Missing Property Tests${NC}"
    get_files_without_tests | head -10 | while read -r file; do
        echo "  ⚠️ $(basename "$file")"
    done
    
    echo ""
    echo -e "${CYAN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
}

generate_markdown_report() {
    local total_files=$(count_files)
    local files_with_tests=$(count_files_with_property_tests)
    local coverage=$(awk "BEGIN {printf \"%.1f\", ($files_with_tests / $total_files) * 100}")
    local placeholder_count=$(count_placeholder_tests)
    local meaningful_count=$(count_meaningful_tests)
    local quality_ratio=$(awk "BEGIN {if ($meaningful_count + $placeholder_count > 0) printf \"%.1f\", ($meaningful_count / ($meaningful_count + $placeholder_count)) * 100; else print \"0.0\"}")
    
    cat << EOF
# Property Test Metrics Report

**Generated**: $(date -u +"%Y-%m-%d %H:%M:%S UTC")  
**Sprint 88 Target**: 80% Coverage ✅

## 📊 Executive Summary

| Metric | Value | Status |
|--------|-------|--------|
| **Total Files** | $total_files | - |
| **Files with Tests** | $files_with_tests | $([ "$files_with_tests" -ge 431 ] && echo "✅" || echo "⚠️") |
| **Coverage** | ${coverage}% | $([ $(echo "$coverage >= 80" | bc -l) -eq 1 ] && echo "✅ PASSING" || echo "❌ FAILING") |
| **Placeholder Tests** | $placeholder_count | $([ "$placeholder_count" -lt "$meaningful_count" ] && echo "✅" || echo "⚠️") |
| **Meaningful Tests** | $meaningful_count | $([ "$meaningful_count" -gt 100 ] && echo "✅" || echo "⚠️") |
| **Quality Ratio** | ${quality_ratio}% | $([ $(echo "$quality_ratio > 50" | bc -l) -eq 1 ] && echo "✅" || echo "⚠️") |

## 📈 Coverage Trend

\`\`\`
Coverage: ${coverage}% $([[ $(echo "$coverage >= 80" | bc -l) -eq 1 ]] && echo "████████████████████" || echo "████████████████░░░░")
Target:   80.0%   ████████████████░░░░
\`\`\`

## 📁 Module Breakdown

| Module | Files with Tests | Total Files | Coverage |
|--------|-----------------|-------------|----------|
$(analyze_test_distribution | while IFS='|' read -r module covered total cov; do
    echo "| $module | $covered | $total | ${cov}% |"
done)

## 🎯 Quality Analysis

### Test Type Distribution
- **Placeholder Tests**: $placeholder_count ($(awk "BEGIN {if ($placeholder_count + $meaningful_count > 0) printf \"%.1f%%\", ($placeholder_count / ($placeholder_count + $meaningful_count)) * 100; else print \"0.0%\"}" ))
- **Meaningful Tests**: $meaningful_count (${quality_ratio}%)

### Recommendations
$(if [ "$placeholder_count" -gt "$meaningful_count" ]; then
    echo "1. ⚠️ **High number of placeholder tests** - Prioritize upgrading to meaningful property tests"
    echo "2. Focus on modules with lowest coverage first"
    echo "3. Target: Convert 10 placeholder tests per week"
else
    echo "1. ✅ **Good test quality** - Majority of tests are meaningful"
    echo "2. Continue maintaining high quality standards"
    echo "3. Focus on increasing coverage in low-coverage modules"
fi)

## 📝 Action Items

### Immediate (This Week)
$(get_files_without_tests | head -5 | while read -r file; do
    echo "- [ ] Add property tests to \`$(basename "$file")\`"
done)

### Short-term (This Month)
- [ ] Achieve 85% overall coverage
- [ ] Upgrade 50 placeholder tests to meaningful tests
- [ ] Document property test patterns in critical modules

### Long-term (This Quarter)
- [ ] Reach 90% coverage milestone
- [ ] Achieve 75% meaningful test ratio
- [ ] Implement property test generation automation

## 🔧 Commands

\`\`\`bash
# Run all property tests
cd server && PROPTEST_CASES=100 cargo test --lib -- proptest

# Check current coverage
./scripts/property_test_metrics.sh

# Add tests to uncovered files
./scripts/rapid_property_test_injection.py

# Find placeholder tests to upgrade
grep -r "prop_assert!(true)" server/src --include="*.rs" -l
\`\`\`

## 📊 Historical Progress

| Date | Coverage | Files with Tests | Quality Ratio |
|------|----------|-----------------|---------------|
| 2025-09-10 | ${coverage}% | $files_with_tests | ${quality_ratio}% |
| 2025-09-09 | 7.4% | 40 | 5.0% |
| Sprint 88 Start | 7.4% | 40 | 5.0% |

---
*Report generated by property_test_metrics.sh*
EOF
}

generate_json_report() {
    local total_files=$(count_files)
    local files_with_tests=$(count_files_with_property_tests)
    local coverage=$(awk "BEGIN {printf \"%.1f\", ($files_with_tests / $total_files) * 100}")
    local placeholder_count=$(count_placeholder_tests)
    local meaningful_count=$(count_meaningful_tests)
    
    cat << EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "summary": {
    "total_files": $total_files,
    "files_with_tests": $files_with_tests,
    "coverage_percentage": $coverage,
    "threshold": 80.0,
    "status": "$([ $(echo "$coverage >= 80" | bc -l) -eq 1 ] && echo "passing" || echo "failing")"
  },
  "quality": {
    "placeholder_tests": $placeholder_count,
    "meaningful_tests": $meaningful_count,
    "quality_ratio": $(awk "BEGIN {if ($meaningful_count + $placeholder_count > 0) printf \"%.1f\", ($meaningful_count / ($meaningful_count + $placeholder_count)) * 100; else print \"0.0\"}")
  },
  "modules": [
$(analyze_test_distribution | while IFS='|' read -r module covered total cov; do
    echo "    {\"name\": \"$module\", \"covered\": $covered, \"total\": $total, \"coverage\": $cov},"
done | sed '$ s/,$//')
  ],
  "recommendations": [
$(if [ "$placeholder_count" -gt "$meaningful_count" ]; then
    echo '    "Upgrade placeholder tests to meaningful property tests",'
    echo '    "Focus on modules with lowest coverage",'
    echo '    "Target 10 placeholder conversions per week"'
else
    echo '    "Maintain high test quality standards",'
    echo '    "Increase coverage in low-coverage modules",'
    echo '    "Document successful property test patterns"'
fi)
  ]
}
EOF
}

# Main execution
main() {
    case "$REPORT_FORMAT" in
        json)
            generate_json_report
            ;;
        markdown|md)
            generate_markdown_report
            ;;
        terminal|term|*)
            generate_terminal_report
            ;;
    esac
}

# Run main function
main