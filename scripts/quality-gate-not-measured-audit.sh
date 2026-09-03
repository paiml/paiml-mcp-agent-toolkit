#!/usr/bin/env bash
# CRUX-02 — `quality-gate` must not render an unmeasured dimension as clean (spec section 8.2, epic 1153).
# Leg 1: dead code on an uncompilable crate is NOT MEASURED (reason names the compile failure);
#        control A: the same crate compiling has no dead_code disclosure;
#        control B: a directory with no Cargo.toml is NOT APPLICABLE, never not_measured.
# Leg 2: a fabricated coverage cache is REJECTED and disclosed as a coverage finding naming the guard;
#        control: a cache from HEAD covering the tree is accepted (no rejection).
# Leg 3: the whole-file count is `identical_files`; `duplicate_violations` is gone; the block-level half is disclosed.
# Leg 4: --help says the gate runs cargo check.
# PMAT=<binary> overrides the pmat used. Reads only schema-declared fields.
set -euo pipefail
fail(){ echo "FAIL: $*"; exit 1; }
PMAT=${PMAT:-$(command -v pmat)}
T=$(mktemp -d)
cleanup(){ case "${T:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$T" ] && rm -rf -- "$T";; esac; }; trap cleanup EXIT

mkcrate(){ # $1 dir, $2 lib body
  mkdir -p "$1/src"
  printf '[package]\nname = "fx"\nversion = "0.0.0"\nedition = "2021"\n\n[lib]\npath = "src/lib.rs"\n' > "$1/Cargo.toml"
  printf '%s\n' "$2" > "$1/src/lib.rs"
}
gate(){ # $1 dir, rest: extra args → JSON on stdout (exit code ignored: the verdict is read from the payload)
  local d=$1; shift
  "$PMAT" quality-gate --format json -p "$d" "$@" 2>/dev/null || true
}

# ---- Leg 1: could not compile → not_measured, reason names the compile failure
mkcrate "$T/broken" 'pub fn broken( {'
gate "$T/broken" --checks dead-code | jq -e '.results.not_measured[] | select(.check=="dead_code" and (.reason|test("could not compile")))' >/dev/null \
  || fail "leg 1: an uncompilable crate did not report dead_code as not_measured (could not compile)"
# control A: the same crate, compiling → no dead_code disclosure in either list
mkcrate "$T/healthy" 'pub fn fine() {}'
gate "$T/healthy" --checks dead-code | jq -e '([.results.not_measured[]?, .results.not_applicable[]?] | map(select(.check=="dead_code")) | length) == 0' >/dev/null \
  || fail "leg 1 control A: a compiling crate reported a dead_code disclosure"
# control B: no Cargo.toml → not_applicable, never not_measured
mkdir -p "$T/nocrate"; printf 'print(1)\n' > "$T/nocrate/main.py"
gate "$T/nocrate" --checks dead-code | jq -e '(.results.not_applicable[] | select(.check=="dead_code" and (.reason|test("no Cargo.toml")))) and (([.results.not_measured[]? | select(.check=="dead_code")] | length) == 0)' >/dev/null \
  || fail "leg 1 control B: a directory without Cargo.toml was not reported as not_applicable (or was reported not_measured)"

# ---- Leg 2: a fabricated cache is rejected and the rejection names its guard
mkdir -p "$T/healthy/.pmat"
printf '{"git_hash":"deadbeefdeadbeefdeadbeefdeadbeefdeadbeef","timestamp":"2019-01-01T00:00:00Z","files":{"src/deleted.rs":{"1":5}}}' > "$T/healthy/.pmat/coverage-cache.json"
r=$(gate "$T/healthy" --checks coverage)
printf '%s' "$r" | jq -e '.results.coverage_violations > 0' >/dev/null || fail "leg 2: a fabricated coverage cache was accepted (coverage_violations == 0)"
printf '%s' "$r" | jq -e '[.violations[]? | select(.check_type=="coverage") | .message] | any(test("REJECTED") and test("git_hash:|mtime:|breadth:"))' >/dev/null \
  || fail "leg 2: the rejection does not name the guard that tripped"
# control: a cache from HEAD, covering the tree, fresh → accepted (no REJECTED finding)
git -C "$T/healthy" init -q && git -C "$T/healthy" -c user.email=t@t -c user.name=t add . >/dev/null && git -C "$T/healthy" -c user.email=t@t -c user.name=t commit -q -m fx
HEAD_SHA=$(git -C "$T/healthy" rev-parse HEAD)
printf '{"git_hash":"%s","files":{"src/lib.rs":{"1":1}}}' "$HEAD_SHA" > "$T/healthy/.pmat/coverage-cache.json"
gate "$T/healthy" --checks coverage | jq -e '[.violations[]? | select(.check_type=="coverage") | .message] | any(test("REJECTED")) | not' >/dev/null \
  || fail "leg 2 control: a fresh report from HEAD covering the tree was rejected"

# ---- Leg 3: the honest name, the old key gone, the block-level half disclosed
r=$(gate "$T/healthy" --checks duplicates)
printf '%s' "$r" | jq -e '(.results | has("duplicate_violations") | not) and (.results.identical_files == 0) and (.results.not_measured[]? | select(.check=="duplicates" and (.reason|test("block-level"))))' >/dev/null \
  || fail "leg 3: duplicate_violations still present, identical_files != 0, or the block-level disclosure is missing"

# ---- Leg 4: the help says the gate runs cargo check
"$PMAT" quality-gate --help 2>/dev/null | grep -qiE 'cargo check' || fail "leg 4: --help does not say the gate runs cargo check"
echo PASS
