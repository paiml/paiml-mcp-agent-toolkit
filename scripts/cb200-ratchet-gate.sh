#!/usr/bin/env bash
# cb200-ratchet-gate.sh — CB-200's `[tdg] baseline`, measured where a merge is decided (PMAT-636).
#
# `[tdg] baseline` in .pmat-gates.toml is a ratchet on how many definitions grade below A. Until
# PMAT-636 nothing that gates a merge measured it. The --lib test that re-derives it,
# `the_committed_baseline_is_the_measured_count`, needs `.pmat/context.db`; a CI checkout has none,
# so the test took its no-index branch and PASSED on every pull request while master drifted from
# 1688 to 1741. This script is what the `tdg-ratchet` job in ci.yml runs instead. It asks CB-200
# itself — which builds pmat's own index outside the checkout when the tree has none (#1008) — and
# passes on exactly one outcome:
#
#   PASS          CB-200 measured the tree and the count EQUALS the recorded baseline
#
# Every other outcome exits 1, and says which:
#
#   OVER          more definitions below A than the baseline: new debt
#   BEATEN        fewer than the baseline, and not banked: slack a regression can hide in
#   STALE         counted from an index older than the sources: a verdict on an OLDER tree
#   NOT MEASURED  CB-200 absent from the report, skipped, not measured, or its verdict does not
#                 lead with a count — a ratchet that did not run has not held
#   DISAGREE      the baseline CB-200 compared against is not the one .pmat-gates.toml records
#
# The verdict is judged, not comply's exit code: comply exits 0 on the Warn a held baseline
# reports, and exit codes cannot tell OVER from NOT MEASURED.
#
# Usage: scripts/cb200-ratchet-gate.sh PMAT [--root DIR]   measure DIR (default: .)
#        scripts/cb200-ratchet-gate.sh PMAT --control      reach every verdict above, planted, and
#                                                          require each one
#        scripts/cb200-ratchet-gate.sh --classify REPORT --gates FILE
#                                                          judge a saved `comply check --format json`
#   PMAT is the pmat built from the tree being judged, and it comes first: scripts/gate.sh's
#   control (arm 12) stubs the scripts a pmat leg calls with `exec "$1"`.
# Exit: 0 PASS (--control: every arm held) · 1 any other verdict (--control: an arm broke) · 2 usage
set -uo pipefail

MODE=measure PMAT="" ROOT=. REPORT="" GATES=""
case "${1:-}" in
  ""|-*) ;;
  *) PMAT=$1; shift ;;
esac
while [ $# -gt 0 ]; do
  case "$1" in
    --root) ROOT="${2:-}"; shift 2 ;;
    --classify) MODE=classify; REPORT="${2:-}"; shift 2 ;;
    --gates) GATES="${2:-}"; shift 2 ;;
    --control) MODE=control; shift ;;
    -h|--help) sed -n '2,/^set -uo pipefail$/p' "$0" | sed '$d'; exit 0 ;;
    *) echo "cb200-ratchet-gate.sh: unknown argument '$1'" >&2; exit 2 ;;
  esac
done
command -v jq >/dev/null || { echo "cb200-ratchet-gate.sh: jq is required" >&2; exit 2; }

# `[tdg] baseline` from a .pmat-gates.toml, read independently of CB-200.
recorded_baseline() {
  awk '/^[[:space:]]*\[/ { in_tdg = ($0 ~ /^[[:space:]]*\[tdg\][[:space:]]*$/) }
       in_tdg && /^[[:space:]]*baseline[[:space:]]*=/ {
         sub(/^[[:space:]]*baseline[[:space:]]*=[[:space:]]*/, ""); sub(/[[:space:]]*(#.*)?$/, ""); print; exit
       }' "$1" 2>/dev/null
}

# classify <report.json> <gates.toml>: print one "<VERDICT>: <why>" line; return 0 only on PASS.
classify() {
  local report=$1 gates=$2 numeric="^[0-9]+\$" against="recorded baseline of ([0-9]+)"
  local found status headline count recorded reported
  found=$(jq -r '[.checks[]? | select((.name // "") | startswith("CB-200"))] | length' "$report" 2>/dev/null)
  if [ "$found" != 1 ]; then
    echo "NOT MEASURED: the report holds ${found:-no readable} CB-200 verdict(s), not exactly one"
    return 1
  fi
  status=$(jq -r '.checks[] | select(.name | startswith("CB-200")) | .status' "$report")
  headline=$(jq -r '.checks[] | select(.name | startswith("CB-200")) | .message | split("\n")[0]' "$report")
  if [ "$status" = Skip ] || [[ "$headline" == "Not measured"* ]]; then
    echo "NOT MEASURED: CB-200 ($status) said: $headline"
    return 1
  fi
  if [[ "$headline" == "NOT A VERDICT"* || "$headline" == *"index is stale"* ]]; then
    echo "STALE: CB-200 counted an index older than the sources, which describes an older tree. Rebuild it with \`pmat query \"x\" --rebuild-index\`, or remove .pmat/context.db so CB-200 builds its own. CB-200 said: $headline"
    return 1
  fi
  if ! [[ "$headline" =~ ^([0-9]+)\ (definition\(s\)|definitions|function\(s\))\ below\ minimum\ grade ]]; then
    echo "NOT MEASURED: CB-200's verdict ($status) does not lead with a count: $headline"
    return 1
  fi
  count=${BASH_REMATCH[1]}
  recorded=$(recorded_baseline "$gates")
  if ! [[ "$recorded" =~ $numeric ]]; then
    echo "NOT MEASURED: $gates records no numeric [tdg] baseline ('$recorded'), so there is no ratchet to hold; CB-200 counted $count"
    return 1
  fi
  if ! [[ "$headline" =~ $against ]]; then
    echo "DISAGREE: $gates records [tdg] baseline = $recorded, and CB-200 compared against no baseline at all: $headline"
    return 1
  fi
  reported=${BASH_REMATCH[1]}
  if [ "$reported" != "$recorded" ]; then
    echo "DISAGREE: $gates records [tdg] baseline = $recorded, CB-200 compared against $reported"
    return 1
  fi
  if [ "$count" -gt "$recorded" ]; then
    echo "OVER: $count definitions below grade A, $((count - recorded)) over the recorded baseline of $recorded. Bring them to grade A, or revert what added them. Raising the baseline is not the fix: it may only go down."
    return 1
  fi
  if [ "$count" -lt "$recorded" ]; then
    echo "BEATEN: $count definitions below grade A, $((recorded - count)) under the recorded baseline of $recorded, and not banked. Set [tdg] baseline = $count in .pmat-gates.toml: slack is headroom a regression hides in."
    return 1
  fi
  echo "PASS: $count definitions below grade A, exactly the recorded baseline of $recorded (CB-200 status $status)"
}

# measure <root>: run CB-200 over <root> and judge its verdict.
measure() {
  local root=$1 out rc verdict
  out=$(mktemp -d)
  # shellcheck disable=SC2064  # expand now: $out is local and gone when the trap fires
  trap "rm -rf '$out'" RETURN
  "$PMAT" comply check --path "$root" --checks CB-200 --format json >"$out/report.json" 2>"$out/stderr"
  rc=$?
  if ! jq -e '.checks' "$out/report.json" >/dev/null 2>&1; then
    echo "NOT MEASURED: \`pmat comply check --checks CB-200\` exited $rc and wrote no JSON report"
    tail -20 "$out/stderr"
    return 1
  fi
  classify "$out/report.json" "$root/.pmat-gates.toml"
  verdict=$?
  if [ "$verdict" = 0 ] && [ "$rc" != 0 ]; then
    echo "NOT MEASURED: the verdict reads PASS but \`pmat comply check\` exited $rc"
    return 1
  fi
  return "$verdict"
}

require_pmat() {
  [ -n "$PMAT" ] || { echo "cb200-ratchet-gate.sh: PMAT, the pmat built from the tree it judges, must be the first argument" >&2; exit 2; }
  [ -x "$PMAT" ] || { echo "cb200-ratchet-gate.sh: PMAT '$PMAT' is not an executable" >&2; exit 2; }
}

# ── control ─────────────────────────────────────────────────────────────────────────────────
# Every verdict this gate can return is planted and required, so a green run is evidence that a
# red one was possible. Arms 1-8 judge fixture reports; arms 9-10 run the real pmat over a
# fixture project holding one definition planted below grade A.
arm() { # arm <n> <name> <expected exit> <expected verdict word> <command...>
  local n=$1 name=$2 want_rc=$3 want=$4 got rc
  shift 4
  got=$("$@" 2>&1)
  rc=$?
  if [ "$rc" = "$want_rc" ] && [[ "$got" == "$want:"* ]]; then
    echo "arm $n $name: GREEN ($want, exit $rc)"
  else
    echo "arm $n $name: RED — wanted $want exit $want_rc, got exit $rc: $(printf '%s' "$got" | tail -3)"
    ARMS_RED=$((ARMS_RED + 1))
  fi
}

report() { # report <file> <status> <message>
  jq -n --arg s "$2" --arg m "$3" '{checks: [{name: "CB-200: TDG Grade Gate", status: $s, message: $m}]}' >"$1"
}

planted_project() { # planted_project <dir> <baseline>
  mkdir -p "$1/src"
  printf '[tdg]\nbaseline = %s\n' "$2" >"$1/.pmat-gates.toml"
  cat >"$1/src/lib.rs" <<'RUST'
/// Planted by scripts/cb200-ratchet-gate.sh --control: one definition below grade A.
pub fn planted_below_a(a: i32, b: i32, c: i32, d: i32, mode: &str) -> i32 {
    let mut n = 0;
    for i in 0..a {
        if i % 2 == 0 && b > 0 {
            if c > 0 || d > 0 {
                n += match mode {
                    "add" => i + b,
                    "sub" => i - b,
                    "mul" => i * b,
                    "div" if b != 0 => i / b,
                    _ => 0,
                };
            } else if c < 0 && d < 0 {
                n -= 1;
            } else {
                while n > 100 {
                    n /= 2;
                    if n % 3 == 0 || n % 5 == 0 {
                        break;
                    }
                }
            }
        } else if i % 3 == 0 {
            n += if d > c { d - c } else { c - d };
        } else {
            for j in 0..b {
                if j > c && j < d {
                    n += 1;
                } else if j == c || j == d {
                    n -= 1;
                }
            }
        }
    }
    n
}
RUST
}

control() {
  local work
  work=$(mktemp -d)
  # shellcheck disable=SC2064  # expand now: $work is local and gone when the trap fires
  trap "rm -rf '$work'" RETURN
  ARMS_RED=0
  printf '[tdg]\nbaseline = 5\n' >"$work/gates.toml"

  report "$work/held.json" Warn "5 definition(s) below minimum grade A across 3 file(s), at the recorded baseline of 5 — this is debt held flat, not a clean tree."
  arm 1 HELD 0 PASS classify "$work/held.json" "$work/gates.toml"
  report "$work/over.json" Fail "6 definition(s) below minimum grade A — 1 OVER the recorded baseline of 5. A ratchet holds only if new debt is refused."
  arm 2 OVER 1 OVER classify "$work/over.json" "$work/gates.toml"
  report "$work/beaten.json" Warn "4 definition(s) below minimum grade A across 3 file(s), at the recorded baseline of 5 — this is debt held flat, not a clean tree. The tree is 1 under the recorded baseline."
  arm 3 BEATEN 1 BEATEN classify "$work/beaten.json" "$work/gates.toml"
  report "$work/skip.json" Skip "not selected (--checks)"
  arm 4 SKIPPED 1 "NOT MEASURED" classify "$work/skip.json" "$work/gates.toml"
  report "$work/unmeasured.json" Fail "Not measured: the project holds no code to grade, so the recorded \`[tdg] baseline\` of 5 was never checked."
  arm 5 UNMEASURED 1 "NOT MEASURED" classify "$work/unmeasured.json" "$work/gates.toml"
  jq -n '{checks: [{name: "CB-1700: Branch Protection", status: "Pass", message: "ok"}]}' >"$work/absent.json"
  arm 6 ABSENT 1 "NOT MEASURED" classify "$work/absent.json" "$work/gates.toml"
  report "$work/stale.json" Warn "5 definition(s) below minimum grade A across 3 file(s), at the recorded baseline of 5 — this is debt held flat, not a clean tree. (index is stale: source files are newer than .pmat/context.db, so this count describes an OLDER tree)"
  arm 7 STALE 1 STALE classify "$work/stale.json" "$work/gates.toml"
  report "$work/disagree.json" Warn "5 definition(s) below minimum grade A across 3 file(s), at the recorded baseline of 6 — this is debt held flat, not a clean tree."
  arm 8 DISAGREE 1 DISAGREE classify "$work/disagree.json" "$work/gates.toml"

  # The real pmat, over a project with one definition planted below grade A. CB-200 builds its
  # index under PMAT_CACHE_DIR, so the fixture tree is never written to and nothing is shared.
  planted_project "$work/over" 0
  arm 9 PLANTED-OVER 1 OVER env PMAT_CACHE_DIR="$work/cache-over" bash -c "$(declare -f recorded_baseline classify measure); PMAT='$PMAT' measure '$work/over'"
  planted_project "$work/held" 1
  arm 10 PLANTED-HELD 0 PASS env PMAT_CACHE_DIR="$work/cache-held" bash -c "$(declare -f recorded_baseline classify measure); PMAT='$PMAT' measure '$work/held'"

  if [ "$ARMS_RED" -ne 0 ]; then
    echo "control: $ARMS_RED arm(s) RED — this gate cannot be trusted to fail; fix it before reading its verdict on the tree"
    return 1
  fi
  echo "control: all 10 arms GREEN — every verdict is reachable, and a planted below-A definition is refused"
}

case "$MODE" in
  classify)
    if [ -z "$REPORT" ] || [ -z "$GATES" ]; then
      echo "cb200-ratchet-gate.sh: --classify needs REPORT and --gates FILE" >&2
      exit 2
    fi
    classify "$REPORT" "$GATES" ;;
  control)
    require_pmat
    PMAT=$(cd "$(dirname "$PMAT")" && pwd)/$(basename "$PMAT")
    control ;;
  measure)
    require_pmat
    measure "$ROOT" ;;
esac
