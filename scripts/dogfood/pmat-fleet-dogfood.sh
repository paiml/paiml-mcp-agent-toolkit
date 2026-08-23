#!/usr/bin/env bash
# Run a RELEASE-INSTALLED pmat against one real repository and classify what
# happens. One repo per invocation; the caller fans out.
#
# Why this exists. Every other gate in the protocol tests pmat against fixtures
# pmat's own authors wrote, so the fixture and the code share an author and
# confirm each other. These repos do not: they are 10 to 60,980 files of code
# nobody wrote to make pmat look good. The 3.32.0 cycle found `analyze
# complexity` returning Ok(empty) on 3,992 readable files, a grader decided by
# `rustfmt.toml`, and a flag sweep that called wired flags inert — none of which
# any fixture caught.
#
# Two DIFFERENT kinds of finding come out of this and must not be conflated:
#
#   PMAT-DEFECT   pmat crashed, hung, emitted unparseable JSON, or reported
#                 success having measured nothing. A bug in pmat. Ticket goes
#                 to paiml/paiml-mcp-agent-toolkit.
#   REPO-FINDING  pmat worked and found a real quality problem in the target.
#                 Ticket goes to that repo.
#
# Conflating them is how a tool's own bugs get filed as other people's debt.

set -uo pipefail
BIN="${BIN:?set BIN to the installed pmat}"
REPO="${1:?usage: fleet-dogfood.sh <repo-path>}"
NAME="$(basename "$REPO")"
TIMEOUT="${TIMEOUT:-300}"
OUT="${OUT:-/tmp/fleet-$NAME.json}"

defects=()   # PMAT-DEFECT
findings=()  # REPO-FINDING
probes=0

emit() { printf '%s\n' "$1"; }
jesc() { python3 -c 'import json,sys;print(json.dumps(sys.stdin.read()))' <<<"$1"; }

# rs_files: the denominator. A tool reporting zero over a non-zero denominator
# is the defect class this whole protocol exists to catch.
RS=$(find "$REPO" -name '*.rs' -not -path '*/target/*' -not -path '*/.git/*' 2>/dev/null | wc -l)

probe() { # probe <label> <expect-findings:yes|no> <cmd...>
  local label="$1" expect="$2"; shift 2
  probes=$((probes + 1))
  local out code
  out=$(timeout "$TIMEOUT" "$@" 2>&1); code=$?

  if [ "$code" -eq 124 ]; then
    defects+=("$label: TIMED OUT after ${TIMEOUT}s on $NAME ($RS .rs files)")
    return
  fi
  if printf '%s' "$out" | grep -q "panicked at"; then
    defects+=("$label: PANICKED — $(printf '%s' "$out" | grep -m1 'panicked at' | cut -c1-160)")
    return
  fi
  # A tool that exits 0 while printing an error is the "absence rendered as
  # success" shape: the diagnosis goes to stderr and the exit code says fine.
  if [ "$code" -eq 0 ] && printf '%s' "$out" | grep -qiE '^error:|^Error:'; then
    defects+=("$label: EXIT 0 WITH AN ERROR ON STDERR — $(printf '%s' "$out" | grep -m1 -iE '^error:' | cut -c1-140)")
    return
  fi
  # Reported on a real Rust repo having analysed nothing.
  if [ "$code" -eq 0 ] && [ "$RS" -gt 20 ] \
     && printf '%s' "$out" | grep -qE '"(files_analyzed|total_files)"[[:space:]]*:[[:space:]]*0\b'; then
    defects+=("$label: analysed 0 files and exited 0, on a repo with $RS .rs files")
    return
  fi
}

probe_json() { # probe_json <label> <cmd...> — the payload must parse
  local label="$1"; shift
  probes=$((probes + 1))
  local out code
  out=$(timeout "$TIMEOUT" "$@" 2>/dev/null); code=$?
  [ "$code" -eq 124 ] && { defects+=("$label: TIMED OUT after ${TIMEOUT}s"); return; }
  [ -z "$out" ] && { defects+=("$label: --format json produced NO output (exit $code)"); return; }
  printf '%s' "$out" | python3 -c 'import json,sys;json.load(sys.stdin)' 2>/dev/null \
    || defects+=("$label: --format json emitted unparseable JSON (exit $code, ${#out} bytes)")
}

emit "── $NAME  ($RS .rs files) ─────────────────────────────"

probe      "analyze complexity"  yes "$BIN" analyze complexity --path "$REPO"
probe_json "analyze complexity"      "$BIN" analyze complexity --path "$REPO" --format json
probe      "analyze satd"        yes "$BIN" analyze satd --path "$REPO"
probe      "analyze dead-code"   yes "$BIN" analyze dead-code --path "$REPO"
probe      "analyze duplicates"  yes "$BIN" analyze duplicates --path "$REPO"
probe      "tdg"                 yes "$BIN" tdg "$REPO"
probe_json "tdg"                     "$BIN" tdg "$REPO" --format json
probe      "analyze churn"       yes "$BIN" analyze churn --path "$REPO"

# ── REPO-FINDINGS: pmat worked; what did it say about the target?
TDG=$(timeout "$TIMEOUT" "$BIN" tdg "$REPO" --format json 2>/dev/null)
if [ -n "$TDG" ]; then
  # `score` is a NESTED object — {"total": .., "grade": .., "breakdown": ..} —
  # so reading it as a scalar splits a dict across three shell words. Measured
  # on cohete before this was fixed:
  #   score={'total': grade=92.637 files='grade': 'A', ...}
  # A parser that produces that is not a parser; it is three fields of noise
  # that would have been reported as a repo's grade.
  read -r SCORE GRADE NFILES NOTMEAS <<<"$(printf '%s' "$TDG" | python3 -c '
import json, sys
try:
    d = json.load(sys.stdin)
    sc = d.get("score")
    if isinstance(sc, dict):
        total, grade = sc.get("total", 0), sc.get("grade", "?")
    else:
        total, grade = sc or d.get("overall_score") or 0, d.get("grade", "?")
    print(round(float(total), 2), grade, d.get("files_analyzed") or 0,
          str(bool(d.get("not_measured"))).lower())
except Exception as e:
    print(0, "PARSE_ERROR", 0, "true")' 2>/dev/null)"

  # `not_measured` is pmat telling us it graded nothing. A grade printed beside
  # it is not a grade.
  if [ "$NOTMEAS" = "true" ] && [ "$RS" -gt 20 ]; then
    defects+=("tdg: not_measured=true on a repo with $RS .rs files, yet it still reported grade=$GRADE")
  fi
  [ "$GRADE" = "PARSE_ERROR" ] && defects+=("tdg --format json: payload did not match the documented shape")
  emit "   tdg: score=$SCORE grade=$GRADE files=$NFILES"
  case "$GRADE" in
    C*|D*|F*) findings+=("overall TDG grade $GRADE (score $SCORE) over $NFILES files") ;;
  esac
fi

SATD=$(timeout "$TIMEOUT" "$BIN" analyze satd --path "$REPO" 2>/dev/null | grep -oE '[0-9]+ (SATD )?(items?|markers?|violations?)' | head -1)
[ -n "$SATD" ] && emit "   satd: $SATD"

printf '{"repo":%s,"rs_files":%s,"probes":%s,"pmat_defects":%s,"repo_findings":%s}\n' \
  "$(jesc "$NAME")" "$RS" "$probes" \
  "$(printf '%s\n' "${defects[@]:-}"  | python3 -c 'import json,sys;print(json.dumps([l for l in sys.stdin.read().split("\n") if l.strip()]))')" \
  "$(printf '%s\n' "${findings[@]:-}" | python3 -c 'import json,sys;print(json.dumps([l for l in sys.stdin.read().split("\n") if l.strip()]))')" \
  > "$OUT"

emit "   probes=$probes  PMAT-DEFECTS=${#defects[@]}  REPO-FINDINGS=${#findings[@]}"
for d in "${defects[@]:-}";  do [ -n "$d" ] && emit "   ✗ PMAT: $d"; done
for f in "${findings[@]:-}"; do [ -n "$f" ] && emit "   • REPO: $f"; done
[ "${#defects[@]}" -eq 0 ]
