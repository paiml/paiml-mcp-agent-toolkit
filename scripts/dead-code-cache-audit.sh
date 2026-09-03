#!/usr/bin/env bash
# CRUX-04 — the dead-code cache must be keyed on the WORKING tree, and a replay must say so
# (spec section 8.4, epic 1153, precedent #748).
# A shim on PATH logs every cargo argv and execs the real cargo, so exec count and
# `cache.hit` are asserted TOGETHER: hit=true pairs with +0 execs, hit=false with exactly +1.
# States: A cold → B rerun → C uncommitted dead fn → D rerun → E revert; F quality-gate sees C;
# G old-schema entry is a miss. Control: A/C/E with the cache deleted answer 0/1/0.
# PMAT=<binary> overrides the pmat used. Reads only schema-declared fields.
set -euo pipefail
fail(){ echo "FAIL: $*"; exit 1; }
PMAT=${PMAT:-$(command -v pmat)}
# The cache lives under the fixture crate; wipe it only when $C is the fixture we made.
wipe_cache(){
  if [ -z "${C:-}" ] || [ ! -f "$C/Cargo.toml" ] || [ ! -d "$C/src" ]; then
    fail "wipe_cache: $C is not the fixture crate"
  fi
  rm -f -- "$C"/.pmat/dead-code-cache-*.json
}
T=$(mktemp -d)
cleanup(){ case "${T:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$T" ] && rm -rf -- "$T";; esac; }; trap cleanup EXIT

# ---- the shim: real cargo resolved with a clean env so a shell function cannot self-recurse
REAL_CARGO=$(env -i PATH="$PATH" sh -c 'command -v cargo') || fail "no cargo on PATH"
mkdir -p "$T/bin"; LOG="$T/cargo-argv.log"; : > "$LOG"
printf '#!/bin/sh\nprintf "%%s\\n" "$*" >> "%s"\nexec "%s" "$@"\n' "$LOG" "$REAL_CARGO" > "$T/bin/cargo"; chmod +x "$T/bin/cargo"
export PATH="$T/bin:$PATH"
checks(){ grep -c '^check' "$LOG" || true; }

# ---- fixture crate, committed
C="$T/crate"; mkdir -p "$C/src"
printf '[package]\nname = "fx"\nversion = "0.0.0"\nedition = "2021"\n' > "$C/Cargo.toml"
printf 'pub fn alive() {}\n' > "$C/src/lib.rs"
( cd "$C" && git init -q && git -c core.hooksPath=/dev/null -c user.email=t@t -c user.name=t add . && git -c core.hooksPath=/dev/null -c user.email=t@t -c user.name=t commit -q -m one )
( cd "$C" && "$REAL_CARGO" generate-lockfile -q 2>/dev/null || true )
wipe_cache

run(){ # $1 label; prints JSON of `analyze dead-code --format json`
  "$PMAT" analyze dead-code -p "$C" --format json 2>/dev/null
}
expect(){ # $1 label $2 expected-delta-execs $3 expected-hit $4 expected-reason $5 expected-dead-functions
  local before=$1; shift
  local label=$1 dexecs=$2 hit=$3 reason=$4 dead=$5
  local r; r=$(run "$label")
  local after; after=$(checks)
  local got=$((after - before))
  [ "$got" = "$dexecs" ] || fail "$label: expected +$dexecs cargo check exec(s), got +$got"
  printf '%s' "$r" | jq -e --argjson h "$hit" '.cache.hit == $h' >/dev/null || fail "$label: cache.hit != $hit ($(printf '%s' "$r" | jq -c .cache))"
  printf '%s' "$r" | jq -e --arg x "$reason" '.compiler_scan.reason == $x' >/dev/null || fail "$label: compiler_scan.reason != $reason ($(printf '%s' "$r" | jq -c .compiler_scan.reason))"
  printf '%s' "$r" | jq -e --argjson d "$dead" '.summary.dead_functions == $d' >/dev/null || fail "$label: dead_functions != $dead ($(printf '%s' "$r" | jq -c .summary.dead_functions))"
  printf '%s' "$r"
}

# A cold: +1 exec, miss, ran, 0 dead
b=$(checks); expect "$b" A 1 false compiler-lint-ran 0 >/dev/null
# B rerun, no edit: +0, hit, cached, 0
b=$(checks); expect "$b" B 0 true compiler-lint-cached 0 >/dev/null
# C append a dead fn, NOT committed: +1, miss, ran, 1 named
printf 'fn dead_uncommitted() {}\n' >> "$C/src/lib.rs"
b=$(checks); r=$(expect "$b" C 1 false compiler-lint-ran 1)
printf '%s' "$r" | jq -e '[.files[].items[].name] | index("dead_uncommitted") != null' >/dev/null || fail "C: the uncommitted dead fn is not named"
# D rerun, no further edit: +0, hit, cached, 1 (the cheapest fix — bypass the cache whenever the tree is dirty — fails here)
b=$(checks); expect "$b" D 0 true compiler-lint-cached 1 >/dev/null
# E revert: +1, miss, ran, 0
( cd "$C" && git checkout -q -- src/lib.rs )
b=$(checks); expect "$b" E 1 false compiler-lint-ran 0 >/dev/null

# F: the gate on state C sees the finding (same analyzer, use_cache on)
printf 'fn dead_uncommitted() {}\n' >> "$C/src/lib.rs"
# The gate FAILS on the finding (exit 1) — that is the point — so its exit code is not the verdict; its payload is.
g=$("$PMAT" quality-gate --checks dead-code --format json -p "$C" 2>/dev/null || true)
[ -n "$g" ] || fail "F: gate produced no output"
# The gate's threshold is a percentage; on a 2-fn crate one dead fn is 50 % — well above any default — so it must count.
printf '%s' "$g" | jq -e '.results.dead_code_violations >= 1' >/dev/null || fail "F: quality-gate --checks dead-code did not see the uncommitted dead fn ($(printf '%s' "$g" | jq -c '.results.dead_code_violations'))"
( cd "$C" && git checkout -q -- src/lib.rs )

# G: an old-schema entry keyed on THIS tree must be a MISS
f=$(ls "$C"/.pmat/dead-code-cache-*.json | head -1); [ -n "$f" ] || fail "G setup: no cache file"
jq '.report_schema = (.report_schema - 1)' "$f" > "$f.tmp" && mv "$f.tmp" "$f"
b=$(checks); expect "$b" G 1 false compiler-lint-ran 0 >/dev/null

# Control: with the cache deleted, A/C/E answer 0/1/0 — a fix that changed the ANSWERS, not the key, is caught here
wipe_cache; b=$(checks); expect "$b" ctlA 1 false compiler-lint-ran 0 >/dev/null
printf 'fn dead_uncommitted() {}\n' >> "$C/src/lib.rs"; wipe_cache; b=$(checks); expect "$b" ctlC 1 false compiler-lint-ran 1 >/dev/null
( cd "$C" && git checkout -q -- src/lib.rs ); wipe_cache; b=$(checks); expect "$b" ctlE 1 false compiler-lint-ran 0 >/dev/null

# --no-cache: the escape hatch exists and bypasses a warm entry
b=$(checks); run A >/dev/null; b2=$(checks); [ "$b2" = "$b" ] || fail "no-cache setup: the warm entry was not served"
"$PMAT" analyze dead-code -p "$C" --format json --no-cache 2>/dev/null | jq -e '.cache.hit == false and .compiler_scan.reason == "compiler-lint-ran"' >/dev/null || fail "--no-cache did not force a fresh compiler pass"
[ "$(checks)" = "$((b2 + 1))" ] || fail "--no-cache did not exec cargo check exactly once"
echo PASS
