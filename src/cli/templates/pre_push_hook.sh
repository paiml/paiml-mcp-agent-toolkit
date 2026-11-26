#!/bin/sh
# PMAT ComputeBrick Pre-Push Enforcement (PROBAR-SPEC-009-P8)
# This hook validates ComputeBrick compliance before push.

set -e

# git writes one line per pushed ref on stdin; read it before anything else can.
PUSH_REFS=""
if [ ! -t 0 ]; then PUSH_REFS=$(cat); fi

echo "Running ComputeBrick compliance checks..."

# Check ComputeBrick compliance via pmat comply
COMPLY_OUTPUT=$(pmat comply check --failures-only 2>&1) || true
if echo "$COMPLY_OUTPUT" | grep -q "ComputeBrick Compliance.*critical"; then
    echo "COMPUTEBRICK COMPLIANCE FAILURE"
    echo ""
    echo "$COMPLY_OUTPUT" | grep -A5 "ComputeBrick"
    echo ""
    echo "Fix critical violations before pushing."
    echo "Run 'pmat comply check' for full details."
    echo ""
    echo "Bypass (NOT RECOMMENDED):"
    echo "  git push --no-verify"
    exit 1
fi

# CB-2113 commit traceability: the judge CI runs, before the push. A push that
# only deletes refs (all-zero local oid) sends no commit to judge.
if [ -n "$PUSH_REFS" ] && ! printf '%s\n' "$PUSH_REFS" | grep -qv '^[^ ]* 00* '; then
    echo "Delete-only push: no commit for CB-2113 to judge."
else
    echo "Running CB-2113 commit traceability..."
    if ! pmat comply check --checks CB-2113 </dev/null; then
        echo ""
        echo "CB-2113 failed: CI will refuse these commits (see above)."
        echo "Each commit needs a 'Pmat-Ticket: <id>' trailer in its last paragraph."
        exit 1
    fi
fi

# Check probar GUI coverage if available (PROBAR-SPEC-009)
if command -v probador >/dev/null 2>&1; then
    echo "Checking probar GUI coverage..."
    if ! probador playbook --validate --min-coverage 80 2>/dev/null; then
        echo "Probar GUI coverage below 80%"
        echo "   Run 'probador playbook' to generate coverage report."
    fi
fi

# Check for .pmat-gates.toml ComputeBrick config
if [ -f "Cargo.toml" ]; then
    if grep -q "trueno\|probar\|realizar" Cargo.toml 2>/dev/null; then
        if [ ! -f ".pmat-gates.toml" ] || ! grep -q "\[compute-brick\]" .pmat-gates.toml 2>/dev/null; then
            echo "ComputeBrick project missing [compute-brick] in .pmat-gates.toml"
            echo "   Add configuration per docs/specifications/compute-brick-support.md"
        fi
    fi
fi

echo "ComputeBrick compliance: PASSED"
exit 0
