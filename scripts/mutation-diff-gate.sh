#!/usr/bin/env bash
#
# PMAT-630 / #1034 EV-4 — run mutation testing on a diff, and let the gate judge it.
#
# WHAT WAS MISSING
# ----------------
# Nothing in this repository asked whether a NEW test could actually fail. Three
# fixtures shipped in this release that avoided their own defect — the dead-code
# parity test compared only dead FUNCTIONS on only a BIN crate, the two
# conditions under which the analyzers agree; CB-2100's hop fixtures both used a
# bare `run: make comply`, the one shape in which the suppressed-hop bug cannot
# appear. All were green. Coverage says a line ran; only mutation says its
# behaviour is pinned.
#
# `src/services/mutation_gate.rs` already holds the verdict — INV-MUT-1/2/3,
# fail-closed, with a fixture suite behind it. What it did not have was a
# caller: no CLI subcommand, no Makefile target, no workflow step ever produced
# a `mutants.out/` for it to read or asked it for an answer. `cargo mutants` was
# in ci.yml and three Makefile targets; `--in-diff` was in none of them.
#
# This script is the missing producer. It is deliberately NOT a second copy of
# the rules: it resolves the diff, clears the artifact, runs cargo-mutants, and
# hands the result to the Rust gate through `examples/mutation_gate.rs`. Two
# implementations of a verdict is two things to keep in sync, and the copy CI
# does not run is the one that drifts.
#
# THE ONE RULE THAT LIVES HERE
# ----------------------------
# Clearing `mutants.out/` before the run, and it is not cosmetic. Measured
# against cargo-mutants 27.0.0: when `--in-diff` selects no mutants the tool
# prints `INFO No mutants to filter`, exits 0, writes no report — and leaves any
# PREVIOUS `mutants.out/` exactly where it was. Judge that directory and you
# judge an unrelated earlier run, which may well say everything was caught. The
# Rust gate fails closed on a missing artifact; it cannot detect a stale one
# that looks perfectly healthy. So the producer deletes it, every time.
#
# There is no --skip, no --allow, no threshold, and no `|| true`. If this is too
# slow for a context it moves to a context where it can run whole; it does not
# get softened. See .github/workflows/mutation-diff.yml for where it runs.
#
# Usage:
#   scripts/mutation-diff-gate.sh run     [--base <ref>] [--work <dir>]
#   scripts/mutation-diff-gate.sh verdict [--diff <file>]
#
# `verdict` re-judges an existing `mutants.out/` without re-running anything.

set -euo pipefail

PROG="$(basename "$0")"

die() {
    printf '%s: FAIL: %s\n' "$PROG" "$*" >&2
    exit 1
}

note() {
    printf '%s: %s\n' "$PROG" "$*"
}

# The verdict, in every path: the compiled Rust gate, never a shell reading JSON.
judge() {
    local diff="$1"
    note "handing mutants.out/ to services::mutation_gate"
    cargo run --quiet --example mutation_gate -- --project . --diff "$diff"
}

cmd_verdict() {
    local diff=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --diff) diff="$2"; shift 2 ;;
            *) die "unknown argument to verdict: $1" ;;
        esac
    done
    [ -f "$diff" ] || die "verdict needs --diff pointing at a readable diff (got '$diff'); an unknown scope is not a pass"
    judge "$diff"
}

cmd_run() {
    local base="${MUTATION_GATE_BASE:-origin/master}"
    local work=""
    while [ $# -gt 0 ]; do
        case "$1" in
            --base) base="$2"; shift 2 ;;
            --work) work="$2"; shift 2 ;;
            *) die "unknown argument to run: $1" ;;
        esac
    done

    # Everything below is relative to the crate root: the artifact cargo-mutants
    # writes, the `--project .` the gate reads it from, and `--in-place`'s notion
    # of the tree. Running from a subdirectory would clear one `mutants.out` and
    # judge another.
    cd "$(git rev-parse --show-toplevel)" ||
        die "not inside a git repository, so there is no diff to gate"

    command -v cargo >/dev/null 2>&1 || die "cargo is not on PATH"
    cargo mutants --version >/dev/null 2>&1 ||
        die "cargo-mutants is not installed; install it with 'cargo install cargo-mutants --locked'"

    [ -n "$work" ] || work="$(mktemp -d)"
    mkdir -p "$work"

    local mergebase
    mergebase="$(git merge-base "$base" HEAD)" ||
        die "cannot find a merge base with '$base' — fetch it first; a gate that cannot see the diff must not pass"
    note "diffing ${mergebase}..HEAD (base ${base})"

    local diff="$work/pr.diff"
    git diff "$mergebase" HEAD >"$diff"

    # See the header: cargo-mutants does not clear this itself, and a stale
    # all-caught report is indistinguishable from a real one.
    rm -rf mutants.out mutants.out.old

    local rc=0
    set +e
    # `--cap-lints true` is load-bearing here, and was measured, not guessed.
    # `src/lib.rs` carries `#![deny(unused_variables)]`. Replacing a function
    # body — cargo-mutants' commonest mutation — leaves that function's
    # parameters unused, so the mutant does not COMPILE and is reported
    # `unviable` rather than tested. Without this flag the first real run on this
    # crate produced 3 mutants, 3 unviable, 0 executed: cargo-mutants exits 0 for
    # that, the gate correctly refuses to call it a pass (nothing was tested),
    # and no diff could ever go green. A denied lint is a property of this
    # crate's style, not of the change under test, and must not decide whether a
    # mutant can be killed.
    cargo mutants \
        --in-place --no-times --colors never --cap-lints true \
        --in-diff "$diff" \
        --timeout-multiplier "${MUTATION_GATE_TIMEOUT_MULTIPLIER:-1.5}" \
        --minimum-test-timeout "${MUTATION_GATE_MIN_TIMEOUT:-900}" \
        -- --lib
    rc=$?
    set -e

    # 0 = every mutant caught, 2 = something survived. Both are results the gate
    # can judge. Anything else (1 usage, 3 clean tests failed, 4 baseline
    # timeout, 5 internal) means the tool did not finish, which is not the same
    # as the code passing — and cargo-mutants also exits 0 when every mutant was
    # unviable, which is why the verdict is never taken from this number.
    case "$rc" in
        0 | 2) : ;;
        *) die "cargo mutants exited $rc, so the run did not complete; see mutants.out/debug.log" ;;
    esac

    judge "$diff"
}

main() {
    [ $# -ge 1 ] || die "usage: $PROG run|verdict [options]"
    local sub="$1"
    shift
    case "$sub" in
        run) cmd_run "$@" ;;
        verdict) cmd_verdict "$@" ;;
        *) die "unknown subcommand '$sub' (expected run or verdict)" ;;
    esac
}

main "$@"
