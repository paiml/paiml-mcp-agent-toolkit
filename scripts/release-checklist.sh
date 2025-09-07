#!/usr/bin/env bash

# Canonical Release Checklist Script
# Following docs/todo/canonical-version-updates-spec.md
#
# This script validates all requirements before a release
# and provides interactive guidance through the release process

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check mark and cross mark
CHECK="✅"
CROSS="❌"
WARN="⚠️ "

# Track overall status
CHECKS_PASSED=true

echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${BLUE}                     CANONICAL RELEASE CHECKLIST                               ${NC}"
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Function to run a check
run_check() {
    local description="$1"
    local command="$2"
    local critical="${3:-true}"
    
    echo -n "🔍 $description... "
    
    if eval "$command" > /dev/null 2>&1; then
        echo -e "${GREEN}${CHECK}${NC}"
        return 0
    else
        if [ "$critical" = "true" ]; then
            echo -e "${RED}${CROSS}${NC}"
            CHECKS_PASSED=false
            return 1
        else
            echo -e "${YELLOW}${WARN}${NC}"
            return 0
        fi
    fi
}

# Function to display a section
section() {
    echo ""
    echo -e "${BLUE}═══ $1 ═══${NC}"
    echo ""
}

# Pre-release checks
section "PRE-RELEASE CHECKS"

# 0. Clean build artifacts for fresh release
echo -n "0️⃣  Cleaning build artifacts... "
if make clean-quick > /dev/null 2>&1; then
    echo -e "${GREEN}${CHECK} Build cleaned${NC}"
else
    echo -e "${YELLOW}${WARN} Could not clean build${NC}"
fi

# 1. Version consistency
echo -n "1️⃣  Version consistency... "
if grep '^version' Cargo.toml server/Cargo.toml | uniq -c | grep -q '      2 version'; then
    CURRENT_VERSION=$(grep -m1 '^version = ' Cargo.toml | cut -d'"' -f2)
    echo -e "${GREEN}${CHECK} (v${CURRENT_VERSION})${NC}"
else
    echo -e "${RED}${CROSS} Versions don't match!${NC}"
    CHECKS_PASSED=false
fi

# 2. Git status
echo -n "2️⃣  Git status... "
if [ -z "$(git status --porcelain)" ]; then
    echo -e "${GREEN}${CHECK} Clean${NC}"
else
    echo -e "${YELLOW}${WARN} Uncommitted changes${NC}"
    git status --short
fi

# 3. Current branch
echo -n "3️⃣  Current branch... "
CURRENT_BRANCH=$(git branch --show-current)
if [ "$CURRENT_BRANCH" = "master" ] || [ "$CURRENT_BRANCH" = "main" ]; then
    echo -e "${GREEN}${CHECK} ($CURRENT_BRANCH)${NC}"
else
    echo -e "${YELLOW}${WARN} Not on main branch ($CURRENT_BRANCH)${NC}"
fi

# 4. Up to date with remote
echo -n "4️⃣  Remote sync... "
git fetch origin > /dev/null 2>&1
if [ "$(git rev-parse HEAD)" = "$(git rev-parse @{u})" ]; then
    echo -e "${GREEN}${CHECK} Up to date${NC}"
else
    echo -e "${YELLOW}${WARN} Not synced with remote${NC}"
fi

section "QUALITY GATES"

# 5. Build check
run_check "5️⃣  Cargo build" "cargo check --all"

# 6. Clippy
run_check "6️⃣  Clippy lints" "cargo clippy --all -- -D warnings" false

# 7. Tests
run_check "7️⃣  Test suite" "cargo test --all" 

# 8. SATD check
echo -n "8️⃣  SATD check... "
if command -v pmat > /dev/null 2>&1; then
    SATD_COUNT=$(pmat analyze satd --strict 2>/dev/null | grep -E 'Total SATD items:|No SATD' | grep -oE '[0-9]+' | head -1 || echo "0")
    if [ "$SATD_COUNT" = "0" ] || [ -z "$SATD_COUNT" ]; then
        echo -e "${GREEN}${CHECK} No SATD${NC}"
    else
        echo -e "${YELLOW}${WARN} $SATD_COUNT SATD items found${NC}"
    fi
else
    echo -e "${YELLOW}${WARN} pmat not installed${NC}"
fi

section "DEPENDENCY CHECKS"

# 9. Security audit
echo -n "9️⃣  Security audit... "
if command -v cargo-audit > /dev/null 2>&1; then
    if cargo audit 2>/dev/null | grep -q "0 vulnerabilities"; then
        echo -e "${GREEN}${CHECK} No vulnerabilities${NC}"
    else
        VULN_COUNT=$(cargo audit 2>/dev/null | grep -oE '[0-9]+ vulnerabilities' | grep -oE '[0-9]+' || echo "unknown")
        echo -e "${YELLOW}${WARN} $VULN_COUNT vulnerabilities${NC}"
    fi
else
    echo -e "${YELLOW}${WARN} cargo-audit not installed${NC}"
fi

# 10. Outdated dependencies
echo -n "🔟 Outdated deps... "
if command -v cargo-outdated > /dev/null 2>&1; then
    OUTDATED=$(cargo outdated --root-deps-only 2>/dev/null | grep -c "^[a-z]" || echo "0")
    if [ "$OUTDATED" = "0" ]; then
        echo -e "${GREEN}${CHECK} All up to date${NC}"
    else
        echo -e "${YELLOW}${WARN} $OUTDATED outdated dependencies${NC}"
    fi
else
    echo -e "${YELLOW}${WARN} cargo-outdated not installed${NC}"
fi

section "SEMVER COMPATIBILITY"

# 11. SemVer check
echo -n "1️⃣1️⃣ SemVer check... "
if command -v cargo-semver-checks > /dev/null 2>&1; then
    if cargo semver-checks check-release 2>&1 | grep -q "PASS\|compatible"; then
        echo -e "${GREEN}${CHECK} Compatible${NC}"
    else
        echo -e "${YELLOW}${WARN} Review needed${NC}"
    fi
else
    echo -e "${YELLOW}${WARN} cargo-semver-checks not installed${NC}"
fi

section "CHANGELOG STATUS"

# 12. Changelog
echo -n "1️⃣2️⃣ CHANGELOG.md... "
if grep -q "## \[Unreleased\]" CHANGELOG.md; then
    UNRELEASED_LINES=$(sed -n '/## \[Unreleased\]/,/## \[/p' CHANGELOG.md | wc -l)
    if [ "$UNRELEASED_LINES" -gt 3 ]; then
        echo -e "${GREEN}${CHECK} Has unreleased changes${NC}"
    else
        echo -e "${YELLOW}${WARN} No unreleased changes documented${NC}"
    fi
else
    echo -e "${RED}${CROSS} Missing [Unreleased] section${NC}"
    CHECKS_PASSED=false
fi

section "VERSION BUMP RECOMMENDATION"

# Analyze changes since last tag
LAST_TAG=$(git describe --tags --abbrev=0 2>/dev/null || echo "")
if [ -n "$LAST_TAG" ]; then
    echo "📊 Analyzing changes since $LAST_TAG..."
    
    # Count different types of commits
    BREAKING=$(git log --oneline "$LAST_TAG"..HEAD | grep -cE '^[a-f0-9]+ (BREAKING|breaking|!)' || echo "0")
    FEATURES=$(git log --oneline "$LAST_TAG"..HEAD | grep -cE '^[a-f0-9]+ feat(\(.*\))?:' || echo "0")
    FIXES=$(git log --oneline "$LAST_TAG"..HEAD | grep -cE '^[a-f0-9]+ fix(\(.*\))?:' || echo "0")
    
    echo "  Breaking changes: $BREAKING"
    echo "  New features: $FEATURES"
    echo "  Bug fixes: $FIXES"
    echo ""
    
    if [ "$BREAKING" -gt 0 ]; then
        echo -e "  ${RED}Recommended: MAJOR release${NC} (breaking changes detected)"
        RECOMMENDED="major"
    elif [ "$FEATURES" -gt 0 ]; then
        echo -e "  ${BLUE}Recommended: MINOR release${NC} (new features added)"
        RECOMMENDED="minor"
    else
        echo -e "  ${GREEN}Recommended: PATCH release${NC} (bug fixes only)"
        RECOMMENDED="patch"
    fi
else
    echo "No previous tags found. This will be the first release."
    RECOMMENDED="minor"
fi

section "RELEASE READINESS"

if [ "$CHECKS_PASSED" = true ]; then
    echo -e "${GREEN}✅ All critical checks passed! Ready for release.${NC}"
    echo ""
    echo "Next steps:"
    echo "  1. Review any warnings above"
    echo "  2. Update CHANGELOG.md with release notes"
    echo "  3. Run: make release-$RECOMMENDED"
    echo "     Or use GitHub Actions: gh workflow run canonical-release.yml"
else
    echo -e "${RED}❌ Some critical checks failed. Please fix before releasing.${NC}"
    exit 1
fi

echo ""
echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

# Interactive mode
if [ "${1:-}" = "--interactive" ]; then
    echo ""
    echo "🤖 Interactive Release Mode"
    echo ""
    read -p "Do you want to proceed with a $RECOMMENDED release? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "Starting release process..."
        make release-$RECOMMENDED
    else
        echo "Release cancelled."
    fi
fi