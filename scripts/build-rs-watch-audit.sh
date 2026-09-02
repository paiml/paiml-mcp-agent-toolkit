#!/usr/bin/env bash
# CRUX-06 — build.rs rerun-if-changed hygiene audit.
#
# A `cargo:rerun-if-changed=` path that does not exist, or that points outside
# CARGO_MANIFEST_DIR, makes cargo treat build.rs as permanently stale, so the
# whole crate relinks on every invocation. `../assets/demo/` did exactly that
# until 3.36.0: 55 s / 265 CPU-seconds per no-op `cargo build --release`.
#
# Legs 1-4 audit build.rs; leg 5 mutation-tests the auditor against six planted
# dodges and one real fix (so the gate itself cannot rot silently); leg 6 is the
# verdict on this tree; leg 7 cross-checks cargo's own recorded fingerprint and
# needs one completed release build to arm (it fails loudly, never skips).
#
# The `--lib` test `rerun_if_changed_paths_exist_inside_the_tree` in
# build_support.rs carries legs 1-4 as a permanent gate; this script is the
# full, mutation-tested form for release evidence. Requires GNU grep -P, jq.
#
# Spec: docs/specifications/pmat-architecture-crux-audit.md §8.6
set -euo pipefail
fail(){ echo "FAIL: $*" >&2; exit 1; }

# audit <tree> — prints one finding per line for every rerun-if-changed defect in
# <tree>/build.rs and nothing at all when it is clean. Legs 1-4 live here so leg 5
# can replay them against planted mutants.
audit() {
  t=$1
  raw=$(grep -oP 'cargo:rerun-if-changed=\K[^"]*' "$t/build.rs") \
       || { echo "EXTRACTOR-ROTTED:no-directives"; return 0; }
  lit=$(printf '%s\n' "$raw" | grep -v '[{$]' | sort -u)
  n=$(printf '%s\n' "$lit" | grep -c . ) || n=0
  # leg 1 CONTROL — green today AND after the fix. Anti-vacuity only: a rotted
  # regex must fail rather than certify. NOT evidence of the defect. `sort -u`
  # first, so duplicate lines cannot pad the floor.
  [ "$n" -ge 8 ] || echo "EXTRACTOR-ROTTED:only-${n}-distinct-literals"
  # leg 2 INVARIANT — required watches by NAME, so a deleted watch cannot be
  # padded back with a duplicate of another. Replaces the old ">= 9" census.
  for req in assets/vendor/ assets/demo/ templates/ \
             src/schema/refactor_state.capnp contracts/binding.yaml \
             mcp_tool_schemas .git/HEAD .git/index; do
    printf '%s\n' "$lit" | grep -qxF "$req" || echo "WATCH-REMOVED:$req"
  done
  # leg 3 — exactly one interpolated directive and it is the schema walk. Matches
  # interpolation ANYWHERE in the directive, not just immediately after '='.
  dynl=$(grep -n 'cargo:rerun-if-changed=[^"]*[{$]' "$t/build.rs" || true)
  dynn=$(printf '%s\n' "$dynl" | grep -c . ) || dynn=0
  [ "$dynn" -eq 1 ] || echo "INTERPOLATED-DIRECTIVES:$dynn"
  printf '%s\n' "$dynl" | grep -q 'path\.display()' || echo "UNKNOWN-INTERPOLATED-DIRECTIVE"
  printf '%s\n' "$dynl" | sed -n 's/.*cargo:rerun-if-changed=\([^{$]*\)[{$].*/\1/p' \
    | while read -r pre; do
        case "$pre" in /*|*..*) echo "ESCAPES-MANIFEST-DIR:${pre}{}";; esac
      done
  # leg 4 — THE DEFECT DETECTOR. Every literal path must be manifest-dir-relative
  # (SHAPE, checked before existence) AND present under the tree under test.
  printf '%s\n' "$lit" | while read -r p; do
    [ -n "$p" ] || continue
    case "$p" in
      /*|*..*) echo "ESCAPES-MANIFEST-DIR:$p" ;;
      *) [ -e "$t/$p" ] || echo "MISSING:$p" ;;
    esac
  done
}

root=$(git rev-parse --show-toplevel) || fail "leg 0: not run from inside the checkout"
test -f "$root/build.rs" || fail "leg 0: no build.rs at $root"

# ---- leg 5 CONTROL (green today by construction): the auditor is mutation-tested.
sb=$(mktemp -d) || fail "leg 5: mktemp"
cleanup() {
  # Remove the sandbox mktemp handed us and NEVER anything else: an unset $sb
  # would expand `rm -rf` to the cwd and $sb=/ would be catastrophic, so the
  # path must match the shape mktemp -d produces before rm sees it.
  case "${sb:-}" in
    "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*)
      [ -d "$sb" ] && { rm -rf -- "$sb" || printf 'could not remove %s\n' "$sb" >&2; } ;;
  esac
}
trap cleanup EXIT
plant() {  # plant <name> -> echoes a sandbox whose repo/ has the real skeleton
  d=$sb/$1; mkdir -p "$d/repo/assets/vendor" "$d/repo/assets/demo" "$d/repo/templates" \
     "$d/repo/src/schema" "$d/repo/contracts" "$d/repo/mcp_tool_schemas" "$d/repo/.git"
  : >"$d/repo/src/schema/refactor_state.capnp"; : >"$d/repo/contracts/binding.yaml"
  : >"$d/repo/.git/HEAD"; : >"$d/repo/.git/index"
  cp "$root/build.rs" "$d/repo/build.rs"; echo "$d"
}
reject(){ [ -n "$(audit "$2/repo")" ] || fail "leg 5: mutant $1 ACCEPTED — the gate is gameable by it"; }
accept(){ [ -z "$(audit "$2/repo")" ] || fail "leg 5: real fix REJECTED — the gate can never pass"; }

# Each mutant PLANTS its own defect, so this control holds before AND after the
# fix — a control that only works while the defect is present cannot pass once
# the defect is gone (the CRUX-30 class), which is what the first draft did.
esc='println!("cargo:rerun-if-changed=../assets/demo/");'
d=$(plant defect); printf '%s\n' "$esc" >>"$d/repo/build.rs"
  reject "DEFECT the historical build.rs:21 line, re-planted" "$d"
d=$(plant g1); printf '%s\n' "$esc" >>"$d/repo/build.rs"; mkdir -p "$d/assets/demo"
  reject "G1 out-of-tree sibling materialised next to the checkout" "$d"
d=$(plant g2); mkdir -p "$d/assets/demo"
  printf '%s\n' 'println!("cargo:rerun-if-changed=../{}", "assets/demo/");' >>"$d/repo/build.rs"
  reject "G2 escape hidden behind an interpolation" "$d"
d=$(plant g2b); mkdir -p "$d/assets/demo"
  printf '%s\n' 'println!("cargo:rerun-if-changed={}", "../assets/demo/");' >>"$d/repo/build.rs"
  reject "G2b whole path behind {}" "$d"
d=$(plant g3); mkdir -p "$d/pmat-book"
  printf '%s\n' 'println!("cargo:rerun-if-changed=../pmat-book/");' >>"$d/repo/build.rs"
  reject "G3 repointed at a different EXISTING sibling" "$d"
d=$(plant g4)
  awk '/rerun-if-changed=\.git\/HEAD|rerun-if-changed=\.git\/index/{next}
       /rerun-if-changed=templates\//{print;print;print;next}{print}' \
      "$d/repo/build.rs" >"$d/b" && mv "$d/b" "$d/repo/build.rs"
  reject "G4 provenance watches deleted, count padded with duplicates" "$d"
d=$(plant g5)
  sed -i 's|cargo:rerun-if-changed=[^"]*|cargo:rerun-if-changed=templates/|g' "$d/repo/build.rs"
  reject "G5 every directive flattened to one present path" "$d"
d=$(plant fix); printf '%s\n' "$esc" >>"$d/repo/build.rs"
  grep -vF 'rerun-if-changed=../assets/demo/' "$d/repo/build.rs" >"$d/b" && mv "$d/b" "$d/repo/build.rs"
  accept "the real fix (the planted line deleted, nothing else)" "$d"
echo "leg 5 CONTROL ok: 7 mutants rejected, the real fix accepted"

# ---- leg 6 — the verdict on the tree under test. RED today.
found=$(audit "$root") || fail "leg 6: auditor crashed"
[ -z "$found" ] || fail "leg 6: $(printf '%s' "$found" | tr '\n' ' ')"

# ---- leg 7 — cargo's own recorded fingerprint must agree. Precondition: one
# completed release build. Fails loudly (never skips) when unmet.
td=$(cargo metadata --no-deps --format-version 1 | jq -r '.target_directory') \
   || fail "leg 7: cargo metadata failed"
fp=$(ls -t "$td"/release/.fingerprint/pmat-*/run-build-script-build-script-build.json 2>/dev/null | head -1) || true
[ -n "${fp:-}" ] || fail "leg 7: no build-script fingerprint under $td — run 'cargo build --release' once to arm this leg"
jq -e '[.local[].RerunIfChanged.paths // empty] | flatten | length >= 9' "$fp" >/dev/null \
   || fail "leg 7: fingerprint $fp records <9 tracked paths — wrong artefact or a rotted schema"
esc=$(jq -r '[.local[].RerunIfChanged.paths // empty] | flatten | .[]' "$fp" \
      | while read -r p; do
          case "$p" in
            /*|*..*) echo "TRACKS-ESCAPING:$p" ;;
            *) [ -e "$root/$p" ] || echo "TRACKS-MISSING:$p" ;;
          esac
        done)
[ -z "$esc" ] || fail "leg 7: cargo is tracking $(printf '%s' "$esc" | tr '\n' ' ')"

echo "PASS: build.rs declares 8 literal watches, all inside CARGO_MANIFEST_DIR and present;"
echo "      1 interpolated directive (the schema walk); cargo tracks nothing outside the tree."
