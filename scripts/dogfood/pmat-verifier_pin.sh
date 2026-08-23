# verifier_pin.sh — THE rule that says which binary a release gate is allowed to
# believe, and the two pins that implement it.
#
# Source it, never execute it:
#     . "$SKILL_DIR/verifier_pin.sh" || exit 2
#     verifier_pin_pmat "$CRATE" "$BINPATH"   # sets+EXPORTS PMAT_BIN; 0 pinned,
#                                             # 1 releasing pmat with no working artifact
#     verifier_pin_pv                         # sets+EXPORTS PV; 0 pinned, 1 broken, 2 unpinned
#
# Both pins EXPORT their result: the gates this protocol discovers from
# [package.metadata.dogfood] run as CHILD processes, and a shell-local pin is a
# pin delivered to nobody — pin_audit would certify consumption the environment
# never carried (#2644 audit, VPIN-4). Both pins verify BEHAVIOR, not existence:
# `-x` accepts a broken stub and even a directory; only the binary answering
# `--version` is evidence it can verify anything (#2644, VPIN-2).
#
# ── THE RULE ────────────────────────────────────────────────────────────────
#
#   A gate that decides a release must not resolve its verifier through PATH.
#   Where the repo pins the tool, use the pin; where it does not, REPORT rather
#   than fall back — a skipped gate that says so beats a green one measured with
#   an unknown binary.
#
# It is stated HERE, once, and nowhere else. Every other file that needs it
# points at this one. #2640 exists because the same rule had been rediscovered
# FIVE times, each time as a local fix that did not know about the other four:
#
#   1  PMAT_BIN (user-scope dogfood runner)  a gate ran a pmat that predated
#      CB-200 becoming a ratchet, and Failed a tree the shipped code passes
#   2  scripts/pv_bin.sh                     PATH pv 0.49.0 vs in-tree 0.63.0
#      disagreed on the gate that DECIDES the release
#   3  scripts/apr_bin.sh                    four `apr` binaries coexisted; a
#      bare `apr` resolved to a 26-day-old copy
#   4  aprender#2384                         MCP spawned a bare `apr`, ran
#      0.60.0 while reporting 0.63.0
#   5  APR-BENCH-RFC-001                     unpinned llama.cpp comparator
#
# Five rediscoveries is the evidence that a rule merely STATED is documentation.
# It is therefore also ENFORCED, by scripts/check_verifier_pinning.sh, which
# fails on any bare pv/pmat/apr in command position in the runner — and which
# proves the two pins BEHAVE, not merely that they are present.
#
# OPTION-NEUTRAL BY CONSTRUCTION: this file sets no shell options. `set -euo
# pipefail` in a SOURCED file mutates the CALLER's shell — that leak once killed
# the nightly six lines in (CLAUDE.md; scripts/check_sourced_libs_option_neutral.sh).
# Failure is signalled by RETURN STATUS only.

# ── WHICH pmat runs the pmat gates ──────────────────────────────────────────
#
# `pmat verify` and `pmat comply check` are normally the INSTALLED pmat applied
# to whatever crate is under test — that is the point of a fleet quality tool.
# But when the crate under test IS pmat, that is the one case where it is wrong:
# the gate then measures a DIFFERENT BUILD than the one being released.
#
# Measured, 2026-08-22, releasing pmat 3.32.0:
#   installed ~/.cargo/bin/pmat   version 3.32.0   commit 8134bb373
#   built from the release tree   version 3.32.0   commit 7a7409e03
# Both print "3.32.0". Only the commit line differs, and nothing in the receipt
# showed it. The installed binary predated CB-200 becoming a ratchet — the
# string "recorded baseline" occurs 0 times in it and 4 times in the new one —
# so the release gate ran the OLD zero-tolerance check and returned Fail against
# a tree the shipped code passes. A gate validating a release with a stale copy
# of the thing being released is the same defect class this protocol exists to
# find, sitting inside the protocol.
#
# So: for pmat, the pmat gates use the artifact just built. For every other
# crate, PATH is correct and is what still happens.
# Returns: 0 = pinned (the fleet pmat for a non-pmat crate, the behavior-verified
#              built artifact when the crate IS pmat)
#          1 = releasing pmat with a missing or non-working artifact — PMAT_BIN is
#              left EMPTY and the caller must fail closed. The old behavior fell
#              back to the PATH pmat silently, which is the recorded incident
#              above happening again with this file watching (#2644, VPIN-1).
#
# The behavior check is `--version` answering with SOMETHING, deliberately not a
# version-equality assert: the measurement above is two binaries both printing
# "3.32.0" while running different code. A version string cannot discriminate
# builds; a binary that cannot even print one cannot verify anything.
verifier_pin_pmat() {
    verifier_pin_crate="${1:-}"
    verifier_pin_built="${2:-}"
    PMAT_BIN=pmat
    export PMAT_BIN
    if [ "$verifier_pin_crate" = "pmat" ]; then
        if [ -z "$verifier_pin_built" ]; then
            PMAT_BIN=""
            export PMAT_BIN
            return 1
        fi
        verifier_pin_ver=""
        if [ -f "$verifier_pin_built" ]; then
            verifier_pin_ver=$("$verifier_pin_built" --version 2>/dev/null) || verifier_pin_ver=""
        fi
        if [ -z "$verifier_pin_ver" ]; then
            PMAT_BIN=""
            export PMAT_BIN
            return 1
        fi
        PMAT_BIN="$verifier_pin_built"
        export PMAT_BIN
    fi
    return 0
}

# ── WHICH pv validates the contracts ────────────────────────────────────────
#
# pv is PINNED, never PATH-resolved. scripts/pv_bin.sh records why: a PATH `pv`
# was 0.49.0 while the in-tree crate was 0.63.0, and the two disagreed on the
# gate that decides the release -- strict-test-binding reported 253 refs / 51
# missing under the stale binary and 371 / 27 under HEAD. The dogfood runner IS
# a surface where the release decision is made, so it must not be the one
# holding the stale binary.
#
# This protocol runs against any repo in the fleet, and not all of them carry
# pv_bin.sh. Where it is absent, pv is left UNRESOLVED and the contract gates
# REPORT that rather than falling back to whatever PATH offers. A skipped gate
# that says so beats a green one measured with an unknown binary.
#
# The subshell is load-bearing and is the one deviation from the copy this was
# merged from. pv_bin.sh ends in `PV=$(...) || return 1 2>/dev/null || exit 1`.
# Sourced at the top level of a function body, `return` would unwind the CALLER;
# sourced inside a subshell, a failure exits only the subshell and this function
# still gets to report it. The pin is thereby usable from a function, which is
# what lets check_verifier_pinning.sh exercise it in isolation instead of taking
# its presence on trust.
#
# Pin discovery anchors to the REPO, not the caller's cwd: `[ -f scripts/... ]`
# from a subdirectory of a repo that DOES ship the pin returned 2, whose
# documented meaning is "this repo ships no pin" — a false statement that
# downgraded every pv gate to REPORT (#2644, VPIN-5). git names the root; a
# non-git directory falls back to cwd, where the relative form was already
# correct.
#
# The subshell also CLEARS the PV_BIN environment channel before sourcing:
# pv_bin.sh honors an inherited PV_BIN and skips the cargo build it calls "THE
# FRESHNESS AUTHORITY", so an exported stale path would ride straight through
# the pin (#2644, VPIN-6/VP-04). Inside the subshell the unset is contained —
# the caller's environment is untouched, per the option-neutral rule above.
#
# On failure the diagnostics pv_bin.sh wrote are no longer swallowed (#2644,
# VPIN-3): they land in a kept tempfile whose tail is printed to stderr, so
# rc=1 arrives saying WHICH stage failed instead of arriving mute.
#
# Returns: 0 = pinned, exported, and behavior-verified (`--version` answered)
#          1 = pin present but FAILED to resolve (diagnostics on stderr)
#          2 = this repo ships no pin (report, never fall back to PATH)
verifier_pin_pv() {
    PV=""
    export PV
    verifier_pin_root=$(git rev-parse --show-toplevel 2>/dev/null) || verifier_pin_root=""
    [ -n "$verifier_pin_root" ] || verifier_pin_root=$PWD
    [ -f "$verifier_pin_root/scripts/pv_bin.sh" ] || return 2
    verifier_pin_pv_log=$(mktemp) || return 1
    # The subshell is load-bearing (see the note above); the `if` wrapper keeps
    # a failing command substitution from killing an errexit caller before the
    # diagnostics below can be printed.
    if PV=$( cd "$verifier_pin_root" && unset PV_BIN \
             && . ./scripts/pv_bin.sh >"$verifier_pin_pv_log" 2>&1 \
             && printf '%s' "$PV" ); then :; fi
    verifier_pin_ver=""
    if [ -n "$PV" ] && [ -f "$PV" ]; then
        verifier_pin_ver=$("$PV" --version 2>/dev/null) || verifier_pin_ver=""
    fi
    if [ -n "$verifier_pin_ver" ]; then
        export PV
        rm -f "$verifier_pin_pv_log"
        return 0
    fi
    PV=""
    export PV
    {
        echo "verifier_pin_pv: the pin failed to resolve a working pv."
        echo "  pv_bin.sh diagnostics (kept at $verifier_pin_pv_log):"
        tail -15 "$verifier_pin_pv_log" 2>/dev/null | sed 's/^/    /'
    } >&2
    return 1
}
