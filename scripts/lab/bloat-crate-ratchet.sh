#!/usr/bin/env bash
# scripts/lab/bloat-crate-ratchet.sh -- #1265 (CRUX-05). The binary's .text grew ~3.4 MiB with nothing watching a
# per-crate figure. This ratchets each crate's .text size from `cargo bloat --release --crates --message-format json`
# against a committed baseline (.pmat/bloat-baseline.json: {"<crate>": bytes}). LAB (C324): run at night or on demand,
# never in a PR, merge-queue or release job. Promotion to a gate needs the operator's yes, 3 green nights, one gate retired.
#   OVER          a crate grew by more than PCT percent AND more than FLOOR bytes, or a new crate is over FLOOR
#   NOT MEASURED  no baseline, no report, a report with no crates, or a crate list that is empty
#   GREEN         every crate within PCT percent or FLOOR bytes of its baseline
# Usage: bloat-crate-ratchet.sh --classify REPORT.json BASELINE.json | --measure BASELINE.json | --self-test
# Env: BLOAT_PCT (default 5), BLOAT_FLOOR (default 65536 bytes). Exit: 0 GREEN, 1 OVER, 2 NOT MEASURED/usage.
set -uo pipefail
PCT=${BLOAT_PCT:-5} FLOOR=${BLOAT_FLOOR:-65536}
classify() { # report baseline
  local rep=$1 base=$2 out
  [ -s "$rep" ] && [ -s "$base" ] || { echo "NOT MEASURED: report or baseline missing/empty"; return 2; }
  command -v jq >/dev/null || { echo "NOT MEASURED: jq not installed"; return 2; }
  out=$(jq -r --slurpfile b "$base" --argjson pct "$PCT" --argjson floor "$FLOOR" '
    ((.crates // []) | map({(.name): .size}) | add // {}) as $now
    | if ($now | length) == 0 then "NM"
      else ($now | to_entries[]
        | .key as $k | .value as $v | ($b[0][$k] // 0) as $was
        | select(($v - $was) > $floor and ($v - $was) * 100 > $was * $pct)
        | "OVER \($k): \($was) -> \($v) bytes (+\($v - $was))") end' "$rep") || { echo "NOT MEASURED: unreadable report or baseline"; return 2; }
  case "$out" in
    NM*) echo "NOT MEASURED: report lists no crates"; return 2 ;;
    "") echo "GREEN: every crate within ${PCT}% or ${FLOOR} bytes of baseline"; return 0 ;;
    *) printf '%s\n' "$out"; return 1 ;;
  esac
}
self_test() {
  local T fails=0 rc; T=$(mktemp -d); trap 'rm -rf "${T:?}"' RETURN
  printf '{"pmat":2000000,"pmcp":700000,"std":300000}\n' > "$T/base.json"
  want() { rc=0; classify "$2" "$T/base.json" >/dev/null || rc=$?; if [ "$rc" = "$1" ]; then echo "ok   $3"; else echo "FAIL $3 (want $1 got $rc)"; fails=$((fails+1)); fi; }
  printf '{"crates":[{"name":"pmat","size":2000000},{"name":"pmcp","size":700000},{"name":"std","size":300000}]}' > "$T/same.json"; want 0 "$T/same.json" "unchanged: GREEN"
  printf '{"crates":[{"name":"pmat","size":3900000},{"name":"pmcp","size":700000},{"name":"std","size":300000}]}' > "$T/over.json"; want 1 "$T/over.json" "pmat +1.9 MiB: OVER"
  printf '{"crates":[{"name":"pmat","size":2000000},{"name":"clap","size":85000},{"name":"std","size":300000}]}' > "$T/new.json"; want 1 "$T/new.json" "new crate over the floor: OVER"
  printf '{"crates":[{"name":"pmat","size":2020000},{"name":"pmcp","size":700000},{"name":"std","size":300000}]}' > "$T/small.json"; want 0 "$T/small.json" "+1% under PCT: GREEN"
  printf '{"crates":[{"name":"tiny","size":1000},{"name":"pmat","size":2000000}]}' > "$T/tinyg.json"; want 0 "$T/tinyg.json" "growth under the byte floor: GREEN"
  printf '{"crates":[]}' > "$T/empty.json"; want 2 "$T/empty.json" "no crates listed: NOT MEASURED, not green"
  : > "$T/zero.json"; want 2 "$T/zero.json" "empty report file: NOT MEASURED"
  printf 'not json' > "$T/bad.json"; want 2 "$T/bad.json" "unreadable report: NOT MEASURED"
  rc=0; classify "$T/same.json" "$T/nope.json" >/dev/null || rc=$?; [ "$rc" = 2 ] && echo "ok   no baseline: NOT MEASURED" || { echo "FAIL no baseline (got $rc)"; fails=$((fails+1)); }
  echo "bloat-crate-ratchet self-test: $fails failure(s)"; [ "$fails" = 0 ]
}
case "${1:-}" in
  --self-test) self_test ;;
  --classify) [ $# = 3 ] || { echo "usage: --classify REPORT BASELINE" >&2; exit 2; }; classify "$2" "$3" ;;
  --measure) [ $# = 2 ] || { echo "usage: --measure BASELINE" >&2; exit 2; }
    r=$(mktemp) && trap 'rm -f "${r:?}"' EXIT
    cargo bloat --release --crates -n 0 --message-format json > "$r" 2>/dev/null || { echo "NOT MEASURED: cargo bloat failed"; exit 2; }
    classify "$r" "$2" ;;
  *) echo "usage: $0 --classify REPORT BASELINE | --measure BASELINE | --self-test" >&2; exit 2 ;;
esac
