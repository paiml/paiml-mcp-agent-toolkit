#!/usr/bin/env bash
# Dogfood: run the built pmat against real trees and assert it agrees with an
# INDEPENDENT measurement.
#
# Why this exists, concretely. Every check below is a bug that actually shipped,
# past a green CI and ~20,000 passing tests, because pmat was only ever checked
# against its own report:
#
#   * `analyze clippy` returned "total_diagnostics": 0 and "Clippy fixes applied
#     successfully" on a crate `cargo clippy` gives 76 warnings for. The counts
#     were taken AFTER a confidence filter, so "none were auto-fixable" and "the
#     crate is clean" rendered identically.
#   * `pmat tdg` graded a directory it could not read one file of as F at exit 0.
#   * `analyze complexity`, `entropy` and `defect-prediction` reported on zero
#     files and exited 0.
#
# The common shape is a tool that cannot tell "measured and clean" from "not
# measured". A unit test cannot catch it, because the fixture and the code share
# an author and confirm each other. Only an external oracle — cargo clippy
# itself, the filesystem itself — can break that loop.
#
# So: a MISSING TOOL IS A FAILURE, never a skip. A dogfood run that quietly
# skips its own dependency proves nothing, which is the exact failure this
# script exists to detect.

set -uo pipefail

REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="${BIN:-}"
WORK="${WORK:-$(mktemp -d "${TMPDIR:-/tmp}/pmat-dogfood-XXXXXX")}"
FAILURES=0
CHECKS=0

say()  { printf '  %s\n' "$*"; }
pass() { CHECKS=$((CHECKS + 1)); printf '  \033[32m✓\033[0m %s\n' "$*"; }
fail() { CHECKS=$((CHECKS + 1)); FAILURES=$((FAILURES + 1)); printf '  \033[31m✗\033[0m %s\n' "$*"; }

# ── The binary under test. Resolved, never assumed: a stale `target/debug/pmat`
# left by an earlier build has more than once been tested in place of the code
# actually under change.
if [ -z "$BIN" ]; then
    BIN="$REPO/target/debug/pmat"
    if [ ! -x "$BIN" ]; then
        echo "❌ no binary at $BIN and \$BIN unset — build first (cargo build --features full)"
        exit 1
    fi
fi
say "binary:  $BIN"
say "version: $("$BIN" --version 2>/dev/null | head -1)"
say "scratch: $WORK"

# ── External oracles. Absent means FAIL.
for tool in cargo git; do
    command -v "$tool" >/dev/null 2>&1 || {
        echo "❌ $tool is not installed. This gate compares pmat against real"
        echo "   external tools; without them it would 'pass' by measuring nothing."
        exit 1
    }
done

# Remove the scratch tree, and NEVER anything else. `rm -rf "$WORK"` with an
# unset or empty WORK expands to `rm -rf` in the cwd, and with WORK=/ it is
# exactly the accident this guard exists to make impossible (bashrs SEC011).
cleanup() {
    # The destructive call is reachable ONLY from a branch that matched the
    # scratch-directory pattern. An empty, root, $HOME or repo-root WORK does not
    # match and therefore cannot reach `rm` at all — the guard is structural, not
    # a check placed before it that a later edit could step over.
    case "${WORK:-}" in
        "${TMPDIR:-/tmp}"/pmat-dogfood-*|/tmp/pmat-dogfood-*)
            [ -d "$WORK" ] && { rm -rf -- "$WORK" || printf 'could not remove %s\n' "$WORK" >&2; }
            ;;
        *)
            printf 'refusing to remove WORK=%s (not a pmat-dogfood scratch dir)\n' \
                "${WORK:-<empty>}" >&2
            ;;
    esac
}

# A fixture crate: manifest, gitignore and git history. The caller writes
# src/lib.rs itself, so no nested command substitution is needed to generate it.
new_crate() { # new_crate <dir> <name>
    mkdir -p "$1/src"
    printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n' "$2" >"$1/Cargo.toml"
    printf 'target/\n.pmat/\n' >"$1/.gitignore"
}

git_seal() { # git_seal <dir> — real history, because churn-aware analysers read it
    ( cd "$1" && git init -q && git add -A \
        && git -c user.email=t@t -c user.name=t commit -q --no-verify -m init ) >/dev/null 2>&1
}

echo
echo "── 1. analyze clippy must agree with cargo clippy"
# The 3.32.0 bug: 76 real warnings reported as 0. The oracle is clippy itself.
NOISY="$WORK/noisy"
new_crate "$NOISY" noisy
{
    printf '// TODO: hack\n'
    i=1
    while [ "$i" -le 12 ]; do
        printf 'pub fn tangled_%s(a: i32, b: i32) -> i32 { if a > %s { if b > %s { return %s; } } 0 }\n' "$i" "$i" "$i" "$i"
        i=$((i + 1))
    done
    printf 'fn never_used() -> i32 { 42 }\n'
} >"$NOISY/src/lib.rs"
git_seal "$NOISY"
REAL=$( cd "$NOISY" && CARGO_TARGET_DIR="$NOISY/t" cargo clippy --message-format=json 2>/dev/null \
        | grep -c '"reason":"compiler-message"' || true )
PMAT_JSON=$("$BIN" analyze clippy --path "$NOISY" 2>/dev/null || true)
FOUND=$(printf '%s' "$PMAT_JSON" | python3 -c 'import json,sys
try: print(json.load(sys.stdin).get("diagnostics_found", "MISSING"))
except Exception: print("UNPARSABLE")' 2>/dev/null)
say "cargo clippy compiler-messages: $REAL   |   pmat diagnostics_found: $FOUND"
if [ "$REAL" -eq 0 ]; then
    fail "the oracle itself found nothing — fixture is not exercising clippy, so this check proves nothing"
elif [ "$FOUND" = "MISSING" ] || [ "$FOUND" = "UNPARSABLE" ]; then
    fail "pmat did not report diagnostics_found at all (payload changed?)"
elif [ "$FOUND" -eq 0 ]; then
    fail "pmat reports 0 diagnostics where cargo clippy reports $REAL — the 3.32.0 defect"
else
    pass "pmat sees the diagnostics clippy sees ($FOUND vs $REAL)"
fi

echo
echo "── 2. an unreadable tree is REFUSED, not graded"
COBOL="$WORK/cobol"; mkdir -p "$COBOL"
printf 'IDENTIFICATION DIVISION.\n' >"$COBOL/legacy.cbl"
for cmd in "tdg $COBOL" "analyze complexity --path $COBOL"; do
    # shellcheck disable=SC2086
    OUT=$("$BIN" $cmd 2>&1); CODE=$?
    if [ "$CODE" -eq 0 ]; then
        fail "pmat $cmd exited 0 on a tree it can read no file of"
    elif printf '%s' "$OUT" | grep -qE '\(F\)|Grade: F'; then
        fail "pmat $cmd graded an unreadable tree F instead of refusing"
    else
        pass "pmat $cmd refuses (exit $CODE)"
    fi
done

echo
echo "── 3. a path that does not exist is REFUSED"
for sub in satd complexity duplicates; do
    "$BIN" analyze "$sub" --path "$WORK/definitely-not-here" >/dev/null 2>&1
    CODE=$?
    if [ "$CODE" -eq 0 ]; then
        fail "analyze $sub exited 0 for a nonexistent path"
    else
        pass "analyze $sub refuses a nonexistent path (exit $CODE)"
    fi
done

echo
echo "── 4. machine-readable output carries no colour"
# An escape inside a JSON string is a parse failure, not a decoration.
CLEAN="$WORK/clean"
new_crate "$CLEAN" clean
{
    printf '//! Tiny.\n\n'
    printf '/// Adds.\n'
    printf 'pub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n'
} >"$CLEAN/src/lib.rs"
git_seal "$CLEAN"
for sub in complexity satd dead-code; do
    if "$BIN" --color always analyze "$sub" --path "$CLEAN" --format json 2>/dev/null | grep -q $'\033'; then
        fail "analyze $sub --format json contains ANSI under --color always"
    else
        pass "analyze $sub --format json is escape-free under --color always"
    fi
done

echo
echo "── 5. --color always actually colours the human output"
# The inverse of check 4, so "emit no colour ever" cannot pass both.
for sub in complexity satd dead-code; do
    if "$BIN" --color always analyze "$sub" --path "$CLEAN" 2>/dev/null | grep -q $'\033'; then
        pass "analyze $sub honours --color always"
    else
        fail "analyze $sub emits no colour under --color always"
    fi
done

echo
echo "── 6. a clean tree is still reported clean"
# The counter-test for the whole file: without it, a pmat that refuses
# everything would pass checks 2 and 3.
OUT=$("$BIN" analyze complexity --path "$CLEAN" 2>&1); CODE=$?
if [ "$CODE" -ne 0 ]; then
    fail "analyze complexity failed on a clean fixture (exit $CODE): $(printf '%s' "$OUT" | head -1)"
else
    pass "analyze complexity succeeds on a readable tree"
fi

echo
printf '  %s checks, %s failure(s)\n' "$CHECKS" "$FAILURES"
if [ "$FAILURES" -ne 0 ]; then
    echo "❌ dogfood-use FAILED — pmat disagrees with reality on $FAILURES check(s)"
    echo "   scratch kept for inspection: $WORK"
    exit 1
fi
cleanup
echo "✅ dogfood-use passed"
