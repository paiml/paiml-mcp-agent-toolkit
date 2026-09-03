#!/usr/bin/env bash
# CRUX-12 — reachability ledger + ratchet audit (spec §8.12, #1152, #1017).
#
# Legs 1-7 are the PERMANENT GATE: the two .pmat-ratchet.toml metrics exist and
# re-derive the analyzer's own numbers (in-process — see measure.rs), the
# ratchet evaluates them, the reachability-ledger CI job is wired into
# feature-gate's failing loop, every unreachable file has a ledger row with a
# reason from the closed enum, and every reason is a claim the tree verifies.
# Leg 8 is the COMPLETION CRITERION for the population (no orphan holds a
# #[test]) and is expected to be RED until #1017 is finished.
#
# Requires: jq, python3 (tomllib), git. PMAT=<binary> overrides the pmat used.
# Run from anywhere inside the checkout.
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
P=${PMAT:-pmat}; FM=.github/workflows/feature-matrix.yml; L=docs/status/orphan-files-ledger.md
tom(){ python3 - "$1" "$2" <<'PY'
import sys,tomllib
try: print(tomllib.load(open(".pmat-ratchet.toml","rb"))["metric"][sys.argv[1]][sys.argv[2]])
except KeyError: sys.exit(1)
PY
}
body(){ awk -v j="  $1:" '$0==j{f=1;next} f&&/^  [a-z0-9-]+:$/{f=0} f' "$FM"; }
cell(){ awk -F' *[|] *' -v f="\`$1\`" -v n="$2" '$2==f{print $n}' "$L"; }

# 0. CONTROL (green today, by design — the instrument works; that was never in doubt)
R=$("$P" analyze reachability -f json) || fail "leg 0 (CONTROL): the analyzer did not run"

# 1. the metrics exist — parsed as TOML, never grepped as a header spelling
CMD=$(tom orphan_files command)       || fail "leg 1a: no [metric.orphan_files] in .pmat-ratchet.toml"
QCMD=$(tom quarantined_files command) || fail "leg 1b: no [metric.quarantined_files]"

# 2. they MEASURE THE TREE. 2a+2c+2f together are what kill `command = '''echo 407'''`
grep -q 'analyze reachability' <<<"$CMD" || fail "leg 2a: orphan_files never invokes the analyzer"
OBS=$(bash -o pipefail -c "$CMD")        || fail "leg 2b: the orphan_files command does not run"
[ "$OBS" = "$(jq -er .orphan_count <<<"$R")" ]       || fail "leg 2c: metric $OBS != analyzer $(jq -er .orphan_count <<<"$R")"
QOBS=$(bash -o pipefail -c "$QCMD")      || fail "leg 2d: the quarantined_files command does not run"
[ "$QOBS" = "$(jq -er .quarantined_count <<<"$R")" ] || fail "leg 2e: quarantined_files is not the analyzer's number"
#    2f — DIFFERENTIAL: the same command, on a tree whose true answer is 1, not 407
T=$(mktemp -d); cleanup(){ case "${T:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$T" ] && rm -rf -- "$T";; esac; }; trap cleanup EXIT
( cd "$T" && git init -q . && mkdir src \
  && printf '[package]\nname="fx"\nversion="0.0.0"\nedition="2021"\n' > Cargo.toml \
  && printf '\n' > src/lib.rs && printf '#[test]\nfn a(){}\n' > src/b.rs && git add -A \
  && git -c user.email=t@t -c user.name=t commit -q --no-verify -m fx >/dev/null )
FT=$(cd "$T" && "$P" analyze reachability -f json | jq -er .orphan_count) || fail "leg 2f: the fixture did not measure"
[ "$FT" = 1 ] || fail "leg 2f: the fixture is not the one this leg needs (orphan_count $FT, expected 1)"
FO=$(cd "$T" && bash -o pipefail -c "$CMD" | tr -d '[:space:]')
[ "$FO" = "$FT" ] || fail "leg 2f: on a fixture whose orphan_count is $FT the metric printed $FO — it measures no tree"

# 3. BUILD-COST LEG. the ratchet EVALUATES it — decoupled from the ratchet's own verdict, so an
#    unrelated red metric cannot masquerade as "not evaluated". Note the `|| true` and the absent pipe.
OUT=$("$P" comply ratchet -p . 2>&1 || true)
grep -qE '^orphan_files +([0-9]+|unmeasured) +/' <<<"$OUT" || fail "leg 3a: comply ratchet does not evaluate orphan_files"
! grep -qE '^orphan_files +unmeasured +/'        <<<"$OUT" || fail "leg 3b: unmeasurable is not a pass"

# 4. the instrument is WIRED — to the leg that can actually fail the gate, not only to `needs:`
grep -qE '^  reachability-ledger:' "$FM"                     || fail "leg 4a: no reachability-ledger job"
body reachability-ledger | grep -q 'analyze reachability'    || fail "leg 4b: the job does not run the analyzer"
G=$(awk '/^  feature-gate:/,0' "$FM")
grep -qE '^    needs: \[.*reachability-ledger.*\]' <<<"$G"   || fail "leg 4c: not in feature-gate's needs"
grep -q 'needs\.reachability-ledger\.result'       <<<"$G"   || fail "leg 4d: not in feature-gate's require-every-leg loop"

# 5. the ledger accounts for EVERY unreachable file, with the analyzer's own tests+lines
[ -f "$L" ] || fail "leg 5a: $L does not exist"
while read -r f t l; do
  r=$(cell "$f" 3); [ -n "$r" ] || fail "leg 5b: no ledger row for $f"
  case "$r" in registered-*|pending-\#[0-9]*|deleted-*|quarantined-\#[0-9]*) ;;
               *) fail "leg 5c: reason '$r' for $f is outside the closed enum" ;; esac
  [ "$(cell "$f" 4)" = "$t" ] && [ "$(cell "$f" 5)" = "$l" ] \
    || fail "leg 5d: $f row disagrees with the analyzer ($t tests / $l lines)"
done < <(jq -r '(.orphans[],.quarantined[])|"\(.file) \(.tests) \(.lines)"' <<<"$R")
[ "$(grep -cE '^\| `[^`]+` \| pending-#'     "$L")" = "$(jq -er .orphan_count <<<"$R")" ]      || fail "leg 5e: pending- rows != orphan_count"
[ "$(grep -cE '^\| `[^`]+` \| quarantined-#' "$L")" = "$(jq -er .quarantined_count <<<"$R")" ] || fail "leg 5f: quarantined- rows != quarantined_count"

# 6. every reason is a CLAIM THE ANALYZER CHECKS — so relocation cannot be spelled as a fix
UN=$(jq -r '(.orphans[],.quarantined[]).file' <<<"$R" | sort); QU=$(jq -r '.quarantined[].file' <<<"$R" | sort)
while IFS= read -r ln; do
  f=$(sed -E 's/^\| `([^`]*)` .*/\1/' <<<"$ln"); r=$(awk -F' *[|] *' '{print $3}' <<<"$ln")
  case "$r" in
    registered-*)  git ls-files --error-unmatch "$f" >/dev/null 2>&1 || fail "leg 6a: registered $f is not tracked"
                   ! grep -qxF "$f" <<<"$UN" || fail "leg 6a: $f claims registered- and is still unreachable" ;;
    deleted-*)     ! git ls-files --error-unmatch "$f" >/dev/null 2>&1 || fail "leg 6b: $f claims deleted- and is tracked" ;;
    quarantined-*) grep -qxF "$f" <<<"$QU" || fail "leg 6c: $f claims quarantined- and the analyzer disagrees" ;;
    pending-*)     grep -qxF "$f" <<<"$UN" || fail "leg 6d: $f claims pending- and is reachable — stale row" ;;
  esac
done < <(grep -E '^\| `[^`]+` \|' "$L")

# 7. ANTI-VACUITY GUARD (green today, by design — labelled, not evidence). Conservation as a git
#    DELTA between two tree-ishes: no constant to transcribe, and no constant to re-baseline.
census(){ git grep -cE '^[[:space:]]*#\[(tokio::)?test' "$1" -- 'src/*.rs' 'tests/*.rs' | awk -F: '{n+=$3} END{print n+0}'; }
B=$(git merge-base HEAD origin/master) || fail "leg 7a: no merge-base to compare against"
NOW=$(census HEAD); WAS=$(census "$B")
[ "$NOW" -gt 0 ] && [ "$WAS" -gt 0 ] || fail "leg 7b: the census predicate measured nothing — it has rotted"
[ "$NOW" -ge "$WAS" ] || fail "leg 7c: declared #[test] fell $WAS -> $NOW; orphans were deleted, not registered"

# 8. COMPLETION CRITERION (#1017), not the day-one gate: an invariant, not a count
jq -e '[.orphans[]|select(.tests>0)]|length == 0' <<<"$R" >/dev/null \
  || fail "leg 8: $(jq -r '[.orphans[]|select(.tests>0)]|length' <<<"$R") orphan files still hold #[test] fns"
echo PASS
