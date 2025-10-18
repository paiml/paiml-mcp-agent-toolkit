#!/bin/bash
# Pre-commit Hook: README Hallucination Detection
#
# This hook validates AI-generated documentation against codebase reality
# before allowing commits that modify documentation files.
#
# Installation:
#   cp docs/examples/pre-commit-validate-readme.sh .git/hooks/pre-commit
#   chmod +x .git/hooks/pre-commit
#
# Based on Sprint 38 (validate-readme CLI command)
#
# Toyota Way Principles:
# - Jidoka (Built-in Quality): Catch hallucinations at commit time
# - Andon Cord: Stop the commit if documentation contains hallucinations
# - Genchi Genbutsu: Validate against actual codebase facts

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🔍 Pre-commit Quality Gate: README Hallucination Detection${NC}"
echo ""

# Check if any documentation files are being committed
DOC_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '(README|CLAUDE|GEMINI|AGENT)\.md$' || true)

if [ -z "$DOC_FILES" ]; then
    echo -e "${GREEN}✅ No documentation changes detected - skipping validation${NC}"
    exit 0
fi

echo -e "${YELLOW}📝 Documentation changes detected:${NC}"
echo "$DOC_FILES" | sed 's/^/  - /'
echo ""

# Check if pmat is installed
if ! command -v pmat &> /dev/null; then
    echo -e "${RED}❌ ERROR: pmat not found${NC}"
    echo "   Install with: cargo install pmat"
    exit 1
fi

# Check if deep context exists and is recent (< 1 hour old)
DEEP_CONTEXT="deep_context.md"
if [ ! -f "$DEEP_CONTEXT" ] || [ $(find "$DEEP_CONTEXT" -mmin +60 2>/dev/null | wc -l) -gt 0 ]; then
    echo -e "${YELLOW}📊 Generating deep context (caching codebase facts)...${NC}"

    pmat context --output "$DEEP_CONTEXT" --format llm-optimized 2>&1 | grep -v "INFO\|DEBUG" || true

    echo -e "${GREEN}✅ Deep context generated${NC}"
    echo ""
fi

# Validate each changed documentation file
VALIDATION_FAILED=0

for FILE in $DOC_FILES; do
    echo -e "${BLUE}🔬 Validating $FILE...${NC}"

    # Run validation (capture exit code)
    if pmat validate-readme \
        --targets "$FILE" \
        --deep-context "$DEEP_CONTEXT" \
        --fail-on-contradiction \
        --failures-only 2>&1; then
        echo -e "${GREEN}✅ $FILE: No hallucinations detected${NC}"
    else
        echo -e "${RED}❌ $FILE: Hallucinations detected!${NC}"
        VALIDATION_FAILED=1
    fi

    echo ""
done

# Summary
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ $VALIDATION_FAILED -eq 0 ]; then
    echo -e "${GREEN}✅ QUALITY GATE PASSED: README validation${NC}"
    echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${GREEN}Toyota Way - Jidoka: Built-in quality verified ✓${NC}"
    exit 0
else
    echo -e "${RED}❌ QUALITY GATE FAILED: Hallucinations detected${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${YELLOW}Fix the hallucinated claims before committing.${NC}"
    echo ""
    echo "To bypass this check (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo ""
    echo -e "${YELLOW}Toyota Way - Andon Cord: Stopping the line due to quality issues.${NC}"
    exit 1
fi

# Configuration Options:
#
# 1. Validate multiple files:
#    Add more patterns to grep: grep -E '(README|DOCS|API)\.md$'
#
# 2. Strict mode (fail on unverified):
#    Add --fail-on-unverified flag to validation
#
# 3. Custom thresholds:
#    Add --verified-threshold 0.85 --contradiction-threshold 0.4
#
# 4. Generate reports:
#    pmat validate-readme --output json > .git/validation_report.json
#
# 5. Skip validation for specific files:
#    DOC_FILES=$(git diff ... | grep -v -E '(ARCHIVE|OLD)')
