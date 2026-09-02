#!/usr/bin/env bash
# CRUX-01 — pmat verify composite verdict + strict SATD audit (spec section 8.1, issue 1146).
# Hermetic: every leg runs in a mktemp fixture. EVIDENCE legs are red before the fix; CONTROL
# legs are green by design and reject the lazy fixes. The REPO leg is one-shot PR evidence.
# PMAT=<binary> REPO=<checkout> override the defaults.

set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
P=${PMAT:-pmat}; R=${REPO:-.}; W=$(mktemp -d)
cleanup(){ case "${W:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$W" ] && rm -rf -- "$W";; esac; }; trap cleanup EXIT
crate(){ mkdir -p "$W/$1/src"
  printf '[package]\nname = "%s"\nversion = "0.1.0"\nedition = "2021"\n' "$1" >"$W/$1/Cargo.toml"
  cat >"$W/$1/src/lib.rs"
  ( cd "$W/$1" && git init -q . && git add -A \
    && git -c core.hooksPath=/dev/null -c user.email=a@b -c user.name=c commit -qm i ); }
crate clean  <<'EOF'
pub fn f(x: u8) -> u8 {
    x + 1
}
EOF
crate red    <<'EOF'
pub fn f(x: u8) -> u8 {
    // TODO: finish this
    x + 1
}
EOF
crate sep    <<'EOF'
pub fn a(x: u8) -> u8 {
    // TODO(CB-128): widen the separator
    x
}
pub fn b(x: u8) -> u8 {
    // TODO[CB-128]: widen the bracket too
    x
}
pub fn c(x: u8) -> u8 {
    // todo: finish this
    x
}
pub fn d(x: u8) -> u8 {
    // TODO:
    x
}
EOF
crate bug    <<'EOF'
//! ```
//!         todo!("Create proper QueryResult")
//! ```
// Bug: Previously used walkdir directly, bypassing ignore file support
pub fn f(x: u8) -> u8 {
    x
}
EOF
crate nodebt <<'EOF'
pub fn f(x: u8) -> u8 {
    x
}
EOF
crate l2     <<'EOF'
pub fn f() -> u8 {
    let x: u8 = 0; // TODO(CB-9): finish this
    x
}
EOF
mkdir -p "$W/empty"
V(){ ( cd "$W/$1" && shift && $P verify --format json "$@" ); }
S(){ $P analyze satd --path "$W/$1" "${@:2}" --format json 2>/dev/null; }

# leg 1 — EVIDENCE. Declining a stage must withdraw the verdict, not fail the build.
out=$(V clean --skip clippy,tests) && rc=0 || rc=$?
jq -e '.ok == null and (.not_measured|type) == "array"
       and .not_measured == ["complexity"] and .stages_measured == 2' >/dev/null <<<"$out" \
  || fail "leg 1: verify still asserts over a stage it declined"
[ "$rc" -eq 0 ] || fail "leg 1: a declined stage must not start failing the build"

# leg 1-RED — CONTROL (green today). A measured failure must survive a co-declining stage.
out=$(V red --skip clippy,tests) && rc=0 || rc=$?
jq -e '.ok == false and ([.stages[]|select(.name=="satd").ok] == [false])' >/dev/null <<<"$out" \
  || fail "leg 1-RED: a measured failure went green because another stage declined"
[ "$rc" -eq 1 ] || fail "leg 1-RED: ok:false must still exit 1"

# leg 1-EMPTY — EVIDENCE for the list type, CONTROL for the v3.30.0 verdict.
out=$(V empty --skip clippy,tests) && rc=0 || rc=$?
jq -e '.ok == false and .stages_measured == 0 and (.not_measured|type) == "array"
       and (.not_measured|sort) == ["complexity","format","satd"]' >/dev/null <<<"$out" \
  || fail "leg 1-EMPTY: all-declined contract regressed, or not_measured is not derived"
[ "$rc" -eq 1 ] || fail "leg 1-EMPTY: nothing measured is not a pass"

# leg 1-SKIP — CONTROL (green today). A skipped stage is not an unmeasured one.
V clean --skip clippy,tests,complexity \
  | jq -e '(.ok|type) == "boolean" and (has("not_measured")|not) and .stages_measured == 2' >/dev/null \
  || fail "leg 1-SKIP: --skip reported as unmeasured"

# leg 2 — EVIDENCE. The separator blind spot, minimal reproduction. Today 0.
S l2 --strict | jq -e '.total_violations == 1' >/dev/null \
  || fail "leg 2: strict still misses the paren separator"

# A3-M — EVIDENCE. Separator widened; case and the work-item requirement are NOT.
S sep --strict | jq -e '[.violations[].line]|sort == [2,6]' >/dev/null \
  || fail "A3-M: strict must match TODO(x): and TODO[x]: and nothing else in this file"
S sep         | jq -e '[.violations[].line]|sort == [2,6,10]' >/dev/null \
  || fail "A3-M: default mode moved"

# A3-D — EVIDENCE. The decided case clause: capitalised `Bug:` is debt, lower-case `todo!(` is not.
S bug --strict | jq -e '[.violations[].line] == [4]' >/dev/null \
  || fail "A3-D: strict must see Bug: and only Bug:"
S bug          | jq -e '[.violations[].line]|sort == [2,4]' >/dev/null \
  || fail "A3-D: default mode moved"

# A3-D2 — EVIDENCE, and the one invariant here fit to become a permanent gate:
# every severity=error SATD finding quality-gate blocks on is visible to verify's own stage.
blk=$($P quality-gate --checks satd --format json -p "$W/bug" 2>/dev/null \
      | jq -c '[.violations[]|select(.severity=="error")|.line]|sort' || true)
jq -e 'length >= 1' >/dev/null <<<"$blk" || fail "A3-D2: fixture no longer reproduces the divergence"
st=$(S bug --strict | jq -c '[.violations[].line]|sort')
jq -ne --argjson b "$blk" --argjson s "$st" '$b - $s == []' >/dev/null \
  || fail "A3-D2: quality-gate blocks on SATD verify's strict stage cannot see"

# A4 — CONTROL (green today). No marker, no finding, in either mode.
S nodebt          | jq -e '.total_violations == 0' >/dev/null || fail "A4: default reported debt"
S nodebt --strict | jq -e '.total_violations == 0' >/dev/null || fail "A4: strict reported debt"

# A1 — CONTROL (green today). Blocks "always null": one real Rust edit and complexity RUNS.
printf 'pub fn g(x: u8) -> u8 {\n    x\n}\n' >>"$W/clean/src/lib.rs"
V clean --skip clippy,tests \
  | jq -e '(.ok|type) == "boolean" and (has("not_measured")|not) and .stages_measured == 3' >/dev/null \
  || fail "A1: no verdict on a tree verify fully measured"
git -C "$W/clean" checkout -- src/lib.rs

# A2 — CONTROL (green today). Blocks deleting a stage.
V clean --skip clippy,tests | jq -e '[.stages[].name] == ["format","complexity","satd","clippy","tests"]' \
  >/dev/null || fail "A2: a stage disappeared from the report"

# REPO — ONE-SHOT PR EVIDENCE, never a committed gate: the site that motivated the fix.
# It PASSED on the fix commit (70771f654) and then the line was stopped on that very
# finding: the marker was real debt (TDG's dead_code weight is a constant 0.0) and was
# converted to a tracked ticket (PMAT-639) in 210810163. Once the marker is gone the
# leg has nothing to see, by design; the permanent invariant is A3-D2 above.
if grep -q 'TODO(CB-128)' "$R/src/services/tdg_calculator_core.rs" 2>/dev/null; then
  $P analyze satd --path "$R" --strict --format json 2>/dev/null \
    | jq -e '[.violations[]|select(.file|endswith("services/tdg_calculator_core.rs"))]|length == 1' \
    >/dev/null || fail "REPO: verify's own stage still cannot see tdg_calculator_core.rs:110"
else
  grep -q 'PMAT-639' "$R/src/services/tdg_calculator_core.rs" 2>/dev/null \
    || fail "REPO: the marker is gone but the site does not cite the ticket that replaced it"
  echo "INFO: REPO leg — marker resolved into PMAT-639; nothing left for strict to see"
fi
echo "PASS"
