#!/bin/bash
# Implementation script for scientific dependency reduction
# Based on: docs/specifications/scientifically-remove-dependencies-time-improve-compile-speed-test-speed.md
# Version: 1.0.1
# Date: 2025-11-23

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT" || exit

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}🔬 Scientific Dependency Reduction Implementation${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Step 1: Baseline Measurements
echo -e "${BLUE}📊 Step 1: Baseline Measurements${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Measure current dependency count
echo -e "${YELLOW}→ Counting dependencies...${NC}"
dep_count_before=$(cargo tree --depth 0 | wc -l)
echo -e "${GREEN}✓ Dependencies: ${dep_count_before}${NC}"

# Measure current compile time (dev build for speed)
echo -e "${YELLOW}→ Measuring compile time (dev build)...${NC}"
cargo clean -p pmat &>/dev/null
# shellcheck disable=DET002  # Timestamp needed for compile time measurement
compile_start=$(date +%s)
cargo build --lib 2>&1 | grep -E "(Compiling|Finished)" | tail -5
# shellcheck disable=DET002  # Timestamp needed for compile time measurement
compile_end=$(date +%s)
compile_time_before=$((compile_end - compile_start))
echo -e "${GREEN}✓ Compile time: ${compile_time_before}s${NC}"

# Measure binary size
echo -e "${YELLOW}→ Building release binary...${NC}"
cargo build --release --quiet 2>&1 | grep -v "warning:" || true
binary_size_before=$(stat -c%s target/release/pmat 2>/dev/null || echo "0")
binary_size_mb_before=$(bc <<< "scale=2; ${binary_size_before} / 1024 / 1024")
echo -e "${GREEN}✓ Binary size: ${binary_size_mb_before} MB${NC}"

echo ""

# Step 2: Run cargo-machete Analysis
echo -e "${BLUE}📋 Step 2: Dependency Analysis${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Check if cargo-machete is installed
if ! command -v cargo-machete &>/dev/null; then
    echo -e "${YELLOW}⚠️  cargo-machete not found. Installing...${NC}"
    cargo install cargo-machete
fi

echo -e "${YELLOW}→ Running cargo-machete analysis...${NC}"
cargo machete --with-metadata 2>&1 | tee /tmp/machete-output.log || true

# Count unused dependencies
unused_count=$(grep -E "^\s+(tree-sitter-|ahash|arc-swap)" /tmp/machete-output.log | wc -l || echo "0")
echo -e "${GREEN}✓ Found ${unused_count} potentially unused dependencies${NC}"

echo ""

# Step 3: Use PMAT for Code Analysis
echo -e "${BLUE}🔍 Step 3: PMAT Dead Code Analysis${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

echo -e "${YELLOW}→ Analyzing language modules...${NC}"
pmat analyze dead-code --path server/src/services/languages 2>&1 | tee /tmp/pmat-dead-code.log || true

echo -e "${YELLOW}→ Checking TDG scores for language modules...${NC}"
pmat analyze tdg --path server/src/services/languages 2>&1 | tee /tmp/pmat-tdg-languages.log

echo ""

# Step 4: Verify Unused Dependencies
echo -e "${BLUE}🔬 Step 4: Source Code Verification${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

for lang in c_sharp java ruby scala swift; do
    echo -e "${YELLOW}→ Checking tree-sitter-${lang} usage...${NC}"
    usage_count=$(rg "tree_sitter_${lang}::" server/src/ 2>/dev/null | wc -l || echo "0")

    if [ "$usage_count" -eq 0 ]; then
        echo -e "${GREEN}✓ tree-sitter-${lang}: 0 references (SAFE TO REMOVE)${NC}"
    else
        echo -e "${RED}✗ tree-sitter-${lang}: ${usage_count} references (UNSAFE)${NC}"
    fi
done

echo ""

# Step 5: Impact Estimation
echo -e "${BLUE}📈 Step 5: Impact Estimation${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Based on research from specification (Proksch et al. 2020)
# 10% dependency removal → 15-20% compile time reduction
estimated_reduction_pct=15
deps_to_remove=5
reduction_factor=$(bc <<< "scale=2; (${deps_to_remove} / ${dep_count_before}) * ${estimated_reduction_pct}")
estimated_compile_reduction=$(bc <<< "scale=0; ${compile_time_before} * ${reduction_factor} / 100")
estimated_compile_after=$((compile_time_before - estimated_compile_reduction))

echo -e "${YELLOW}Dependencies to remove: ${deps_to_remove}${NC}"
echo -e "${YELLOW}Current dependency count: ${dep_count_before}${NC}"
echo -e "${YELLOW}Estimated reduction factor: ${reduction_factor}%${NC}"
echo -e "${GREEN}Estimated compile time: ${compile_time_before}s → ${estimated_compile_after}s (save ${estimated_compile_reduction}s)${NC}"

echo ""

# Step 6: Generate Implementation Report
echo -e "${BLUE}📝 Step 6: Generate Implementation Report${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

cat > /tmp/dependency-reduction-report.md << EOF
# Dependency Reduction Implementation Report

$(# shellcheck disable=DET002  # Timestamp needed for report metadata
)
**Date**: $(date +"%Y-%m-%d %H:%M:%S")
**Baseline Version**: v2.202.0
**Specification**: docs/specifications/scientifically-remove-dependencies-time-improve-compile-speed-test-speed.md

## Baseline Metrics

| Metric | Value |
|--------|-------|
| Total Dependencies | ${dep_count_before} |
| Compile Time (dev) | ${compile_time_before}s |
| Binary Size | ${binary_size_mb_before} MB |
| Unused Dependencies | ${unused_count} (flagged by cargo-machete) |

## Analysis Results

### cargo-machete Findings

\`\`\`
$(cat /tmp/machete-output.log)
\`\`\`

### PMAT Dead Code Analysis

\`\`\`
$(cat /tmp/pmat-dead-code.log 2>/dev/null || echo "Analysis running...")
\`\`\`

### PMAT TDG Scores (Language Modules)

\`\`\`
$(cat /tmp/pmat-tdg-languages.log)
\`\`\`

## Source Code Verification

| Dependency | References | Status |
|------------|-----------|--------|
$(for lang in c_sharp java ruby scala swift; do
    count=$(rg "tree_sitter_${lang}::" server/src/ 2>/dev/null | wc -l || echo "0")
    if [ "$count" -eq 0 ]; then
        echo "| tree-sitter-${lang} | ${count} | ✅ Safe to remove |"
    else
        echo "| tree-sitter-${lang} | ${count} | ❌ In use |"
    fi
done)

## Impact Estimation

Based on **Proksch et al. (2020)** research:
- 10% dependency removal → 15-20% compile time reduction

**Estimated Impact**:
- Dependencies to remove: ${deps_to_remove}
- Reduction factor: ${reduction_factor}%
- Compile time: ${compile_time_before}s → ${estimated_compile_after}s (save ${estimated_compile_reduction}s)

## Recommendations

### Phase 1: Immediate Actions

1. **Feature-gate language modules** that use removed parsers
   - Move \`src/services/languages/{csharp,java,scala,swift}.rs\` behind feature flags
   - Update \`Cargo.toml\` to remove unused parser dependencies

2. **Remove truly unused dependencies** (0 references)
   - Verify each with \`rg "dependency_name" server/src/\`
   - Test compilation after each removal

3. **Measure actual impact**
   - Run this script again after changes
   - Compare compile time, binary size, test time

### Phase 2: Future Work

1. **Transitive dependency optimization**
   - Replace heavy dependencies with lighter alternatives
   - Target: <200 total dependencies

2. **Continuous monitoring**
   - Add pre-commit hook to check for new unused deps
   - Add CI/CD check for dependency count threshold

## Next Steps

\`\`\`bash
# 1. Review this report
cat /tmp/dependency-reduction-report.md

# 2. Implement changes manually or run:
#    (Not automated - requires careful review)

# 3. Measure impact
./scripts/implement-dependency-reduction.sh

# 4. Commit changes with evidence
git commit -m "refactor: Remove unused dependencies per scientific analysis"
\`\`\`

---

**Generated by**: PMAT v2.202.0
**Specification**: Scientific Dependency Reduction (10 peer-reviewed studies)
EOF

echo -e "${GREEN}✓ Report saved to: /tmp/dependency-reduction-report.md${NC}"

# Display summary
echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}📊 Summary${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${YELLOW}Current State:${NC}"
echo -e "  Dependencies: ${dep_count_before}"
echo -e "  Compile Time: ${compile_time_before}s"
echo -e "  Binary Size: ${binary_size_mb_before} MB"
echo ""
echo -e "${YELLOW}Estimated After Removal:${NC}"
echo -e "  Dependencies: $((dep_count_before - deps_to_remove))"
echo -e "  Compile Time: ${estimated_compile_after}s (save ${estimated_compile_reduction}s)"
binary_size_estimate=$(bc <<< "scale=2; ${binary_size_mb_before} - 1.2")
echo -e "  Binary Size: ~${binary_size_estimate} MB"
echo ""
echo -e "${GREEN}✓ Full report: /tmp/dependency-reduction-report.md${NC}"
echo -e "${GREEN}✓ PMAT analysis: /tmp/pmat-*-languages.log${NC}"
echo ""

# Open report in less if interactive
if [ -t 1 ]; then
    echo -e "${BLUE}Press Enter to view full report (or Ctrl+C to exit)${NC}"
    read -r
    less /tmp/dependency-reduction-report.md
fi
