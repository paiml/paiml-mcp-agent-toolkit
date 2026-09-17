#!/usr/bin/env bash
# gate-control.sh — prove `make gate` can fail before its green is read (PMAT-1365).
#
#   scripts/gate-control.sh
#
# `scripts/gate.sh` is only worth running if every property it promises can be seen to
# break. Each arm feeds it a FIXTURE table and a fixture workflow under a scratch
# directory — nothing here runs the repository's real legs, and nothing is written to
# the tree — and asserts both the exit code and the text that justifies it:
#
#   arm 1  DECLARED    `make -n gate` resolves, and to scripts/gate.sh — not to a guess
#   arm 2  LIST        the real table: every required context has a row, every step source
#                      resolves in .github/workflows, the extension-point marker is present
#   arm 3  GREEN       passing legs exit 0 AND still print every CI-ONLY row by name, with the
#                      floor sentence; a step leg ran the text read from the workflow
#   arm 4  RED         one failing leg exits 1, names it, still prints every CI-ONLY row, and
#                      does not stop the legs after it (one run names every red leg)
#   arm 5  STEP-GONE   a step renamed in the workflow is a FAIL, never a skip
#   arm 6  STEP-GITHUB a step that needs GitHub Actions (`${{ }}`) is a FAIL
#   arm 7  STEP-RED    a step whose script fails is a FAIL
#   arm 8  UNACCOUNTED a required context with no row is refused (exit 2) before any leg runs
#   arm 9  NO-REASON   a CI-ONLY row without `<category>: <reason>` is refused (exit 2)
#   arm 10 VACUOUS     a table in which nothing runs is refused (exit 2)
#   arm 11 LIVE        the pinned REQUIRED_CONTEXTS equal master's branch protection plus the
#                      org ruleset — measured through `gh api` when it can be reached, and
#                      printed as NOT MEASURED when it cannot (never as a pass)
#   arm 12 FRESH-BINARY every leg of the REAL table that runs pmat runs the executable cargo
#                      reports building — a stale pmat planted at ./target/debug/pmat is never run
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 a prerequisite
# (bash, make, python3 with PyYAML) is missing, so nothing was judged.
set -uo pipefail

here="$(cd "$(dirname "$0")/.." && pwd)"
gate="$here/scripts/gate.sh"
failed=0
fail_arm() { echo "gate-control: arm $1 FAILED — $2" >&2; failed=$((failed + 1)); }

for tool in make python3; do
  command -v "$tool" >/dev/null 2>&1 || { echo "gate-control: $tool is required" >&2; exit 2; }
done
python3 -c 'import yaml' 2>/dev/null || { echo "gate-control: python3 needs PyYAML" >&2; exit 2; }
[ -f "$gate" ] || { echo "gate-control: arm 1 FAILED — $gate does not exist" >&2; exit 1; }

work="$(mktemp -d "${TMPDIR:-/tmp}/gate-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT

# ── arm 1: DECLARED ────────────────────────────────────────────────────────────────────────
if out=$(make -C "$here" -n gate 2>&1); then
  printf '%s\n' "$out" | grep -q 'scripts/gate.sh' \
    || fail_arm 1 "make -n gate resolves but does not run scripts/gate.sh: $(printf '%s\n' "$out" | grep -v '^make' | head -3 | tr '\n' ' ')…"
else
  fail_arm 1 "make -n gate does not resolve: $(printf '%s\n' "$out" | tail -2 | tr '\n' ' ')"
fi
[ "$failed" != 0 ] || echo "gate-control: arm 1 DECLARED    — make -n gate runs scripts/gate.sh"

# ── arm 2: LIST ────────────────────────────────────────────────────────────────────────────
f2=$failed
out=$(bash "$gate" --list 2>&1); rc=$?
[ "$rc" -eq 0 ] || fail_arm 2 "scripts/gate.sh --list exited $rc on the real table: $(printf '%s' "$out" | tail -5)"
grep -qF -- '── EXTENSION POINT' "$gate" || fail_arm 2 "the extension-point marker is gone from scripts/gate.sh"
[ "$failed" != "$f2" ] || echo "gate-control: arm 2 LIST        — $(printf '%s' "$out" | tail -1)"

# ── fixtures ───────────────────────────────────────────────────────────────────────────────
root="$work/root"
mkdir -p "$root/.github/workflows"
cat > "$root/.github/workflows/w.yml" <<'YML'
name: fixture
on: [push]
jobs:
  j:
    runs-on: ubuntu-latest
    steps:
      - name: passes
        run: echo "step text read from the workflow"
      - name: fails
        run: exit 7
      - name: needs github
        run: echo 'bash would run this happily, so only the expression check can fail it ${{ github.sha }}'
YML

# every required context accounted for, CI-only, with a reason
required_rows() {
  cat <<'ROWS'
ci-only | gate | r-gate | fixture | platform: fixture | -
ci-only | ci / gate | r-ci-gate | fixture | credential: fixture | -
ci-only | docs build (docs.rs environment) | r-docs | fixture | cost: fixture | -
ci-only | feature-gate | r-feature | fixture | cost: fixture | -
ci-only | pmat score | r-score | fixture | trigger: fixture | -
ci-only | provable ladder | r-ladder | fixture | not-gating: fixture | -
ROWS
}

run_fixture() {  # run_fixture <name> <extra rows> — sets RC, OUT
  { required_rows; printf '%b\n' "$2"; } > "$work/$1.legs"
  OUT=$(bash "$gate" --legs "$work/$1.legs" --root "$root" 2>&1); RC=$?
}

# every CI-only fixture row is printed by name, with its reason
prints_unmeasured() {
  local leg
  for leg in r-gate r-ci-gate r-docs r-feature r-score r-ladder; do
    printf '%s\n' "$OUT" | grep -qE "^  $leg +\[" || return 1
  done
  printf '%s\n' "$OUT" | grep -qF 'NOT MEASURED HERE'
}

# ── arm 3: GREEN ───────────────────────────────────────────────────────────────────────────
f3=$failed
run_fixture green 'cmd | ci / gate | ok-cmd | fixture | - | true\nstep | gate | ok-step | .github/workflows/w.yml#j#passes | - | -'
[ "$RC" -eq 0 ] || fail_arm 3 "passing legs must exit 0, got $RC: $(printf '%s' "$OUT" | tail -5)"
prints_unmeasured || fail_arm 3 "a green run must still print every CI-ONLY row by name"
printf '%s\n' "$OUT" | grep -q 'never a substitute for the required checks' || fail_arm 3 "a green run must say it is a floor"
logdir=$(printf '%s\n' "$OUT" | sed -n 's/^logs: //p')
grep -qs 'step text read from the workflow' "$logdir"/*-ok-step.log || fail_arm 3 "the step leg must run the text read from the workflow"
[ -z "$logdir" ] || rm -rf "${logdir:?}"
[ "$failed" != "$f3" ] || echo "gate-control: arm 3 GREEN       — exit 0, the CI-ONLY rows and the floor sentence printed, the step ran from the workflow"

# ── arm 4: RED ─────────────────────────────────────────────────────────────────────────────
f4=$failed
run_fixture red "cmd | ci / gate | bad-cmd | fixture | - | false\ncmd | gate | after-bad | fixture | - | touch '$work/after-bad-ran'"
[ "$RC" -eq 1 ] || fail_arm 4 "a failing leg must exit 1, got $RC"
printf '%s\n' "$OUT" | grep -qE 'verdict: RED .*bad-cmd' || fail_arm 4 "the verdict must name the failing leg"
prints_unmeasured || fail_arm 4 "a red run must still print every CI-ONLY row by name"
[ -f "$work/after-bad-ran" ] || fail_arm 4 "a failing leg must not stop the legs after it"
logdir=$(printf '%s\n' "$OUT" | sed -n 's/^logs: //p'); [ -z "$logdir" ] || rm -rf "${logdir:?}"
[ "$failed" != "$f4" ] || echo "gate-control: arm 4 RED         — exit $RC, the failing leg named, the CI-ONLY rows printed, the next leg still ran"

# ── arms 5-7: step legs that cannot run are FAIL, never skipped ─────────────────────────────
arm_step() {  # arm_step <arm> <name> <step source> <why>
  run_fixture "step$1" "step | gate | s$1 | $3 | - | -"
  if [ "$RC" -ne 1 ] || ! printf '%s\n' "$OUT" | grep -qE "FAIL +s$1 "; then
    fail_arm "$1" "$4 must be a FAIL (exit 1), got exit $RC"
  else
    echo "gate-control: arm $1 $2 — exit 1, the leg reported FAIL"
  fi
  logdir=$(printf '%s\n' "$OUT" | sed -n 's/^logs: //p'); [ -z "$logdir" ] || rm -rf "${logdir:?}"
}
arm_step 5 "STEP-GONE  " ".github/workflows/w.yml#j#renamed" "a step missing from the workflow"
arm_step 6 "STEP-GITHUB" ".github/workflows/w.yml#j#needs github" "a step that needs GitHub Actions"
arm_step 7 "STEP-RED   " ".github/workflows/w.yml#j#fails" "a step whose script fails"

# ── arms 8-10: tables that must be refused before anything runs ─────────────────────────────
arm_refused() {  # arm_refused <arm> <name> <legs file content> <expected text> <why>
  printf '%b\n' "$3" > "$work/refuse$1.legs"
  OUT=$(bash "$gate" --legs "$work/refuse$1.legs" --root "$root" 2>&1); RC=$?
  if [ "$RC" -ne 2 ] || ! printf '%s\n' "$OUT" | grep -qF "$4" || [ -e "$work/refused-ran-$1" ]; then
    fail_arm "$1" "$5 must be refused (exit 2, '$4') before any leg runs; got exit $RC: $(printf '%s' "$OUT" | tail -3)"
  else
    echo "gate-control: arm $1 $2 — exit 2, refused before any leg ran"
  fi
}
arm_refused 8 "UNACCOUNTED" "$(required_rows | grep -v 'pmat score')\ncmd | gate | x | fixture | - | touch '$work/refused-ran-8'" \
  "required context 'pmat score' has no row" "a table that drops a required context"
arm_refused 9 "NO-REASON  " "$(required_rows)\nci-only | gate | lazy | fixture | slow | -\ncmd | gate | x | fixture | - | touch '$work/refused-ran-9'" \
  "a ci-only row must say why" "a CI-ONLY row with no category and reason"
arm_refused 10 "VACUOUS    " "$(required_rows)" \
  "no row runs anything" "a table in which nothing runs"

# ── arm 11: LIVE — the pinned required set against GitHub ──────────────────────────────────
pinned=$(sed -n '/^REQUIRED_CONTEXTS=(/,/^)/p' "$gate" | sed -n 's/^ *"\(.*\)"$/\1/p' | sort)
repo=$(git -C "$here" remote get-url origin 2>/dev/null | sed -E 's#^.*github\.com[:/]##; s#\.git$##')
if command -v gh >/dev/null 2>&1 && [ -n "$repo" ] \
   && prot=$(gh api "repos/$repo/branches/master/protection/required_status_checks" --jq '.contexts[]' 2>/dev/null) \
   && rules=$(gh api "repos/$repo/rules/branches/master" --jq '.[] | select(.type=="required_status_checks") | .parameters.required_status_checks[].context' 2>/dev/null); then
  live=$(printf '%s\n%s\n' "$prot" "$rules" | grep -v '^$' | sort -u)
  if [ "$live" = "$pinned" ]; then
    echo "gate-control: arm 11 LIVE       — REQUIRED_CONTEXTS equals master's $(printf '%s\n' "$live" | wc -l) required contexts on $repo"
  else
    fail_arm 11 "REQUIRED_CONTEXTS drifted from master's required contexts: $(diff <(printf '%s\n' "$pinned") <(printf '%s\n' "$live") | grep '^[<>]' | tr '\n' ' ')"
  fi
else
  echo "gate-control: arm 11 LIVE       — NOT MEASURED: gh api could not read master's branch protection and rulesets here (no gh, no token, or no network); the pinned set was not compared"
fi

# ── arm 12: FRESH-BINARY — a leg that runs pmat runs the one cargo built from the tree ───────
# A hand-written ./target/debug/pmat ignores CARGO_TARGET_DIR: under an isolated target dir it is
# whatever an earlier build left there, and a leg that ran it judged that binary, not this tree
# (PMAT-1365, sixth session). The fixture plants exactly that — a STALE pmat at ./target/debug/pmat —
# and puts first on PATH a cargo whose every build writes a FRESH pmat into a different target dir
# and reports it as cargo does. The REAL table's legs that run pmat are then run against it: each
# must print FRESH and none may print STALE. Control scripts they call are stubs that exec $1.
f12=$failed
root12="$work/root12" bin12="$work/bin12"
export FAKE_FRESH="$work/target12/debug/pmat"
mkdir -p "$root12/target/debug" "$root12/scripts" "$root12/.github" "$bin12"
cp -R "$here/.github/workflows" "$root12/.github/"
printf '#!/usr/bin/env bash\necho "STALE-PMAT-RAN $*"\n' > "$root12/target/debug/pmat"
cat > "$bin12/cargo" <<'SH'
#!/usr/bin/env bash
mkdir -p "${FAKE_FRESH%/*}"
printf '#!/usr/bin/env bash\necho "FRESH-PMAT-RAN $*"\n' > "$FAKE_FRESH"
chmod +x "$FAKE_FRESH"
case "$1" in
  build) case " $* " in *" --message-format json "*|*" --message-format=json "*)
    printf '{"reason":"compiler-artifact","target":{"name":"pmat","kind":["bin"]},"executable":"%s"}\n' "$FAKE_FRESH" ;; esac ;;
  run) while [ $# -gt 0 ] && [ "$1" != -- ]; do shift; done; shift; exec "$FAKE_FRESH" "$@" ;;
esac
SH
chmod +x "$root12/target/debug/pmat" "$bin12/cargo"
sel=$(python3 - "$gate" "$here" "$root12/scripts" <<'PY2'
import os, re, sys, yaml
gate, here, stubs = sys.argv[1:4]
table = open(gate, encoding="utf-8").read().split("cat <<'LEGS'\n", 1)[1].split("\nLEGS\n", 1)[0]
runs = re.compile(r"target/(debug|release)/pmat|\$\{?PMAT_BIN\b|cargo run\b[^|;&]*--bin pmat\b")
for line in table.splitlines():
    if not line.strip() or line.lstrip().startswith("#"):
        continue
    f = [x.strip() for x in line.split("|", 5)]
    body = f[5] if f[0] == "cmd" else ""
    if f[0] == "step":
        wf, job, name = f[3].split("#", 2)
        doc = yaml.safe_load(open(os.path.join(here, wf), encoding="utf-8")) or {}
        hits = [s for s in ((doc.get("jobs") or {}).get(job) or {}).get("steps") or []
                if isinstance(s, dict) and s.get("name") == name]
        body = (hits[0].get("run") or "") if len(hits) == 1 else ""
    if not runs.search(body):
        continue
    for sc in re.findall(r"\bscripts/[A-Za-z0-9_.-]+\.sh\b", body):
        with open(os.path.join(stubs, os.path.basename(sc)), "w") as fh:
            fh.write('#!/usr/bin/env bash\nexec "$1" stub\n')
    print(line)
PY2
)
names=$(printf '%s\n' "$sel" | awk -F'|' 'NF { gsub(/[[:space:]]/, "", $3); print $3 }')
for must in cb-2113-cb-2115 pmat-score roadmap-validate traceability-control unrun-tests; do
  printf '%s\n' "$names" | grep -qx -- "$must" \
    || fail_arm 12 "the real table's pmat legs were not all found — '$must' is missing (found: $(echo $names))"
done
{ required_rows; printf '%s\n' "$sel"; } > "$work/fresh.legs"
OUT=$(PATH="$bin12:$PATH" GH_TOKEN=fixture-token bash "$gate" --legs "$work/fresh.legs" --root "$root12" 2>&1); RC=$?
logdir=$(printf '%s\n' "$OUT" | sed -n 's/^logs: //p')
stale=() unrun=()
for leg in $names; do
  log=""; for l in "$logdir"/[0-9][0-9]-"$leg".log; do [ -f "$l" ] && log="$l"; done
  if [ -n "$log" ] && grep -q STALE-PMAT-RAN "$log"; then stale+=("$leg"); fi
  if [ -z "$log" ] || ! grep -q FRESH-PMAT-RAN "$log"; then unrun+=("$leg"); fi
done
[ "${#stale[@]}" -eq 0 ] || fail_arm 12 "legs ran the stale pmat planted at ./target/debug/pmat, not the one cargo built from the tree: ${stale[*]}"
[ "${#unrun[@]}" -eq 0 ] || fail_arm 12 "legs never ran the pmat cargo built from the tree: ${unrun[*]}"
[ -z "$logdir" ] || rm -rf "${logdir:?}"
[ "$failed" != "$f12" ] || echo "gate-control: arm 12 FRESH-BINARY — $(echo $names | wc -w) real legs that run pmat ran the binary cargo built, never the stale ./target/debug/pmat"

[ "$failed" = 0 ] || { echo "gate-control: RED" >&2; exit 1; }
echo "gate-control: GREEN — make gate is declared, and every property it promises was seen to break"
