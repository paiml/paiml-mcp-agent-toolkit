# pv_bin.sh — resolve THE pv built from HEAD, and prove it.
#
# Source it, never execute it:
#     . scripts/pv_bin.sh || exit 1
#     "$PV" lint contracts/
#
# WHY THIS EXISTS. `pv` on PATH was 0.49.0 while the in-tree crate was 0.63.0,
# and the two disagreed on the gate that matters: strict-test-binding reported
# 253 refs / 51 missing under the stale binary and 371 / 27 under HEAD. Both
# surfaces where the RELEASE decision is made were using the stale one:
#   scripts/dogfood_surfaces.sh  printed `pv present (pv 0.49.0)` into the
#                                release receipt AS EVIDENCE OF CORRECTNESS
#   Makefile `contracts:`        ran a bare `pv lint contracts/` as the gate
#
# This is the same defect the repo already solved for `apr` (CLAUDE.md "Step 0 —
# pin the binary, ALWAYS"; four apr binaries once coexisted and a bare `apr`
# resolved to a 26-day-old copy). Same remedy, same shape as scripts/apr_bin.sh.
#
# CARGO IS THE FRESHNESS AUTHORITY, not a version string. A version match is not
# freshness: during this work pv was rebuilt three times at the SAME version with
# two distinct md5s, and the two binaries gave different answers on an identical
# tree. So we `cargo build` first and take the artifact cargo produces; the
# version assert below is a second line of defence against a PATH fallback, not
# the primary proof.
#
# OPTION-NEUTRAL BY CONSTRUCTION: this file sets no shell options. `set -euo
# pipefail` in a SOURCED file mutates the CALLER's shell — that leak once killed
# the nightly six lines in (see CLAUDE.md, scripts/check_sourced_libs_option_neutral.sh).
# Failure is signalled by RETURN STATUS only.

pv_bin_die() {
    printf 'pv_bin: %s\n' "$*" >&2
    return 1
}

pv_bin_root() {
    git rev-parse --show-toplevel 2>/dev/null || pwd
}

# Build from HEAD and hand back the artifact cargo produced.
pv_bin_resolve() {
    if [ -n "${PV_BIN:-}" ]; then
        printf '%s\n' "$PV_BIN"
        return 0
    fi
    pv_bin_root_dir=$(pv_bin_root)
    ( cd "$pv_bin_root_dir" && cargo build -q -p aprender-contracts-cli --bin pv ) >&2 \
        || { pv_bin_die "cargo build of aprender-contracts-cli failed"; return 1; }
    pv_bin_td=$( cd "$pv_bin_root_dir" \
        && cargo metadata --no-deps --format-version 1 2>/dev/null \
        | jq -r '.target_directory // empty' 2>/dev/null )
    [ -n "$pv_bin_td" ] || { pv_bin_die "could not read cargo target_directory"; return 1; }
    for pv_bin_cand in "$pv_bin_td/debug/pv" "$pv_bin_td/release/pv"; do
        [ -x "$pv_bin_cand" ] && { printf '%s\n' "$pv_bin_cand"; return 0; }
    done
    pv_bin_die "no pv binary under $pv_bin_td after a successful build"
    return 1
}

# Second line of defence: the resolved binary must report the version the tree
# declares. Catches a PATH fallback or a hand-copied artifact.
pv_bin_assert_fresh() {
    pv_bin_b="$1"
    [ -x "$pv_bin_b" ] || { pv_bin_die "not executable: $pv_bin_b"; return 1; }
    pv_bin_declared=$(awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' \
        "$(pv_bin_root)/crates/aprender-contracts-cli/Cargo.toml" 2>/dev/null)
    if [ -z "$pv_bin_declared" ]; then
        pv_bin_declared=$(awk -F'"' '/^version[[:space:]]*=/{print $2; exit}' \
            "$(pv_bin_root)/Cargo.toml" 2>/dev/null)
    fi
    [ -n "$pv_bin_declared" ] || { pv_bin_die "could not read declared pv version"; return 1; }
    # POSITIONAL, first line only. `pv --version` is deliberately multi-line as
    # of #2559 — it has to say WHICH pv this is, because four things claim that
    # name. The old `awk '{print $NF}'` took the last field of EVERY line and
    # handed this comparison four lines of prose. The version line's shape is
    # pinned from the other side by
    # crates/aprender-contracts-cli/tests/version_identity.rs
    # (`semver_stays_the_second_field_of_the_first_line`), and by the case table
    # in scripts/check_pv_version_parse.sh.
    # ONE invocation, then both reads off the SAME captured text -- the semver
    # and the identity must describe the same binary and the same run.
    pv_bin_vers=$("$pv_bin_b" --version 2>&1)
    pv_bin_actual=$(printf '%s\n' "$pv_bin_vers" | awk 'NR==1{print $2; exit}')
    pv_bin_first=$(printf '%s\n' "$pv_bin_vers" | awk 'NR==1{print; exit}')
    [ "$pv_bin_actual" = "$pv_bin_declared" ] || {
        pv_bin_die "resolved pv reports $pv_bin_actual, tree declares $pv_bin_declared ($pv_bin_b)"
        return 1
    }
    # IDENTITY, on the same line. #2559 added an identity string to `pv
    # --version` precisely because four things claim the name `pv` -- but this
    # function, the place the release actually DECIDES whether it has the right
    # binary, went on proving freshness from the SEMVER alone. A semver is not
    # an identity: pv(1) the pipe viewer ships 0.x versions too, and a stale
    # sibling `pv` that happened to match the declared version would have
    # satisfied the check above unchanged. So the marker is asserted here, where
    # the decision is made, not only in the unit tests that watch the string.
    case "$pv_bin_first" in
        *"(aprender provable-contracts verifier)"*) ;;
        *)
            pv_bin_die "resolved pv does not identify as the aprender contracts verifier. First --version line: $pv_bin_first ($pv_bin_b). If that is pv(1) the pipe viewer, or another crate named pv, it is the wrong binary however its version reads."
            return 1
            ;;
    esac
    return 0
}

PV=$(pv_bin_resolve) || return 1 2>/dev/null || exit 1
pv_bin_assert_fresh "$PV" || return 1 2>/dev/null || exit 1
export PV
