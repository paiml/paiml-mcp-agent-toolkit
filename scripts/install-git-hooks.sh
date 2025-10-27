#!/usr/bin/env bash
# Install git hooks for paiml-mcp-agent-toolkit repository
#
# Run this script after cloning the repository:
#   bash scripts/install-git-hooks.sh

set -e

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m'

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}📦 Installing git hooks for paiml-mcp-agent-toolkit...${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Get repository root
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
HOOK_DIR="${REPO_ROOT}/.git/hooks"

# Check if we're in a git repository
if [ ! -d "${REPO_ROOT}/.git" ]; then
    echo -e "${YELLOW}Error: Not in a git repository${NC}"
    exit 1
fi

# === PRE-COMMIT HOOK ===
echo -e "${GREEN}1. Installing pre-commit hook (bashrs + book sync warning)...${NC}"

cat > "${HOOK_DIR}/pre-commit" <<'HOOK_EOF'
#!/bin/bash
# Pre-commit hook for paiml-mcp-agent-toolkit
# Enforces bash/Makefile quality with bashrs
# Warns about unpushed pmat-book commits

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

echo "Running pre-commit checks..."

# Check pmat-book sync status
BOOK_DIR="/home/noah/src/pmat-book"
if [ -d "${BOOK_DIR}/.git" ]; then
    cd "${BOOK_DIR}"
    REMOTE_BRANCH=$(git rev-parse --abbrev-ref --symbolic-full-name "@{u}" 2>/dev/null || echo "")

    if [ -n "${REMOTE_BRANCH}" ]; then
        UNPUSHED_BOOK_COMMITS=$(git log "${REMOTE_BRANCH}..HEAD" --oneline 2>/dev/null || echo "")

        if [ -n "${UNPUSHED_BOOK_COMMITS}" ]; then
            COMMIT_COUNT=$(echo "${UNPUSHED_BOOK_COMMITS}" | wc -l)
            echo ""
            echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo -e "${RED}⚠️  WARNING: pmat-book has ${COMMIT_COUNT} unpushed commit(s)${NC}"
            echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            echo -e "${YELLOW}Unpushed commits in pmat-book:${NC}"
            echo "${UNPUSHED_BOOK_COMMITS}"
            echo ""
            echo -e "${CYAN}📚 POLICY: Book updates MUST be pushed with code changes${NC}"
            echo -e "${CYAN}   The live book at https://paiml.github.io/pmat-book/ is out of date${NC}"
            echo ""
            echo -e "${GREEN}To push pmat-book commits:${NC}"
            echo -e "${GREEN}   cd ${BOOK_DIR} && git push origin main${NC}"
            echo ""
            echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
            echo ""
            echo -e "${YELLOW}⚠️  This is a WARNING (commit allowed)${NC}"
            echo -e "${YELLOW}   But the pre-push hook will BLOCK until book is pushed${NC}"
            echo ""
        fi
    fi
    cd - > /dev/null
fi

# Check if bashrs is installed
if ! command -v bashrs &> /dev/null; then
    echo -e "${RED}ERROR: bashrs not found${NC}"
    echo "Install bashrs: cargo install bashrs"
    echo "Or build from source: cd ../bashrs && cargo build --release"
    exit 1
fi

# Get list of staged shell scripts and Makefiles
STAGED_FILES=$(git diff --cached --name-only --diff-filter=ACM | grep -E '\.(sh|bash)$|^Makefile$' || true)

if [ -z "${STAGED_FILES}" ]; then
    echo -e "${GREEN}✓ No bash/Makefile files to check${NC}"
    exit 0
fi

echo "Checking bash/Makefile files with bashrs..."
echo ""

FAILED=0
ERROR_COUNT=0
WARNING_COUNT=0

for FILE in ${STAGED_FILES}; do
    if [ -f "${FILE}" ]; then
        echo "Linting: ${FILE}"

        # Run bashrs lint and capture exit code
        if bashrs lint "${FILE}" 2>&1; then
            echo -e "${GREEN}✓ ${FILE} passed${NC}"
        else
            EXIT_CODE=$?

            # Exit code 1 = warnings, 2 = errors
            if [ ${EXIT_CODE} -eq 2 ]; then
                echo -e "${RED}✗ ${FILE} has ERRORS${NC}"
                ERROR_COUNT=$((ERROR_COUNT + 1))
                FAILED=1
            elif [ ${EXIT_CODE} -eq 1 ]; then
                echo -e "${YELLOW}⚠ ${FILE} has warnings${NC}"
                WARNING_COUNT=$((WARNING_COUNT + 1))
                # Warnings don't block commit by default
            else
                echo -e "${RED}✗ ${FILE} failed with exit code ${EXIT_CODE}${NC}"
                FAILED=1
            fi
        fi
        echo ""
    fi
done

echo "----------------------------------------"
if [ ${ERROR_COUNT} -gt 0 ]; then
    echo -e "${RED}FAILED: ${ERROR_COUNT} file(s) with errors${NC}"
    echo ""
    echo "Fix errors with: bashrs lint <file> --fix (when available)"
    echo "Or bypass this check with: git commit --no-verify"
    exit 1
elif [ ${WARNING_COUNT} -gt 0 ]; then
    echo -e "${YELLOW}WARNINGS: ${WARNING_COUNT} file(s) with warnings${NC}"
    echo "Consider fixing warnings before committing"
    echo ""
    # Allow commit with warnings
fi

echo -e "${GREEN}✓ All bashrs checks passed${NC}"
exit 0
HOOK_EOF

chmod +x "${HOOK_DIR}/pre-commit"
echo -e "${GREEN}   ✅ pre-commit hook installed${NC}"
echo ""

# === PRE-PUSH HOOK ===
echo -e "${GREEN}2. Installing pre-push hook (pmat-book sync enforcement)...${NC}"

cat > "${HOOK_DIR}/pre-push" <<'HOOK_EOF'
#!/bin/bash
# Pre-push hook for paiml-mcp-agent-toolkit
# ENFORCES that pmat-book commits are pushed alongside code changes
#
# CRITICAL POLICY:
# - Any push to paiml-mcp-agent-toolkit MUST have all pmat-book commits pushed first
# - This prevents the 404 issue where live book becomes out of sync with code
# - Especially important for crates.io releases which MUST have updated documentation

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
BOLD='\033[1m'
NC='\033[0m' # No Color

BOOK_DIR="/home/noah/src/pmat-book"

echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${CYAN}📚 Checking pmat-book sync status...${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""

# Check if pmat-book directory exists
if [ ! -d "${BOOK_DIR}/.git" ]; then
    echo -e "${YELLOW}⚠️  WARNING: pmat-book repository not found at ${BOOK_DIR}${NC}"
    echo -e "${YELLOW}   Skipping book sync check${NC}"
    echo ""
    exit 0
fi

# Navigate to book directory
cd "${BOOK_DIR}"

# Check for remote tracking branch
REMOTE_BRANCH=$(git rev-parse --abbrev-ref --symbolic-full-name "@{u}" 2>/dev/null || echo "")

if [ -z "${REMOTE_BRANCH}" ]; then
    echo -e "${YELLOW}⚠️  WARNING: pmat-book has no remote tracking branch${NC}"
    echo -e "${YELLOW}   Set up tracking with: cd ${BOOK_DIR} && git push -u origin main${NC}"
    echo ""
    cd - > /dev/null
    exit 0
fi

# Check for unpushed commits
UNPUSHED_COMMITS=$(git log "${REMOTE_BRANCH}..HEAD" --oneline 2>/dev/null || echo "")

if [ -n "${UNPUSHED_COMMITS}" ]; then
    COMMIT_COUNT=$(echo "${UNPUSHED_COMMITS}" | wc -l)

    echo ""
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}${BOLD}❌ PUSH BLOCKED: pmat-book has unpushed commits${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${BOLD}CRITICAL POLICY VIOLATION:${NC}"
    echo -e "  You have ${COMMIT_COUNT} unpushed commit(s) in pmat-book"
    echo ""
    echo -e "${YELLOW}Unpushed commits:${NC}"
    echo "${UNPUSHED_COMMITS}"
    echo ""
    echo -e "${CYAN}📚 WHY THIS MATTERS:${NC}"
    echo -e "  • Live book at https://paiml.github.io/pmat-book/ will be out of sync"
    echo -e "  • Users will see outdated documentation (404 issue)"
    echo -e "  • crates.io releases MUST have matching documentation"
    echo -e "  • Book updates must deploy with code changes"
    echo ""
    echo -e "${GREEN}${BOLD}TO FIX:${NC}"
    echo -e "${GREEN}  1. Push pmat-book commits first:${NC}"
    echo -e "${GREEN}     cd ${BOOK_DIR} && git push origin main${NC}"
    echo ""
    echo -e "${GREEN}  2. Then push paiml-mcp-agent-toolkit:${NC}"
    echo -e "${GREEN}     cd - && git push${NC}"
    echo ""
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${RED}${BOLD}Push rejected. Fix the issue above and try again.${NC}"
    echo -e "${RED}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo ""
    echo -e "${YELLOW}To bypass (NOT RECOMMENDED):${NC}"
    echo -e "${YELLOW}  git push --no-verify${NC}"
    echo ""

    cd - > /dev/null
    exit 1
fi

# All good!
echo -e "${GREEN}✅ pmat-book is in sync - no unpushed commits${NC}"
echo -e "${GREEN}   Book is ready for deployment${NC}"
echo ""
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

cd - > /dev/null
exit 0
HOOK_EOF

chmod +x "${HOOK_DIR}/pre-push"
echo -e "${GREEN}   ✅ pre-push hook installed${NC}"
echo ""

# === SUMMARY ===
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✅ All git hooks installed successfully!${NC}"
echo -e "${CYAN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo -e "${GREEN}What these hooks do:${NC}"
echo ""
echo -e "${CYAN}📋 pre-commit hook:${NC}"
echo -e "  1. Checks bash/Makefile files with bashrs"
echo -e "  2. Warns about unpushed pmat-book commits"
echo -e "  3. Allows commit (warning only)"
echo ""
echo -e "${CYAN}🚫 pre-push hook:${NC}"
echo -e "  1. BLOCKS push if pmat-book has unpushed commits"
echo -e "  2. Prevents 404 issue (live book out of sync)"
echo -e "  3. Critical for crates.io releases"
echo ""
echo -e "${YELLOW}To bypass hooks (not recommended):${NC}"
echo -e "  git commit --no-verify"
echo -e "  git push --no-verify"
echo ""
echo -e "${GREEN}See CLAUDE.md for full policy documentation${NC}"
echo ""
