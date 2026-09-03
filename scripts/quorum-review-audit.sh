#!/usr/bin/env bash
# quorum-review-audit — AD-04 acceptance: the merge helper refuses `--auto` without an agreeing quorum verdict.
#   scripts/quorum-review-audit.sh [--clean <artifact.json>] [--planted <artifact.json>]
# Legs 1-4 are offline and deterministic (a stub `gh` on PATH answers `pr view` and records `pr merge`).
# Legs 5-6 read the two live lane artifacts when given: the clean diff must be three PASS, the planted
# contradiction must carry at least one FAIL naming the planted file. Exit 0 = every leg green.
set -euo pipefail
SKILL_DIR=${QUORUM_SKILL_DIR:-$HOME/.claude/skills/quorum-review}
CLEAN=""; PLANTED=""
while [ $# -gt 0 ]; do case "$1" in --clean) CLEAN=$2; shift 2;; --planted) PLANTED=$2; shift 2;; *) echo "unknown argument $1" >&2; exit 2;; esac; done
red=0; leg(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
echo "quorum-review-audit (AD-04) — skill dir: $SKILL_DIR"
# leg 1: the helper exists and is executable
if [ -x "$SKILL_DIR/pmat-merge" ]; then leg "helper present: $SKILL_DIR/pmat-merge" 0; else leg "helper present (missing: $SKILL_DIR/pmat-merge)" 1; echo "RED"; exit 1; fi
T=$(mktemp -d); [ -n "$T" ] && [ "$T" != "/" ] || exit 2
cleanup() {
  if [ -n "${T:-}" ] && [ "$T" != "/" ] && [ -d "$T" ]; then find "$T" -type f -delete; find "$T" -depth -type d -empty -delete; fi
}
trap cleanup EXIT
HEAD=1111111111111111111111111111111111111111
cat > "$T/gh" <<STUB
#!/usr/bin/env bash
case "\$1 \$2" in
  "pr view") echo "$HEAD";;
  "pr merge") shift 2; printf '%s\n' "\$*" > "$T/merge-args"; exit 0;;
  *) exit 99;;
esac
STUB
chmod +x "$T/gh"; export PATH="$T:$PATH"
run(){ ( cd "$T/repo" && "$SKILL_DIR/pmat-merge" 42 "$@" ); }
mkdir -p "$T/repo/docs/audits"
# leg 2: no artifact → refuse (exit 1) naming the missing file, and gh pr merge is never called
rm -f "$T/merge-args"; out=$(run --auto --merge 2>&1 || echo "rc=$?")
if echo "$out" | grep -q 'rc=1' && echo "$out" | grep -q 'quorum-<ticket>.json' && [ ! -e "$T/merge-args" ]; then leg "no artifact → refused, names the file, merge not called" 0; else leg "no artifact → refused (got: ${out:0:80})" 1; fi
# leg 3: artifact for another head, and agreed=false for this head → both refuse
printf '{"ticket":"PMAT-1","head":"2222222222222222222222222222222222222222","agreed":true,"lanes":[]}\n' > "$T/repo/docs/audits/quorum-PMAT-1.json"
printf '{"ticket":"PMAT-2","head":"%s","agreed":false,"lanes":[]}\n' "$HEAD" > "$T/repo/docs/audits/quorum-PMAT-2.json"
rm -f "$T/merge-args"; rc=0; run --auto --merge >/dev/null 2>&1 || rc=$?
if [ "$rc" = 1 ] && [ ! -e "$T/merge-args" ]; then leg "other head / agreed=false → refused" 0; else leg "other head / agreed=false → refused (rc=$rc)" 1; fi
# leg 4: agreeing artifact for this head → gh pr merge is called with --auto
printf '{"ticket":"PMAT-3","head":"%s","agreed":true,"lanes":[]}\n' "$HEAD" > "$T/repo/docs/audits/quorum-PMAT-3.json"
rm -f "$T/merge-args"; rc=0; run --auto --merge >/dev/null 2>&1 || rc=$?
if [ "$rc" = 0 ] && grep -q -- '--auto' "$T/merge-args" 2>/dev/null; then leg "agreeing artifact → gh pr merge --auto called" 0; else leg "agreeing artifact → merge (rc=$rc)" 1; fi
# leg 4b: a non-auto merge is passed through regardless of artifacts
rm -f "$T/repo/docs/audits/"*.json "$T/merge-args"; rc=0; run --merge >/dev/null 2>&1 || rc=$?
if [ "$rc" = 0 ] && [ -e "$T/merge-args" ]; then leg "non-auto merge passes through" 0; else leg "non-auto merge passes through (rc=$rc)" 1; fi
verd(){ python3 -c 'import json,sys; a=json.load(open(sys.argv[1])); print(a["width"], sum(1 for l in a["lanes"] if l["verdict"]=="PASS"), sum(1 for l in a["lanes"] if l["verdict"]=="FAIL"), a["agreed"])' "$1"; }
if [ -n "$CLEAN" ]; then read -r w p f a < <(verd "$CLEAN"); if [ "$a" = True ] && [ "$p" = "$w" ]; then leg "clean diff: $p/$w PASS, agreed" 0; else leg "clean diff: $p/$w PASS ($f FAIL), agreed=$a" 1; fi; fi
if [ -n "$PLANTED" ]; then read -r w p f a < <(verd "$PLANTED"); named=$(python3 -c 'import json,sys; a=json.load(open(sys.argv[1])); print(sum(1 for l in a["lanes"] for x in l["findings"] if "commit_enforcement_tests" in x.get("file","")))' "$PLANTED"); if [ "$a" = False ] && [ "$f" -ge 1 ] && [ "$named" -ge 1 ]; then leg "planted contradiction: $f/$w FAIL, $named finding(s) name the planted file, not agreed" 0; else leg "planted contradiction: $f/$w FAIL, named=$named, agreed=$a" 1; fi; fi
[ "$red" = 0 ] && echo "GREEN" || { echo "RED"; exit 1; }
