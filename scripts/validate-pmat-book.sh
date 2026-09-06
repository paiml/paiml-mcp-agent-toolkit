#!/bin/bash
# Fast pmat-book validation script
# Runs critical chapter tests in parallel for speed
# Exit immediately on first failure (fail-fast)
#
# FALSIFICATION CONTROL (issue #1084)
# -----------------------------------
# Before a chapter's real run, every script in that chapter is run against a
# deliberately broken `pmat` (answers --version, fails everything else). At
# least one script must FAIL that control. A chapter whose scripts all pass
# while pmat is non-functional has asserted nothing about pmat, and is
# reported VACUOUS.
#
# This was measured, not assumed: on 2026-08-28, with a broken-pmat shim first
# on PATH, 6 of the 7 scripts the gate then executed still passed. Only
# tests/ch13/test_language_examples.sh failed. A gate that has never been shown
# capable of failing is not evidence.

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

BOOK_DIR="${PMAT_BOOK_DIR:-$HOME/src/pmat-book}"
PARALLEL_JOBS="${PMAT_BOOK_JOBS:-4}"

# Per-script wall-clock budget, applied to both the control run and the real
# run. The control is on by default because it is cheap: measured end-to-end
# on 2026-08-28 against the real pmat-book, 0.69s with the control vs 0.48s
# with PMAT_BOOK_SKIP_CONTROL=1 - nowhere near this per-script budget, because
# a broken pmat makes a script exit early rather than late. Set
# PMAT_BOOK_SKIP_CONTROL=1 to turn it off; doing so downgrades this gate to
# "the scripts exited 0", which is what #1084 is about.
SCRIPT_TIMEOUT="${PMAT_BOOK_TIMEOUT:-60}"
SKIP_CONTROL="${PMAT_BOOK_SKIP_CONTROL:-0}"

# PMAT_BOOK_DIR is external input and we cd into it and execute the shell
# scripts we find underneath, so refuse a value that can climb out of the
# directory the operator named.
if [ -z "$BOOK_DIR" ] || [ "$BOOK_DIR" != "${BOOK_DIR#*..}" ]; then
    echo -e "${RED}❌ PMAT_BOOK_DIR must be non-empty and free of '..': $BOOK_DIR${NC}" >&2
    exit 1
fi

# Check if pmat-book exists.
# Absent book => skip, not fail. But say so on stderr and say it loudly: a
# green exit here means NOTHING was validated, and CLAUDE.md calls this the
# release gate.
if [ ! -d "$BOOK_DIR" ]; then
    {
        echo ""
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${YELLOW}⚠️  SKIPPED - NOTHING WAS VALIDATED${NC}"
        echo -e "${YELLOW}   pmat-book not found at: $BOOK_DIR${NC}"
        echo -e "${YELLOW}   This exits 0. A green 'make validate-book' from${NC}"
        echo -e "${YELLOW}   this machine is therefore NOT evidence of anything.${NC}"
        echo -e "${YELLOW}   Clone pmat-book or set PMAT_BOOK_DIR to validate.${NC}"
        echo -e "${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo ""
    } >&2
    exit 0  # Don't fail if book doesn't exist
fi

echo -e "${YELLOW}📚 Validating pmat-book (parallel, fail-fast)${NC}"
echo ""

# ---------------------------------------------------------------------------
# The broken-pmat shim used by the falsification control.
# Built once, into a temp dir, and put first on PATH only for the control run.
# ---------------------------------------------------------------------------
CONTROL_DIR=""

cleanup_control() {
    if [ -n "$CONTROL_DIR" ] && [ -d "$CONTROL_DIR" ]; then
        rm -rf "${CONTROL_DIR:?}"
    fi
}
trap cleanup_control EXIT

if [ "$SKIP_CONTROL" != "1" ]; then
    CONTROL_DIR="$(mktemp -d)"
    cat > "$CONTROL_DIR/pmat" <<'CONTROL_SHIM'
#!/bin/sh
# Deliberately broken pmat. Used only by validate-pmat-book.sh's falsification
# control. It answers --version so a chapter script's "is pmat installed?"
# probe still succeeds and the script still takes its non-mock path, and it
# fails every other invocation. Any assertion that really depends on pmat must
# fail against this.
case "$1" in
    --version|-V)
        echo "pmat 0.0.0-broken-falsification-control"
        exit 0
        ;;
esac
echo "broken-pmat control: refusing to run 'pmat $*'" >&2
exit 1
CONTROL_SHIM
    chmod +x "$CONTROL_DIR/pmat"
    echo -e "${YELLOW}🔬 Falsification control ON (broken-pmat shim: $CONTROL_DIR)${NC}"
else
    echo -e "${YELLOW}⚠️  Falsification control DISABLED (PMAT_BOOK_SKIP_CONTROL=1)${NC}" >&2
    echo -e "${YELLOW}   Chapters are not checked for the ability to fail.${NC}" >&2
fi
echo ""

# Critical chapters that MUST pass (covers all major functionality)
CRITICAL_CHAPTERS=(
    "13"  # Multi-language support (CRITICAL)
    "14"  # Quality-driven development
    "05"  # Analyze command suite
    "07"  # Quality gate
)

# Specific test files to run for each chapter (if needed)
declare -A SPECIFIC_TESTS
SPECIFIC_TESTS[13]="test_language_examples.sh"  # Use main multi-lang test, not minimal

# Resolve the script list for a chapter. Single source of truth: the control
# run and the real run must execute exactly the same set, or the control
# proves nothing about what actually ran.
# Prints one absolute script path per line.
resolve_chapter_scripts() {
    local ch="$1"
    local test_dir="$BOOK_DIR/tests/ch$ch"

    # Rebuild SPECIFIC_TESTS from the SPECIFIC_TESTS_<ch> variables the parent
    # exported: bash cannot export an array (`export -a` returns 0 and the
    # child still sees nothing - verified), so the flat variables are the only
    # channel into this subshell.
    declare -A SPECIFIC_TESTS
    local var
    for var in $(compgen -v SPECIFIC_TESTS_ 2>/dev/null || true); do
        SPECIFIC_TESTS["${var#SPECIFIC_TESTS_}"]="${!var}"
    done

    # Check if we have a specific test file for this chapter.
    # NOTE: this subscript used to be written `${SPECIFIC_TESTS[$ch ]}` with a
    # stray space, at three places. "13 " is not "13" for an associative
    # array, so the lookup always missed, the find fallback below silently
    # rescued it, and ch13 ran all three of its scripts - the opposite of the
    # comment on SPECIFIC_TESTS[13] above.
    local test_scripts=""
    if [ -n "${SPECIFIC_TESTS[$ch]:-}" ]; then
        test_scripts="$test_dir/${SPECIFIC_TESTS[$ch]}"
        if [ ! -f "$test_scripts" ]; then
            echo -e "${YELLOW}⚠️  Specific test ${SPECIFIC_TESTS[$ch]} not found for Chapter $ch${NC}" >&2
            test_scripts=""
        fi
    fi

    # Otherwise, find all test scripts
    if [ -z "$test_scripts" ]; then
        test_scripts="$(find "$test_dir" -name 'test_*.sh' -type f 2>/dev/null || true)"
    fi

    if [ -n "$test_scripts" ]; then
        printf "%s\n" "$test_scripts"
    fi
}

# The falsification control for one chapter.
# Usage: run_falsification_control <chapter> <script>...
# Returns 0 if at least one script FAILED against the broken pmat, 1 if every
# script passed (chapter is vacuous).
run_falsification_control() {
    local ch="$1"
    shift
    local total=$#
    local failed_count=0
    local could_not_fail=()
    local script script_name

    for script in "$@"; do
        script_name="$(basename "$script")"
        if (cd "$BOOK_DIR" \
            && PATH="$CONTROL_DIR:$PATH" timeout "$SCRIPT_TIMEOUT" bash "$script" >/dev/null 2>&1); then
            could_not_fail+=("$script_name")
        else
            failed_count=$((failed_count + 1))
        fi
    done

    if [ "$failed_count" -gt 0 ]; then
        echo -e "${GREEN}🔬 Ch$ch: control OK ($failed_count of $total script(s) fail without a working pmat)${NC}"
        return 0
    fi

    echo -e "${RED}❌ Ch$ch: VACUOUS - all $total script(s) PASSED against a deliberately broken pmat${NC}"
    echo -e "${RED}   These scripts cannot fail, so they validate nothing:${NC}"
    for script_name in "${could_not_fail[@]}"; do
        echo -e "${RED}     - tests/ch$ch/$script_name${NC}"
    done
    echo -e "${RED}   Fix them in pmat-book (see paiml/pmat#1084), or re-point${NC}"
    echo -e "${RED}   SPECIFIC_TESTS[$ch] at a script that exercises pmat.${NC}"
    return 1
}

# Function to run a single chapter test
run_chapter_test() {
    local ch="$1"
    local test_dir="$BOOK_DIR/tests/ch$ch"

    if [ ! -d "$test_dir" ]; then
        echo -e "${YELLOW}⚠️  Chapter $ch NOT VALIDATED: no test directory at $test_dir${NC}" >&2
        return 0
    fi

    local raw_scripts
    raw_scripts="$(resolve_chapter_scripts "$ch")"

    if [ -z "$raw_scripts" ]; then
        echo -e "${YELLOW}⚠️  Chapter $ch NOT VALIDATED: no test scripts found${NC}" >&2
        return 0
    fi

    local scripts=()
    mapfile -t scripts <<< "$raw_scripts"

    # Control first: a chapter that cannot fail is not worth running.
    if [ "$SKIP_CONTROL" != "1" ]; then
        if ! run_falsification_control "$ch" "${scripts[@]}"; then
            return 1  # Fail fast
        fi
    fi

    # Run each test script for real
    local script script_name
    for script in "${scripts[@]}"; do
        script_name="$(basename "$script")"
        # cd to book directory before running test to ensure correct working directory
        if (cd "$BOOK_DIR" && timeout "$SCRIPT_TIMEOUT" bash "$script" >/dev/null 2>&1); then
            echo -e "${GREEN}✅ Ch$ch: $script_name${NC}"
        else
            echo -e "${RED}❌ Ch$ch: $script_name FAILED${NC}"
            return 1  # Fail fast
        fi
    done

    return 0
}

# Run tests in parallel with fail-fast.
# NOTE: there used to be PASSED_TESTS/FAILED_TESTS arrays here, appended to
# inside run_chapter_test and `export -a`d. Neither could ever reach the
# parent - a subshell cannot write its caller's variables, and bash cannot
# export an array at all - so both were always empty in the summary below.
# The subshells report by printing instead.
export -f run_chapter_test resolve_chapter_scripts run_falsification_control
export BOOK_DIR CONTROL_DIR SCRIPT_TIMEOUT SKIP_CONTROL
export GREEN RED YELLOW NC
# Export the associative array as flat per-key variables (see the note in
# resolve_chapter_scripts: bash has no array export).
for key in "${!SPECIFIC_TESTS[@]}"; do
    export "SPECIFIC_TESTS_$key=${SPECIFIC_TESTS[$key]}"
done

# Use xargs for parallel execution with fail-fast.
# The `|| EXIT_CODE=$?` matters: under `set -e` a bare failing pipeline aborts
# the script on the spot, which is why the failure banner below was previously
# unreachable (a broken pmat exited 123 from xargs and printed no summary).
EXIT_CODE=0
printf "%s\n" "${CRITICAL_CHAPTERS[@]}" \
    | xargs -P "$PARALLEL_JOBS" -I {} bash -c 'run_chapter_test "$@"' _ {} \
    || EXIT_CODE=$?

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

if [ "$EXIT_CODE" -eq 0 ]; then
    echo -e "${GREEN}✅ QUALITY GATE PASSED: pmat-book validation${NC}"
    echo -e "${GREEN}   ${#CRITICAL_CHAPTERS[@]} critical chapters validated${NC}"
    if [ "$SKIP_CONTROL" != "1" ]; then
        echo -e "${GREEN}   Every chapter that had scripts was first shown${NC}"
        echo -e "${GREEN}   capable of failing against a broken pmat${NC}"
    else
        echo -e "${YELLOW}   Falsification control was DISABLED for this run${NC}"
    fi
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 0
else
    echo -e "${RED}❌ QUALITY GATE FAILED: pmat-book validation${NC}"
    echo -e "${RED}   A critical chapter failed, or could not fail (VACUOUS)${NC}"
    echo -e "${RED}   See the per-chapter lines above for which scripts${NC}"
    echo ""
    echo "To bypass (NOT RECOMMENDED):"
    echo "  git commit --no-verify"
    echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
    exit 1
fi
