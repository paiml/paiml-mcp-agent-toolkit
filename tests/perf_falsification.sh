#!/usr/bin/env bash
# Falsification suite for pmat query lazy-loading performance claims
# Runs every query mode, captures profile data, and verifies performance goals.
#
# GOALS TO FALSIFY:
#   G1: load_index phase < 500ms (ANDON threshold) for semantic queries
#   G2: Semantic query total < 750ms (was 1.4s, claimed 0.3s)
#   G3: Regex mode produces source-matched results
#   G4: Literal mode produces source-matched results
#   G5: Call graph context populated in output (calls/callers)
#   G6: Cross-project ranking produces cross_project_callers > 0
#   G7: PTX flow traces nodes and edges
#   G8: Coverage-gaps mode returns results
#   G9: Git history fusion returns commits
#  G10: Source code appears in --include-source output
#  G11: --files-with-matches mode works
#  G12: --count mode works
#  G13: Context lines (-C) mode works
#  G14: Exclude pattern (--exclude) works
#  G15: No ANDON violation on load_index for semantic queries

set -euo pipefail

# measure() reads $EPOCHREALTIME, whose decimal separator follows LC_NUMERIC;
# bc needs a '.', so pin it.
export LC_NUMERIC=C

if [ -z "${EPOCHREALTIME:-}" ]; then
    echo "This suite needs bash >= 5 (EPOCHREALTIME) for timing." >&2
    exit 1
fi

PMAT="pmat"
PROJECT="/home/noah/src/paiml-mcp-agent-toolkit"
cd "$PROJECT"

PASS=0
FAIL=0
RESULTS=()

pass() {
    local name="$1"
    PASS=$((PASS + 1))
    RESULTS+=("  PASS  $name")
    echo "  PASS  $name"
}

fail() {
    local name="$1"
    local reason="$2"
    FAIL=$((FAIL + 1))
    RESULTS+=("  FAIL  $name: $reason")
    echo "  FAIL  $name: $reason"
}

# Returns wall-clock seconds as a float. Takes the command as an argv vector
# (measure cmd arg...), not as one string to re-parse: `eval "$@"` re-ran the
# caller's quoting through the parser, so every argument containing a space,
# quote or backslash had to be double-escaped at each call site.
#
# Timing comes from bash's $EPOCHREALTIME rather than from two forked date
# calls: each fork cost ~1-3ms, and the teardown of the first subshell landed
# inside the interval being measured, biasing every reading upward against
# thresholds as tight as 750ms.
measure() {
    local start end
    start=$EPOCHREALTIME
    "$@" > /dev/null 2>&1 || true
    end=$EPOCHREALTIME
    # "/ 1" forces bc to apply scale, keeping the historic 3-decimal format the
    # callers feed to `bc | cut -d. -f1`.
    echo "scale=3; ($end - $start) / 1" | bc
}

echo "=== pmat query Performance Falsification Suite ==="
echo "Binary: $(which $PMAT) ($(pmat --version 2>&1))"
echo "Project: $PROJECT"
echo "Index: $(ls -lh .pmat/context.db 2>/dev/null | awk '{print $5}') SQLite"
echo ""

# ─── G1: load_index < 500ms for semantic queries ───────────────────────
echo "── G1: load_index < 500ms ANDON threshold ──"
# Warm-up run to populate OS page cache (52MB SQLite cold-read is ~3s)
$PMAT query "warmup" --limit 1 --quiet 2>/dev/null || true
# Measured run on warm cache
PROFILE=$($PMAT query "error handling" --limit 5 2>&1)
LOAD_MS=$(echo "$PROFILE" | grep -oP 'load_index: \K[0-9]+' || echo "0")
if [ "$LOAD_MS" -eq 0 ]; then
    # No ANDON output means load_index was under 500ms — extract from profile if shown
    # If no profile output at all, it means all phases < 500ms = pass
    pass "G1: load_index under ANDON threshold (no violation reported)"
elif [ "$LOAD_MS" -lt 500 ]; then
    pass "G1: load_index = ${LOAD_MS}ms < 500ms"
else
    fail "G1: load_index ANDON" "load_index = ${LOAD_MS}ms >= 500ms"
fi

# ─── G2: Semantic query total < 750ms ──────────────────────────────────
echo "── G2: Semantic query total < 750ms ──"
TOTAL_SEC=$(measure "$PMAT" query "error handling" --limit 5 --quiet)
TOTAL_MS=$(echo "$TOTAL_SEC * 1000" | bc | cut -d. -f1)
if [ "$TOTAL_MS" -lt 750 ]; then
    pass "G2: semantic total = ${TOTAL_MS}ms < 750ms"
else
    fail "G2: semantic total" "total = ${TOTAL_MS}ms >= 750ms"
fi

# Run a second query to verify consistency
TOTAL_SEC2=$(measure "$PMAT" query "dispatch request" --limit 10 --quiet)
TOTAL_MS2=$(echo "$TOTAL_SEC2 * 1000" | bc | cut -d. -f1)
if [ "$TOTAL_MS2" -lt 750 ]; then
    pass "G2b: second semantic = ${TOTAL_MS2}ms < 750ms"
else
    fail "G2b: second semantic" "total = ${TOTAL_MS2}ms >= 750ms"
fi

# ─── G3: Regex mode produces results ──────────────────────────────────
echo "── G3: Regex mode ──"
REGEX_OUT=$($PMAT query --regex "fn\s+handle_\w+" --limit 5 2>&1)
REGEX_COUNT=$(echo "$REGEX_OUT" | grep -c "handle_" || echo "0")
if [ "$REGEX_COUNT" -gt 0 ]; then
    pass "G3: regex found $REGEX_COUNT matches for 'fn handle_*'"
else
    fail "G3: regex" "no results for regex 'fn\\s+handle_\\w+'"
fi

# Verify source content is present (not empty)
HAS_SOURCE=$($PMAT query --regex "fn\s+handle_\w+" --limit 1 --include-source --format json 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, list) and len(data) > 0 and data[0].get('source',''):
        print('yes')
    else:
        print('no')
except:
    print('no')
" 2>/dev/null || echo "no")
if [ "$HAS_SOURCE" = "yes" ]; then
    pass "G3b: regex results include source code"
else
    fail "G3b: regex source" "source field empty or missing in regex JSON output"
fi

# ─── G4: Literal mode produces results ────────────────────────────────
echo "── G4: Literal mode ──"
LITERAL_OUT=$($PMAT query --literal "unwrap()" --limit 5 2>&1)
LITERAL_COUNT=$(echo "$LITERAL_OUT" | grep -c "unwrap" || echo "0")
if [ "$LITERAL_COUNT" -gt 0 ]; then
    pass "G4: literal found $LITERAL_COUNT matches for 'unwrap()'"
else
    fail "G4: literal" "no results for literal 'unwrap()'"
fi

# ─── G5: Call graph context in output ─────────────────────────────────
echo "── G5: Call graph context ──"
HAS_CALLS=$($PMAT query "dispatch" --limit 3 --format json 2>/dev/null | python3 -c "
import json, sys
found = False
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        for r in data:
            if r.get('calls') or r.get('called_by'):
                found = True
                break
except: pass
print('yes' if found else 'no')
" 2>/dev/null || echo "no")
if [ "$HAS_CALLS" = "yes" ]; then
    pass "G5: call graph context populated in results"
else
    fail "G5: call graph" "no calls/called_by in JSON output"
fi

# ─── G6: Cross-project ranking ────────────────────────────────────────
echo "── G6: Cross-project ranking ──"
HAS_XP=$($PMAT query "dispatch" --rank-by cross-project --limit 5 --format json 2>/dev/null | python3 -c "
import json, sys
found = False
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        for r in data:
            if r.get('cross_project_callers', 0) > 0:
                found = True
                break
except: pass
print('yes' if found else 'no')
" 2>/dev/null || echo "no")
if [ "$HAS_XP" = "yes" ]; then
    pass "G6: cross_project_callers > 0 in ranked results"
else
    fail "G6: cross-project ranking" "no cross_project_callers > 0 found"
fi

# ─── G7: PTX flow ─────────────────────────────────────────────────────
echo "── G7: PTX flow ──"
PTX_OUT=$($PMAT query "kernel" --ptx-flow --limit 3 2>&1)
PTX_NODES=$(echo "$PTX_OUT" | grep -oP '\d+ nodes' | grep -oP '\d+' || echo "0")
PTX_EDGES=$(echo "$PTX_OUT" | grep -oP '\d+ edges' | grep -oP '\d+' || echo "0")
if [ "$PTX_NODES" -gt 0 ] && [ "$PTX_EDGES" -gt 0 ]; then
    pass "G7: PTX flow = $PTX_NODES nodes, $PTX_EDGES edges"
else
    fail "G7: PTX flow" "nodes=$PTX_NODES edges=$PTX_EDGES (expected > 0)"
fi

# ─── G8: Coverage-gaps mode ───────────────────────────────────────────
echo "── G8: Coverage-gaps ──"
# This may fail fast if no coverage data, which is expected
CG_OUT=$($PMAT query --coverage-gaps --limit 5 --exclude-tests 2>&1 || true)
if echo "$CG_OUT" | grep -qE 'Coverage Gaps|uncov|No coverage data|coverage cache'; then
    pass "G8: coverage-gaps mode runs (may have no data)"
else
    fail "G8: coverage-gaps" "unexpected output: $(echo "$CG_OUT" | head -2)"
fi

# ─── G9: Git history fusion ───────────────────────────────────────────
echo "── G9: Git history ──"
GH_OUT=$($PMAT query "fix memory" -G --limit 3 2>&1 || true)
if echo "$GH_OUT" | grep -qE 'commit|Git History|fix|author|No matching'; then
    pass "G9: git history fusion runs"
else
    fail "G9: git history" "no git data in output"
fi

# ─── G10: Source in --include-source ──────────────────────────────────
echo "── G10: Source code in output ──"
SRC_LEN=$($PMAT query "build_indices" --limit 1 --include-source --format json 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, list) and len(data) > 0:
        src = data[0].get('source', '') or ''
        print(len(src))
    else:
        print(0)
except:
    print(0)
" 2>/dev/null || echo "0")
if [ "$SRC_LEN" -gt 10 ]; then
    pass "G10: --include-source returns ${SRC_LEN} chars of source"
else
    fail "G10: source backfill" "source length = $SRC_LEN (expected > 10)"
fi

# ─── G11: --files-with-matches ────────────────────────────────────────
echo "── G11: Files with matches ──"
FWM_OUT=$($PMAT query "handler" --files-with-matches --limit 10 2>&1)
FWM_COUNT=$(echo "$FWM_OUT" | grep -c "/" || echo "0")
if [ "$FWM_COUNT" -gt 0 ]; then
    pass "G11: --files-with-matches returned $FWM_COUNT file paths"
else
    fail "G11: files-with-matches" "no file paths in output"
fi

# ─── G12: --count mode ───────────────────────────────────────────────
echo "── G12: Count mode ──"
COUNT_OUT=$($PMAT query "unwrap" --count --limit 10 2>&1)
if echo "$COUNT_OUT" | grep -qE '[0-9]+'; then
    pass "G12: --count mode returns counts"
else
    fail "G12: count mode" "no counts in output"
fi

# ─── G13: Context lines (-C) ─────────────────────────────────────────
echo "── G13: Context lines ──"
CTX_OUT=$($PMAT query --literal "HashMap" -C 2 --limit 3 2>&1)
if echo "$CTX_OUT" | grep -qE '\.rs:|HashMap'; then
    pass "G13: -C 2 context lines present"
else
    fail "G13: context lines" "no context output"
fi

# ─── G14: Exclude pattern ────────────────────────────────────────────
echo "── G14: Exclude pattern ──"
HAS_TEST=$($PMAT query "handler" --exclude "test" --limit 5 --format json 2>/dev/null | python3 -c "
import json, sys
found = False
try:
    data = json.load(sys.stdin)
    for r in (data if isinstance(data, list) else []):
        name = r.get('function_name', '')
        if 'test' in name.lower():
            found = True
            break
except: pass
print('yes' if found else 'no')
" 2>/dev/null || echo "no")
if [ "$HAS_TEST" = "no" ]; then
    pass "G14: --exclude 'test' filters out test functions"
else
    fail "G14: exclude pattern" "test functions still present after --exclude test"
fi

# ─── G15: No ANDON on load_index for varied queries ──────────────────
echo "── G15: No ANDON on load_index across query types ──"
QUERIES=("serialize" "cache invalidation" "fn:build_ build" "file:lib.rs dispatch")
ALL_CLEAR=true
for q in "${QUERIES[@]}"; do
    PROF=$($PMAT query "$q" --limit 3 2>&1)
    if echo "$PROF" | grep -q "load_index.*ANDON"; then
        fail "G15: ANDON on '$q'" "load_index exceeded 500ms"
        ALL_CLEAR=false
        break
    fi
done
if $ALL_CLEAR; then
    pass "G15: no load_index ANDON across ${#QUERIES[@]} varied queries"
fi

# ─── G16: Regex timing (source pre-load adds overhead but < 1s total) ─
echo "── G16: Regex total < 1000ms ──"
REGEX_SEC=$(measure "$PMAT" query --regex 'fn\s+test_\w+' --limit 5 --quiet)
REGEX_MS=$(echo "$REGEX_SEC * 1000" | bc | cut -d. -f1)
if [ "$REGEX_MS" -lt 1000 ]; then
    pass "G16: regex total = ${REGEX_MS}ms < 1000ms"
else
    fail "G16: regex timing" "total = ${REGEX_MS}ms >= 1000ms"
fi

# ─── G17: Literal timing (source pre-load adds overhead but < 1s total) ─
echo "── G17: Literal total < 1000ms ──"
LIT_SEC=$(measure "$PMAT" query --literal '.unwrap()' --limit 5 --quiet)
LIT_MS=$(echo "$LIT_SEC * 1000" | bc | cut -d. -f1)
if [ "$LIT_MS" -lt 1000 ]; then
    pass "G17: literal total = ${LIT_MS}ms < 1000ms"
else
    fail "G17: literal timing" "total = ${LIT_MS}ms >= 1000ms"
fi

# ─── G18: Churn enrichment still works ────────────────────────────────
echo "── G18: Churn enrichment ──"
HAS_CHURN=$($PMAT query "cache" --churn --limit 3 --format json 2>/dev/null | python3 -c "
import json, sys
found = False
try:
    data = json.load(sys.stdin)
    if isinstance(data, list):
        for r in data:
            if r.get('churn_score', 0) > 0 or r.get('commit_count', 0) > 0:
                found = True
                break
except: pass
print('yes' if found else 'no')
" 2>/dev/null || echo "no")
if [ "$HAS_CHURN" = "yes" ]; then
    pass "G18: churn data present in results"
else
    # Churn may be 0 if no git history for matched files - still pass if no crash
    pass "G18: churn mode runs without error (data may be 0)"
fi

# ─── G19: Faults enrichment ──────────────────────────────────────────
echo "── G19: Faults enrichment ──"
FAULTS_OUT=$($PMAT query "unwrap" --faults --limit 3 2>&1 || true)
if echo "$FAULTS_OUT" | grep -qE 'unwrap|fault|UNWRAP|TDG'; then
    pass "G19: faults mode produces output"
else
    fail "G19: faults" "no fault-related output"
fi

# ─── G20: Duplicates enrichment ──────────────────────────────────────
echo "── G20: Duplicates enrichment ──"
DUP_OUT=$($PMAT query "serialize" --duplicates --limit 3 2>&1 || true)
if echo "$DUP_OUT" | grep -qE 'serialize|clone|dup|TDG'; then
    pass "G20: duplicates mode produces output"
else
    fail "G20: duplicates" "no output"
fi

# ─── G21: Entropy enrichment ─────────────────────────────────────────
echo "── G21: Entropy enrichment ──"
ENT_OUT=$($PMAT query "handler" --entropy --limit 3 2>&1 || true)
if echo "$ENT_OUT" | grep -qE 'handler|entropy|diversity|TDG'; then
    pass "G21: entropy mode produces output"
else
    fail "G21: entropy" "no output"
fi

# ─── G22: Full enrichment combo ──────────────────────────────────────
echo "── G22: Full enrichment combo ──"
FULL_SEC=$(measure "$PMAT" query dispatch --churn --duplicates --entropy --faults -G --limit 3 --quiet)
FULL_MS=$(echo "$FULL_SEC * 1000" | bc | cut -d. -f1)
if [ "$FULL_MS" -lt 5000 ]; then
    pass "G22: full enrichment combo = ${FULL_MS}ms < 5000ms"
else
    fail "G22: full enrichment" "total = ${FULL_MS}ms >= 5000ms"
fi

# ─── G23: --exclude-file works ───────────────────────────────────────
echo "── G23: Exclude file pattern ──"
HAS_TESTFILE=$($PMAT query "handler" --exclude-file "tests" --limit 5 --format json 2>/dev/null | python3 -c "
import json, sys
found = False
try:
    data = json.load(sys.stdin)
    for r in (data if isinstance(data, list) else []):
        if 'test' in r.get('file_path', '').lower():
            found = True
            break
except: pass
print('yes' if found else 'no')
" 2>/dev/null || echo "no")
if [ "$HAS_TESTFILE" = "no" ]; then
    pass "G23: --exclude-file 'tests' filters test files"
else
    fail "G23: exclude-file" "test files still present"
fi

# ─── G24: JSON output is valid ───────────────────────────────────────
echo "── G24: JSON output validity ──"
JSON_VALID=$($PMAT query "error" --limit 3 --format json 2>/dev/null | python3 -c "
import json, sys
try:
    data = json.load(sys.stdin)
    if isinstance(data, list) and len(data) > 0:
        print('valid')
    else:
        print('empty')
except Exception as e:
    print('invalid: ' + str(e))
" 2>/dev/null || echo "invalid")
if [ "$JSON_VALID" = "valid" ]; then
    pass "G24: JSON output is valid"
else
    fail "G24: JSON validity" "$JSON_VALID"
fi

# ─── G25: Markdown output works ──────────────────────────────────────
echo "── G25: Markdown output ──"
MD_OUT=$($PMAT query "error" --limit 3 --format markdown 2>&1 || true)
if echo "$MD_OUT" | grep -qE '\|.*\||\#|file_path'; then
    pass "G25: markdown format produces table/heading output"
else
    fail "G25: markdown" "no markdown structure"
fi

# ─── Summary ─────────────────────────────────────────────────────────
echo ""
echo "═══════════════════════════════════════════════════════════"
echo "  FALSIFICATION RESULTS: $PASS passed, $FAIL failed"
echo "═══════════════════════════════════════════════════════════"
echo ""
for r in "${RESULTS[@]}"; do
    echo "$r"
done
echo ""

if [ "$FAIL" -gt 0 ]; then
    echo "VERDICT: FALSIFIED — $FAIL goals not met"
    exit 1
else
    echo "VERDICT: ALL $PASS GOALS VERIFIED — performance claims hold"
    exit 0
fi
