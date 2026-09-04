#!/usr/bin/env bash
# work-ladder-claim-audit — #1186 acceptance: a ticket claims the ladder level its bindings evidence, the claim
# can be set and the bindings attached after start, and the ladder check runs before the quality gates.
#   PMAT=<path to the binary under test> scripts/work-ladder-claim-audit.sh
# Exit 0 = every leg green; 1 = a leg red; 2 = harness failure.
set -uo pipefail
PMAT=${PMAT:-}
if [ -z "$PMAT" ] || ! command -v "$PMAT" >/dev/null 2>&1; then echo "PMAT='$PMAT' is not an executable" >&2; exit 2; fi
case "$PMAT" in */*) ;; *) echo "PMAT must be a path, not a bare name" >&2; exit 2;; esac
red=0; leg(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "work-ladder-claim-audit (#1186) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
[[ "$T" == /* && "$T" != *..* ]] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -depth -delete 2>/dev/null; fi; }
trap cleanup EXIT
R="$T/fx"; mkdir -p "$R/src" "$R/docs/roadmaps" "$R/contracts" "$R/docs/audits"
printf '[package]\nname = "fx"\nversion = "0.1.0"\nedition = "2021"\n' > "$R/Cargo.toml"; echo 'pub fn f() -> u32 { 1 }' > "$R/src/lib.rs"
printf "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: fx/fx\nroadmap: []\n" > "$R/docs/roadmaps/roadmap.yaml"
cat > "$R/contracts/fx-v1.yaml" <<'Y'
metadata:
  version: "1.0.0"
  created: "2026-09-04"
  author: t
  references: []
  registry: true
  description: fixture
  contract: fx
  status: draft
equations:
  one_is_one:
    formula: "f() == 1"
    domain: "fx"
    codomain: bool
    invariants: []
    preconditions: []
falsification_tests:
  - id: F1
    rule: "f returns one"
    prediction: "1"
    test: "cargo test"
    if_fails: "f changed"
Y
( cd "$R" && git init -q --template= && git config user.email t@t && git config user.name t && git config core.hooksPath /dev/null && git add . && git commit -qm init )
w(){ ( cd "$R" && timeout 300 "$PMAT" work "$@" 2>&1 ); }
level(){ python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(d.get("verification_level"))' "$R/.pmat-work/$1/contract.json" 2>/dev/null; }
binds(){ python3 -c 'import json,sys; d=json.load(open(sys.argv[1])); print(",".join(b.get("equation","?") for b in d.get("implements",[])))' "$R/.pmat-work/$1/contract.json" 2>/dev/null; }
newid(){ w add "$1" 2>&1 | grep -oE 'PMAT-[0-9]+' | head -1; }
# ---- 1: an unbound ticket claims L1, not L3
A=$(newid "unbound"); w start "$A" >/dev/null; [ "$(level "$A")" = L1 ] && leg "1: an unbound ticket started plainly claims L1 (got $(level "$A"))" 0 || leg "1: an unbound ticket claims L1 (got $(level "$A"), the shipped default is L3)" 1
# ---- 2: a ticket started with --implements claims L2
B=$(newid "bound at start"); w start "$B" --implements fx-v1/one_is_one >/dev/null; [ "$(level "$B")" = L2 ] && [ "$(binds "$B")" = one_is_one ] && leg "2: a ticket started with --implements claims L2 and carries the binding" 0 || leg "2: start --implements → L2 with the binding (got level $(level "$B"), binds '$(binds "$B")')" 1
# ---- 3: --level on add and edit sets the claim explicitly
C=$(w add "explicit" --level L2 2>&1 | grep -oE 'PMAT-[0-9]+' | head -1); if [ -n "$C" ]; then w start "$C" >/dev/null; [ "$(level "$C")" = L2 ] && leg "3a: add --level L2 is the claim after start" 0 || leg "3a: add --level L2 (got $(level "$C"))" 1; else leg "3a: add accepts --level (the shipped add has no such flag)" 1; fi
if w edit "$A" --level L2 >/dev/null 2>&1; then [ "$(level "$A")" = L2 ] && leg "3b: edit --level L2 rewrites the claim" 0 || leg "3b: edit --level L2 (got $(level "$A"))" 1; else leg "3b: edit accepts --level (the shipped edit has no such flag)" 1; fi
w edit "$A" --level L9 >/dev/null 2>&1 && leg "3c control: --level L9 is refused" 1 || leg "3c control: --level L9 is refused" 0
# ---- 4: --implements on edit binds an in-progress ticket
D=$(newid "bound after start"); w start "$D" >/dev/null
if w edit "$D" --implements fx-v1/one_is_one >/dev/null 2>&1; then
  if [ "$(binds "$D")" = one_is_one ] && [ "$(level "$D")" = L2 ]; then
    leg "4: edit --implements binds an in-progress ticket and lifts the claim to L2" 0
  else
    leg "4: edit --implements binds (binds '$(binds "$D")', level $(level "$D"))" 1
  fi
else
  leg "4: edit accepts --implements while InProgress (the shipped edit has no such flag; start refuses InProgress)" 1
fi
w edit "$D" --implements fx-v1/no_such_equation >/dev/null 2>&1 && leg "4b control: an unknown equation is refused" 1 || leg "4b control: an unknown equation is refused" 0
# ---- 5: complete checks the ladder before the quality gates
E=$(newid "overclaim"); w start "$E" >/dev/null; python3 -c 'import json,sys; p=sys.argv[1]; d=json.load(open(p)); d["verification_level"]="L3"; json.dump(d,open(p,"w"),indent=2)' "$R/.pmat-work/$E/contract.json"
echo "receipt" > "$R/docs/audits/impl-$E-receipt.md"; out=$(w complete "$E"); rc=$?
echo "$out" | grep -q 'LadderShortfall' && leg "5a: an over-claimed ticket is refused with LadderShortfall" 0 || leg "5a: over-claim refused (no LadderShortfall in: $(echo "$out" | tail -2 | tr '\n' ' ' | cut -c1-80))" 1
ql=$(echo "$out" | grep -nE -i 'quality|invariant|falsif' | head -1 | cut -d: -f1); ll=$(echo "$out" | grep -n 'LadderShortfall' | head -1 | cut -d: -f1)
if [ -n "$ll" ] && { [ -z "$ql" ] || [ "$ll" -lt "$ql" ]; }; then leg "5b: the ladder refusal comes before any quality-gate output" 0; else leg "5b: ladder refusal before the quality gates (ladder at line ${ll:-none}, quality output at line ${ql:-none})" 1; fi
# ---- 6 control: an honestly-claimed L1 ticket completes with a receipt
F=$(newid "honest"); w start "$F" >/dev/null; python3 -c 'import json,sys; p=sys.argv[1]; d=json.load(open(p)); d["verification_level"]="L1"; json.dump(d,open(p,"w"),indent=2)' "$R/.pmat-work/$F/contract.json"; echo "receipt" > "$R/docs/audits/impl-$F-receipt.md"
out=$(w complete "$F"); rc=$?
if echo "$out" | grep -q 'LadderShortfall'; then
  leg "6 control: an honestly-claimed L1 ticket is not refused by the ladder" 1
elif echo "$out" | grep -qiE 'usage:|more information'; then
  leg "6 control: complete ran (got a usage error: $(echo "$out" | head -1 | cut -c1-60))" 1
else
  leg "6 control: an honestly-claimed L1 ticket is not refused by the ladder (complete exit $rc)" 0
fi
[ "$red" = 0 ] && echo "GREEN" || { echo "RED"; exit 1; }
