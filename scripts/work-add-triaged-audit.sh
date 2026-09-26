#!/usr/bin/env bash
# work-add-triaged-audit — FLOW-03 (#1440) acceptance: `pmat work add` refuses an untriaged ticket
# (no priority, no kind, no epic, a closed/unlabelled epic, no issue to link) and writes nothing; a
# triaged add links the ticket's issue under the epic on GitHub BEFORE writing the row, and the row
# carries epic:, the priority and a kind: label. Contract: contracts/pmat-work-add-triaged-v1.yaml.
#   PMAT=<path to the binary under test> scripts/work-add-triaged-audit.sh
# GitHub is a stub `gh` on PATH that serves fixture issues and logs every call.
# Exit 0 = every arm green; 1 = an arm red; 2 = harness failure.
set -uo pipefail
# --self-test: the audit must be able to fail. Run it against a pmat that does nothing and
# require RED; a checker that passes on a no-op binary checks nothing.
if [ "${1:-}" = "--self-test" ]; then
  S=$(mktemp -d); printf '#!/bin/sh\nexit 0\n' > "$S/pmat"; chmod +x "$S/pmat"
  if PMAT="$S/pmat" bash "$0" >/dev/null 2>&1; then echo "self-test FAILED: the audit passed against a pmat that does nothing"; rm -rf "$S"; exit 1; fi
  rm -rf "$S"; echo "self-test OK: the audit is RED against a pmat that does nothing"; exit 0
fi
PMAT=${PMAT:-}
if [ -z "$PMAT" ] || ! command -v "$PMAT" >/dev/null 2>&1; then echo "PMAT='$PMAT' is not an executable" >&2; exit 2; fi
case "$PMAT" in */*) ;; *) echo "PMAT must be a path, not a bare name" >&2; exit 2;; esac
red=0; arm(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "work-add-triaged-audit (#1440) — $($PMAT --version 2>/dev/null | head -1)"
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
[[ "$T" == /* && "$T" != *..* ]] || exit 2
cleanup() { if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -depth -delete 2>/dev/null; fi; }
trap cleanup EXIT

# ── the stub GitHub ─────────────────────────────────────────────────────────
mkdir -p "$T/bin" "$T/gh/issues" "$T/gh/subs"
: > "$T/gh/calls.log"                          # exists from the start, so posts() always prints a count
issue() { printf '{"id":%s,"number":%s,"state":"%s","labels":[%s]}\n' "$2" "$1" "$3" "$4" > "$T/gh/issues/$1.json"; }
issue 10 9010 open '{"name":"epic"}'          # the epic
issue 11 9011 closed '{"name":"epic"}'        # a closed epic
issue 12 9012 open '{"name":"kind:code"}'     # an open issue that is not an epic
for n in 20 21 22 23 24 25; do issue "$n" "90$n" open ''; done
echo 21 > "$T/gh/subs/10"                     # #21 is already under #10
cat > "$T/bin/gh" <<'STUB'
#!/usr/bin/env bash
D=$(dirname "$(dirname "$0")")/gh
printf '%s\n' "$*" >> "$D/calls.log"
[ "$1" = api ] || exit 1; shift
post=0; paginate=0; path=""; field=""
while [ $# -gt 0 ]; do case "$1" in
  -X) [ "$2" = POST ] && post=1; shift 2;; --paginate) paginate=1; shift;; --jq) shift 2;;
  -F) field=$2; shift 2;; *) path=$1; shift;; esac; done
case "$path" in
  */issues/*/sub_issues)
    e=${path%/sub_issues}; e=${e##*/}
    if [ "$post" = 1 ]; then
      id=${field#sub_issue_id=}; [ "$id" = 9023 ] && { echo "HTTP 422: Unprocessable Entity" >&2; exit 1; }
      n=$(grep -l "\"id\":$id," "$D"/issues/*.json | head -1); n=${n##*/}; echo "${n%.json}" >> "$D/subs/$e"; echo '{}'
    else cat "$D/subs/$e" 2>/dev/null; true; fi ;;
  */issues/*) n=${path##*/}; [ -f "$D/issues/$n.json" ] || { echo "HTTP 404: Not Found" >&2; exit 1; }; cat "$D/issues/$n.json" ;;
  *) exit 1 ;;
esac
STUB
chmod +x "$T/bin/gh"
export PATH="$T/bin:$PATH"
posts() { grep -c -- '-X POST' "$T/gh/calls.log" 2>/dev/null || true; }

# ── the fixture repository ──────────────────────────────────────────────────
R="$T/fx"; mkdir -p "$R/docs/roadmaps"
printf "roadmap_version: '1.0'\ngithub_enabled: false\ngithub_repo: fx/fx\nroadmap: []\n" > "$R/docs/roadmaps/roadmap.yaml"
git -C "$R" init -q && git -C "$R" add . && git -C "$R" -c core.hooksPath=/dev/null -c user.email=a@b -c user.name=a commit -qm fx || exit 2
RM="$R/docs/roadmaps/roadmap.yaml"
add() { (cd "$R" && "$PMAT" work add "$@" --path "$R") > "$T/out" 2>&1; }
row() { awk -v id="$1" '$0 ~ "^- id: "id"$"{p=1;print;next} p && /^- id:/{p=0} p' "$RM"; }

# Refusals: each names what is missing, leaves the roadmap byte-identical, and posts nothing.
refused() { # <arm> <expected text> <add args...>
  local name=$1 want=$2; shift 2
  local before; before=$(sha256sum "$RM"); local p0; p0=$(posts)
  add "$@"; local rc=$?
  [ "$rc" != 0 ] && grep -qF -- "$want" "$T/out" && [ "$(sha256sum "$RM")" = "$before" ] && [ "$(posts)" = "$p0" ]
  arm "$name (exit $rc)" $?
}
refused "1  the pre-FLOW-03 add (no epic, no priority, no kind) is refused" "--priority P0|P1|P2|P3" "t1" --github-issue 20
refused "2  no --priority is refused"                   "--priority P0|P1|P2|P3" "t2" --github-issue 20 --epic 10 --kind code
refused "3  no --kind and no kind: tag is refused"      "--kind code"            "t3" --github-issue 20 --epic 10 -p P1
refused "4  no --epic is refused"                       "--epic <issue#>"        "t4" --github-issue 20 -p P1 --kind code
refused "5  a closed epic is refused"                   "#11 is closed"          "t5" --github-issue 20 -p P1 --kind code --epic 11
refused "6  an epic without the epic label is refused"  "not labelled \`epic\`"  "t6" --github-issue 20 -p P1 --kind code --epic 12
refused "7  --epic with no issue to link is refused"    "--github-issue N"       "t7" --id PMAT-777 -p P1 --kind code --epic 10

# 8: a triaged add links first (one POST, the child's REST id), then writes the row with epic:, priority, kind label.
add "t8" --github-issue 20 -p P1 --kind code --epic 10; rc=$?
r=$(row PMAT-020)
[ "$rc" = 0 ] && [ "$(posts)" = 1 ] && grep -q -- '-X POST repos/{owner}/{repo}/issues/10/sub_issues -F sub_issue_id=9020' "$T/gh/calls.log" \
  && grep -qx '  epic: 10' <<<"$r" && grep -qx '  priority: high' <<<"$r" && grep -qx '  - kind:code' <<<"$r"
arm "8  a triaged add links #20 under #10, then writes epic: 10, priority: high, kind:code (exit $rc)" $?

# 9: an issue already under the epic is not linked twice.
add "t9" --github-issue 21 -p P2 --kind docs --epic 10; rc=$?
r=$(row PMAT-021)
[ "$rc" = 0 ] && [ "$(posts)" = 1 ] && grep -qx '  epic: 10' <<<"$r" && grep -qx '  - kind:docs' <<<"$r"
arm "9  an issue already under the epic: no second link, row still carries epic: 10 (exit $rc)" $?

# 10: P0 with no milestone is accepted; the kind may come from a kind: tag.
add "t10" --github-issue 22 -p P0 -t kind:triage --epic 10; rc=$?
r=$(row PMAT-022)
[ "$rc" = 0 ] && grep -qx '  priority: critical' <<<"$r" && grep -qx '  - kind:triage' <<<"$r" && [ "$(grep -c 'kind:' <<<"$r")" = 1 ]
arm "10 P0 without a milestone is accepted; kind from a tag, recorded once (exit $rc)" $?

# 11: a failed link writes nothing.
before=$(sha256sum "$RM"); add "t11" --github-issue 23 -p P1 --kind code --epic 10; rc=$?
[ "$rc" != 0 ] && grep -qF "nothing was written" "$T/out" && [ "$(sha256sum "$RM")" = "$before" ]
arm "11 a failed link refuses and leaves the roadmap byte-identical (exit $rc)" $?

# 12: in fragment mode the fragment carries epic: and the kind label, and roadmap.yaml is untouched.
mkdir -p "$R/docs/roadmaps/entries"; before=$(sha256sum "$RM")
add "t12" --github-issue 24 -p P3 --kind code --epic 10; rc=$?
F="$R/docs/roadmaps/entries/PMAT-024.yaml"
[ "$rc" = 0 ] && [ -f "$F" ] && grep -q '^ *epic: 10$' "$F" && grep -q 'kind:code' "$F" && grep -q '^ *priority: low$' "$F" && [ "$(sha256sum "$RM")" = "$before" ]
arm "12 fragment mode: PMAT-024.yaml carries epic: 10, priority: low, kind:code; roadmap.yaml untouched (exit $rc)" $?

# Control: an arm-8-shaped add for an issue whose ticket already exists is refused BEFORE any link.
p0=$(posts); add "t13" --github-issue 20 -p P1 --kind code --epic 10; rc=$?
[ "$rc" != 0 ] && grep -qF "already exists" "$T/out" && [ "$(posts)" = "$p0" ]
arm "13 control: an existing ticket is refused before GitHub is written (exit $rc)" $?

[ "$red" = 0 ] && echo "work-add-triaged-audit: all arms green" || echo "work-add-triaged-audit: RED"
exit "$red"
