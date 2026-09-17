#!/usr/bin/env bash
# gate.sh — the DECLARED quality gate for this repository (PMAT-1365). `make gate` runs it.
#
# paiml-implement's discovery probes `make -n gate`. This repository had no such target,
# so discovery fell back to `cargo test --workspace` (gate_cmd_fallback=true) — a guess
# weaker than every required status check on master, trusted by whoever read its green.
# This script is the declared answer, and it rests on two rules:
#
#   1. A REQUIRED CONTEXT IS NEVER SILENT. Every status context master requires
#      (REQUIRED_CONTEXTS below) has at least one row in the table: a leg this script
#      runs, or a CI-ONLY row saying why it is not run here. A required context with no
#      row refuses the whole table before anything runs (exit 2).
#   2. WHAT IS NOT MEASURED IS PRINTED ON EVERY EXIT, green or red. A gate that quietly
#      covers less than CI is worse than none, because its green is trusted.
#
# Table rows, `|`-separated, six fields; the last field may itself contain `|`:
#
#   kind    | contexts | leg | source | note | command
#
#   step     runs the `run:` text of <source> = <workflow>#<job id>#<step name>, READ FROM
#            THE WORKFLOW AT RUN TIME, so the leg cannot drift from CI. A step that is gone,
#            renamed, duplicated, or needs GitHub (`${{ }}`, `if:`, `env:`, a non-bash shell) is a FAIL.
#            <note> is how running it here differs from CI, or `-`. <command> is `-`.
#   cmd      runs <command> from the repository root. <source> says where CI runs the
#            check; <note> says how this differs (printed with the verdict). Used where the
#            step cannot be read from this tree (the reusable sovereign-ci.yml lives in
#            paiml/.github) or cannot run verbatim outside Actions.
#   ci-only  runs nothing. <note> is `<category>: <reason>`, category one of platform,
#            credential, cost, trigger, not-gating, not-required. Printed by name on every exit.
#
# <contexts> is `;`-separated. Blank lines and lines starting with `#` are ignored.
#
# $PMAT_BIN — a leg that runs pmat runs the executable `cargo build --message-format json` REPORTS
# building from this tree, never a hand-written ./target/debug/pmat: that path ignores
# CARGO_TARGET_DIR and .cargo/config.toml, and under either one runs whatever binary an earlier
# build left there. cmd rows name "$PMAT_BIN"; in a step's run: text, CI's ./target/debug/pmat is
# rewritten to it. It is resolved the first time a leg needs it, and a leg that runs pmat when
# cargo reports no pmat is a FAIL.
#
# Usage: scripts/gate.sh                       run the gate
#        scripts/gate.sh --list                check the table and print it; run nothing
#        scripts/gate.sh --legs FILE --root DIR   a fixture table and tree (scripts/gate-control.sh)
#
# Exit: 0 every leg run here passed · 1 a leg failed, or (--list) a step source does not
#       resolve · 2 the table is refused: a required context has no row, a row is malformed,
#       a CI-ONLY row gives no reason, or nothing in it runs.
set -uo pipefail

REQUIRED_CONTEXTS=(
  "gate"
  "ci / gate"
  "docs build (docs.rs environment)"
  "feature-gate"
  "pmat score"
  "provable ladder"
)
EXTENSION_MARKER="── EXTENSION POINT"
PMAT_SPELLING="./target/debug/pmat"   # CI's path to the pmat it built; rewritten to $PMAT_BIN here

legs_table() {
  cat <<'LEGS'
# ── make gate — this gate's own falsifier runs first: the gate proves it can fail before its green is read.
cmd     | make gate | gate-control | scripts/gate-control.sh | - | bash scripts/gate-control.sh

# ── ci / gate — paiml/.github sovereign-ci.yml@main (jobs test, lint, coverage, security, provenance),
#    reached through ci.yml job `ci`. That file is not in this tree, so these rows are `cmd`.
cmd     | ci / gate | fmt | sovereign-ci.yml lint "Format check" | - | cargo fmt --all -- --check
# clippy: sovereign-ci.yml lint runs `cargo clippy --all-targets -- -D warnings -A unused-variables`;
# feature-matrix.yml all-targets runs the same without `-A unused-variables`. The stricter one covers both.
step    | ci / gate;feature-gate | clippy-all-targets | .github/workflows/feature-matrix.yml#all-targets#clippy --all-targets (compiles benches and examples) | - | -
# lib-tests: `cargo test --lib` takes ~20 min on this machine and nextest under 5 (ci.yml PMAT-1313 comment).
# The `gate` profile in .config/nextest.toml kills a test only after 10 min, as CI's cargo test kills none.
cmd     | ci / gate | lib-tests | sovereign-ci.yml test "Run tests" | CI runs `cargo test --lib`; this runs nextest (profile gate), one process per test, without the 2 tests .config/nextest.toml excludes because they hang under nextest (PMAT-1314) | cargo nextest run --lib --locked --no-fail-fast --profile gate
# cargo deny: sovereign-ci.yml lint runs `cargo deny check advisories licenses sources`, a subset of this step.
step    | pmat score;ci / gate | cargo-deny | .github/workflows/quality-gate.yml#score#Supply chain gate — cargo deny (blocking, all families) | - | -
cmd     | ci / gate | cargo-audit | sovereign-ci.yml security "Audit" | same .cargo/audit.toml ignores; no CI git credential header | flags=(); if [ -f .cargo/audit.toml ]; then while read -r id; do flags+=(--ignore "$id"); done < <(sed -n 's/.*\(RUSTSEC-[0-9]*-[0-9]*\).*/\1/p' .cargo/audit.toml); fi; cargo audit "${flags[@]}"
ci-only | ci / gate | coverage | sovereign-ci.yml coverage | cost: an instrumented rebuild and a serial run of the whole lib suite (~29 min in CI) against max(coverage_min, .coverage-baseline.txt) | -
ci-only | ci / gate | provenance | sovereign-ci.yml provenance | credential: SLSA attestation needs the GitHub Actions OIDC token | -
ci-only | ci / gate | roadmap-fragment-parity | sovereign-ci.yml security "roadmap-fragment-parity" | trigger: pull_request only, diffed against the PR's base sha; a no-op while docs/roadmaps/entries/ does not exist | -

# ── gate — ci.yml top-level job; the org ruleset requires it. Its needs are ci (above) plus:
step    | gate | reusable-pin-drift | .github/workflows/ci.yml#reusable-pin-drift#every reusable workflow is referenced at exactly one ref | - | -
step    | gate | build-pmat | .github/workflows/ci.yml#roadmap-validate#build pmat from this tree | - | -
step    | gate | roadmap-validate-control | .github/workflows/ci.yml#roadmap-validate#control — a duplicated id must be refused (exit 1) | - | -
step    | gate | roadmap-validate | .github/workflows/ci.yml#roadmap-validate#pmat work validate (0 = valid, 1 = invalid or unreadable) | - | -
step    | gate | traceability-control | .github/workflows/ci.yml#traceability#control — a commit with no Pmat-Ticket trailer must be refused (exit 1) | - | -
step    | gate | roadmap-coherence-control | .github/workflows/ci.yml#traceability#control — an open item naming a closed issue must be refused (CB-2115, exit 1) | - | -
step    | gate | work-sync-control | .github/workflows/ci.yml#traceability#control — pmat work sync --check-only can fail, can pass, and plans every write | - | -
step    | gate | ticket-release-control | .github/workflows/ci.yml#traceability#control — an open item with no issue, and one with no release:, must be refused (CB-2112, CB-2114, exit 1) | - | -
step    | gate | spec-epic-control | .github/workflows/ci.yml#traceability#control — an active spec with no epic must be refused (CB-2110, exit 1) | - | -
step    | gate | spec-review-control | .github/workflows/ci.yml#traceability#control — an active spec with no current review must be refused (CB-2111, exit 1) | - | -
step    | gate | pr-lane-control | .github/workflows/ci.yml#traceability#PR lane control | - | -
step    | gate | tests-dont-write-self-test | .github/workflows/ci.yml#traceability#the tree-cleanliness checker can still detect a write | - | -
# CB-2115 reads open GitHub issues. It was CI-only on "credential: the workflow's GH_TOKEN" until traceability went
# red on PR #1368 (an issue opened with no roadmap row) while `make gate` read green; any clone that can push has a
# gh token, and the check takes ~7s. No token is a FAIL here, never a skip: an unread GitHub is not an agreeing one.
cmd     | gate | cb-2113-cb-2115 | ci.yml traceability "the closed loop holds (CB-2113) and the roadmap and GitHub agree (CB-2115)" | CI sets GH_TOKEN to the workflow token; this uses $GH_TOKEN, else `gh auth token`, and fails without either | token="${GH_TOKEN:-$(gh auth token)}"; [ -n "$token" ]; GH_TOKEN="$token" "$PMAT_BIN" comply check --checks CB-2113,CB-2115
ci-only | gate | tests-dont-write | ci.yml traceability "the test suite does not write to the repository (pre-release lane)" | trigger: runs on push to master only (the pre-release lane), never on a pull request | -
ci-only | gate | windows-check | ci.yml windows-check | platform: cargo check --bin pmat on windows-latest | -

# ── docs build (docs.rs environment) — docsrs.yml job build
ci-only | docs build (docs.rs environment) | docs-rs-build | .github/workflows/docsrs.yml build | cost: `cargo +nightly doc` with the docs.rs feature set compiles every dependency again under a second toolchain | -

# ── feature-gate — feature-matrix.yml job feature-gate requires every leg below
step    | feature-gate | orphan-ledger | .github/workflows/feature-matrix.yml#orphan-ledger#every orphan feature is tested or explained | - | -
step    | feature-gate | dependabot-self-test | .github/workflows/feature-matrix.yml#dependabot-alerts#the gate proves it can fail | - | -
# dependabot-alerts-live was CI-only on "credential: the DEPENDABOT_TOKEN secret"; with the gh token it runs in <1s.
cmd     | feature-gate | dependabot-alerts-live | feature-matrix.yml dependabot-alerts "no open Dependabot alerts at or above medium" | CI reads DEPENDABOT_TOKEN and SKIPS this arm when that secret is unset; this uses $GH_TOKEN, else `gh auth token`, and fails without either or on a 403 | token="${GH_TOKEN:-$(gh auth token)}"; [ -n "$token" ]; GH_TOKEN="$token" ./scripts/dependabot-alerts-gate.sh
# unrun-tests and reachability-ledger: each CI job only builds pmat and runs one subcommand. They were CI-only
# on "cost: a release build" until reachability-ledger went red in CI on this branch's own new file while
# `make gate` read green; on the debug build build-pmat already made they take ~14s and ~2s. `cargo run`, never
# a ./target path, for the reason feature-matrix.yml gives: a redirected target dir measures a stale binary.
cmd     | feature-gate | unrun-tests | feature-matrix.yml unrun-tests "every test is executed by some leg, or the ledger says why" | CI runs the release build; this runs the debug build | cargo run --locked --quiet --bin pmat -- analyze unrun-tests --executed '' --check-ledger
cmd     | feature-gate | reachability-ledger | feature-matrix.yml reachability-ledger "the committed orphan-files ledger matches the tree" | CI runs the release build; this runs the debug build | cargo run --locked --quiet --bin pmat -- analyze reachability --check-ledger
ci-only | feature-gate | bundles | feature-matrix.yml bundles | cost: cargo check --lib --tests and --bin once per feature bundle | -
ci-only | feature-gate | individual | feature-matrix.yml individual | cost: cargo check of every feature in Cargo.toml, one at a time, in 6 shards | -
ci-only | feature-gate | feature-tests | feature-matrix.yml feature-tests | cost: clippy and the whole lib suite rebuilt once per feature set | -
ci-only | feature-gate | package-size | feature-matrix.yml package-size | cost: cargo package verifies the tarball with a from-scratch build | -
ci-only | feature-gate | binary-size | feature-matrix.yml binary-size | cost: a release build of pmat | -
ci-only | feature-gate | differential-corpus | feature-matrix.yml differential-corpus | cost: make gate-differential builds and sweeps generated corpora | -
ci-only | feature-gate | flag-efficacy | feature-matrix.yml flag-efficacy | cost: make gate-flag-efficacy exercises every CLI flag | -
ci-only | feature-gate | cli-doc-sync-falsifier | feature-matrix.yml cli-doc-sync-falsifier | cost: three release-mode runs of tests/all | -

# ── pmat score — quality-gate.yml job score (cargo-deny, above, is its first step)
cmd     | pmat score | pmat-score | quality-gate.yml score "Run unified quality gate" | CI runs a release `cargo install --path .`; this runs the tree's debug build and writes score.json to the log directory, not the tree | "$PMAT_BIN" score --gate 60 --format json -o "$GATE_LOGDIR/score.json"

# ── provable ladder — quality-gate.yml job provable-ladder
cmd     | provable ladder | lean-build | quality-gate.yml provable-ladder "L5 — build Lean proofs (lake build)" | CI uses lean-action; this runs lake build on a scratch copy of contracts/lean so .lake/ never lands in the tree | work=$(mktemp -d) && cp -R contracts/lean/. "$work" && cd "$work" && lake build
step    | provable ladder | lean-no-holes | .github/workflows/quality-gate.yml#provable-ladder#L5 — assert zero proof holes (no sorry / admit) | - | -
step    | provable ladder | pv-obligations | .github/workflows/quality-gate.yml#provable-ladder#L2/L3 — every contract validates and its obligations are visible to pv | CI installs pv from aprender-contracts-cli at the version its "Install pv" step pins; this uses the pv on PATH | -
ci-only | provable ladder | comply-ladder | quality-gate.yml provable-ladder "Ladder gate — pmat comply" | not-gating: CI runs it with continue-on-error, so it gates nothing there either | -

# ── not a required context, named so nobody assumes it ran
ci-only | mutation-diff (not required) | mutation-diff | .github/workflows/mutation-diff.yml | not-required: runs nightly and on pull requests labelled `mutation` | -

# ── EXTENSION POINT ─────────────────────────────────────────────────────────────────────────
# Sibling gates append their rows BELOW this marker, in their own pull requests:
#   D0 — issue-closure contract
#   D2 — roadmap-write query gate
# A new row needs no other edit: the runner, the CI-ONLY printout and the verdict pick it up.
# Keep this marker; scripts/gate-control.sh and src/make_gate_tests.rs assert it is here.
# ── gate — ci.yml tdg-ratchet (PMAT-636): CB-200's `[tdg] baseline`, measured where a merge is decided.
step    | gate | tdg-ratchet-control | .github/workflows/ci.yml#tdg-ratchet#control — every CB-200 verdict is reachable, and a planted below-A definition is refused | - | -
step    | gate | tdg-ratchet | .github/workflows/ci.yml#tdg-ratchet#CB-200 is measured and equals the recorded baseline | CI measures a fresh checkout over pmat's own out-of-tree index; in a clone that holds .pmat/context.db CB-200 counts that index instead, and a stale one is a FAIL (STALE) | -

# ── D2 — PMAT-1385: no raw write lands under docs/roadmaps/ without the repository lock. The gate is a
#    --lib suite (a taint analysis of every compiled file, its planted mutants, and the migrate tests), so
#    CI's `cargo test --lib` already runs it; this leg runs those 15 tests by name and refuses a filter
#    that matched none, which the lib-tests leg above would not notice.
cmd     | ci / gate | roadmap-writer-gate | sovereign-ci.yml test "Run tests" (roadmap_writer_gate_* and work_migrate_*) | CI runs these inside the whole lib suite; this runs only them, and fails when a required test did not run | bash scripts/roadmap-writer-gate.sh

# D0 — PMAT-900001 (contracts/pmat-issue-closure-v1.yaml): the call-site gate, run as CI's own step.
# Its PR-body sibling step needs `${{ }}` and a pull_request event, so it cannot be a leg here.
step    | gate | issue-closure | .github/workflows/ci.yml#traceability#control — no pmat code path can close a GitHub issue (PMAT-900001) | - | -
LEGS
}

# ── arguments ───────────────────────────────────────────────────────────────────────────────
MODE=run LEGS_FILE="" ROOT=""
while [ $# -gt 0 ]; do
  case "$1" in
    --list) MODE=list; shift ;;
    --legs) LEGS_FILE="${2:-}"; shift 2 ;;
    --root) ROOT="${2:-}"; shift 2 ;;
    -h|--help) sed -n '2,/^set -uo pipefail$/p' "$0" | sed '$d'; exit 0 ;;
    *) echo "gate.sh: unknown argument '$1'" >&2; exit 2 ;;
  esac
done
if [ -n "$LEGS_FILE" ] && [ -z "$ROOT" ]; then echo "gate.sh: --legs needs --root" >&2; exit 2; fi
if [ -z "$ROOT" ]; then ROOT="$(cd "$(dirname "$0")/.." && pwd)"; fi
ROOT="$(cd "$ROOT" && pwd)" || { echo "gate.sh: --root is not a directory" >&2; exit 2; }

shopt -u patsub_replacement 2>/dev/null || true   # bash 5.2: `&` in a ${x//p/r} replacement means the match

trim() { local s="$1"; s="${s#"${s%%[![:space:]]*}"}"; s="${s%"${s##*[![:space:]]}"}"; printf '%s' "$s"; }

refuse() { echo "gate.sh: TABLE REFUSED — $*" >&2; exit 2; }

# ── load and validate the table ────────────────────────────────────────────────────────────
KIND=() CTX=() LEG=() SRC=() NOTE=() CMD=()
if [ -n "$LEGS_FILE" ]; then
  [ -r "$LEGS_FILE" ] || refuse "cannot read --legs $LEGS_FILE"
  table="$(< "$LEGS_FILE")"
else
  table="$(legs_table)"
fi

n=0 runnable=0
while IFS= read -r line; do
  case "$(trim "$line")" in ''|'#'*) continue ;; esac
  n=$((n + 1))
  IFS='|' read -r k c l s o m <<< "$line"
  k="$(trim "$k")" c="$(trim "$c")" l="$(trim "$l")" s="$(trim "$s")" o="$(trim "${o:-}")" m="$(trim "${m:-}")"
  [ -n "$c" ] && [ -n "$l" ] && [ -n "$s" ] && [ -n "$o" ] && [ -n "$m" ] \
    || refuse "row $n has an empty field (want kind | contexts | leg | source | note | command): $line"
  case "$l" in *[!A-Za-z0-9_.-]*) refuse "row $n: leg id '$l' must be [A-Za-z0-9_.-]" ;; esac
  case "$k" in
    step)
      [ "$m" = "-" ] || refuse "row $n ($l): a step row runs its workflow step; its command field must be '-'"
      case "$s" in *'#'*'#'*) ;; *) refuse "row $n ($l): step source must be <workflow>#<job id>#<step name>" ;; esac
      runnable=$((runnable + 1)) ;;
    cmd)
      [ "$m" != "-" ] || refuse "row $n ($l): a cmd row needs a command"
      runnable=$((runnable + 1)) ;;
    ci-only)
      [ "$m" = "-" ] || refuse "row $n ($l): a ci-only row runs nothing; its command field must be '-'"
      case "$o" in
        platform:?*|credential:?*|cost:?*|trigger:?*|not-gating:?*|not-required:?*) ;;
        *) refuse "row $n ($l): a ci-only row must say why — note must be '<platform|credential|cost|trigger|not-gating|not-required>: <reason>', got '$o'" ;;
      esac
      [ -n "$(trim "${o#*:}")" ] || refuse "row $n ($l): ci-only reason is empty" ;;
    *) refuse "row $n: kind '$k' is not step, cmd or ci-only" ;;
  esac
  KIND+=("$k") CTX+=("$c") LEG+=("$l") SRC+=("$s") NOTE+=("$o") CMD+=("$m")
done <<< "$table"

[ "$n" -gt 0 ] || refuse "the table has no rows"
[ "$runnable" -gt 0 ] || refuse "no row runs anything — a gate that measures nothing is vacuous, not green"
for req in "${REQUIRED_CONTEXTS[@]}"; do
  found=0
  for i in "${!CTX[@]}"; do
    IFS=';' read -ra parts <<< "${CTX[$i]}"
    for p in "${parts[@]}"; do [ "$(trim "$p")" = "$req" ] && found=1; done
  done
  [ "$found" = 1 ] || refuse "required context '$req' has no row — it would be neither run nor named"
done
if [ -z "$LEGS_FILE" ]; then
  legs_table | grep -qF "$EXTENSION_MARKER" || refuse "the extension-point marker is missing"
fi

# step_script <workflow> <job> <step name> — print the step's run: text; non-zero with a reason.
step_script() {
  python3 - "$ROOT" "$1" "$2" "$3" <<'PY'
import sys
try:
    import yaml
except ImportError:
    sys.exit("python3 has no yaml module (PyYAML) — cannot read the workflow, so the step cannot run")
root, wf, job, name = sys.argv[1:5]
try:
    with open(f"{root}/{wf}", encoding="utf-8") as fh:
        doc = yaml.safe_load(fh) or {}
except OSError as e:
    sys.exit(f"cannot read {wf}: {e}")
j = (doc.get("jobs") or {}).get(job)
if not isinstance(j, dict):
    sys.exit(f"{wf} has no job '{job}'")
hits = [s for s in (j.get("steps") or []) if isinstance(s, dict) and s.get("name") == name]
if len(hits) != 1:
    sys.exit(f"{wf} job '{job}' has {len(hits)} steps named '{name}' (want exactly 1) — renamed or removed in CI?")
s = hits[0]
run = s.get("run")
if not isinstance(run, str) or not run.strip():
    sys.exit(f"{wf} job '{job}' step '{name}' has no run: script")
blockers = [k for k in ("if", "env", "working-directory") if k in s]
shell = s.get("shell")
if shell not in (None, "bash"):
    blockers.append(f"shell: {shell}")
if "${{" in run:
    blockers.append("${{ }} in run")
if (j.get("defaults") or {}).get("run"):
    blockers.append("job defaults.run")
if blockers:
    sys.exit(f"{wf} job '{job}' step '{name}' needs GitHub Actions ({', '.join(blockers)}) — declare it cmd or ci-only instead")
# GitHub runs an explicit `shell: bash` as `bash --noprofile --norc -eo pipefail {0}`, and a
# step with no shell as `bash -e {0}`. The first line tells the runner which one to use.
sys.stdout.write(("#shell=bash" if shell == "bash" else "#shell=default") + "\n" + run)
PY
}

# ── --list ─────────────────────────────────────────────────────────────────────────────────
if [ "$MODE" = list ]; then
  bad=0
  for i in "${!KIND[@]}"; do
    printf '%-8s %-26s [%s] %s\n' "${KIND[$i]}" "${LEG[$i]}" "${CTX[$i]}" "${SRC[$i]}"
    if [ "${KIND[$i]}" = step ]; then
      IFS='#' read -r wf job step <<< "${SRC[$i]}"
      if ! err=$(step_script "$wf" "$job" "$step" 2>&1 >/dev/null); then
        echo "  UNRESOLVED: $err"; bad=1
      fi
    fi
  done
  echo "gate.sh --list: ${#KIND[@]} rows, $runnable run here, $(( ${#KIND[@]} - runnable )) CI-only; every required context has a row"
  [ "$bad" = 0 ] || { echo "gate.sh --list: a step source does not resolve (above)"; exit 1; }
  exit 0
fi

# ── run ────────────────────────────────────────────────────────────────────────────────────
GATE_LOGDIR="$(mktemp -d "${TMPDIR:-/tmp}/pmat-gate.XXXXXX")"
export GATE_LOGDIR
RESULT=() SECS=()
PMAT_BIN="" PMAT_BIN_RC=""

# resolve_pmat_bin — set $PMAT_BIN to the pmat executable cargo reports building from $ROOT.
# Once per run: a failed resolution is remembered, so every later pmat leg fails without a rebuild.
resolve_pmat_bin() {
  if [ -z "$PMAT_BIN_RC" ]; then
    local pick
    pick=$(cat <<'PY'
import json, sys
exe = []
for line in sys.stdin:
    try:
        m = json.loads(line)
    except ValueError:
        continue
    t = m.get("target") or {}
    if m.get("reason") == "compiler-artifact" and t.get("name") == "pmat" and "bin" in (t.get("kind") or []) and m.get("executable"):
        exe.append(m["executable"])
if len(exe) != 1:
    sys.exit(f"cargo reported {len(exe)} pmat executables, want exactly 1")
print(exe[0])
PY
)
    PMAT_BIN=$( (cd "$ROOT" && cargo build --locked --bin pmat --message-format json) 2> "$GATE_LOGDIR/pmat-bin.log" \
      | python3 -c "$pick" 2>> "$GATE_LOGDIR/pmat-bin.log")
    PMAT_BIN_RC=$?
    if [ "$PMAT_BIN_RC" -eq 0 ] && [ ! -x "$PMAT_BIN" ]; then
      echo "cargo reported '$PMAT_BIN', which is not an executable" >> "$GATE_LOGDIR/pmat-bin.log"
      PMAT_BIN_RC=1
    fi
    [ "$PMAT_BIN_RC" -eq 0 ] || PMAT_BIN=""
    export PMAT_BIN
  fi
  return "$PMAT_BIN_RC"
}

# needs_pmat_bin <leg text> <log> — resolve $PMAT_BIN when the text runs pmat, and name that
# binary in the log. Non-zero when the leg runs pmat and cargo reported none.
needs_pmat_bin() {
  case "$1" in *"$PMAT_SPELLING"*|*PMAT_BIN*) ;; *) return 0 ;; esac
  if ! resolve_pmat_bin; then
    { echo "gate.sh: this leg runs pmat, and cargo build --message-format json reported no pmat built from $ROOT:"
      tail -n 25 "$GATE_LOGDIR/pmat-bin.log"; } >> "$2"
    return 1
  fi
  echo "gate.sh: pmat = $PMAT_BIN (reported by cargo build --message-format json)" >> "$2"
}

print_unmeasured() {
  echo ""
  echo "NOT MEASURED HERE — CI-only, by name (a local green says nothing about these):"
  for i in "${!KIND[@]}"; do
    [ "${KIND[$i]}" = ci-only ] || continue
    printf '  %-26s [%s] %s\n' "${LEG[$i]}" "${CTX[$i]}" "${NOTE[$i]}"
  done
  echo "RUN HERE, BUT NOT AS CI RUNS IT:"
  for i in "${!KIND[@]}"; do
    [ "${KIND[$i]}" != ci-only ] && [ "${NOTE[$i]}" != "-" ] || continue
    printf '  %-26s %s\n' "${LEG[$i]}" "${NOTE[$i]}"
  done
  [ -z "$PMAT_BIN" ] || echo "  every leg that ran pmat ran $PMAT_BIN, the executable cargo reported; a step's $PMAT_SPELLING was rewritten to it"
}

finish() {
  local rc=$? failed=() i
  trap - EXIT
  echo ""
  echo "make gate — legs run here:"
  for i in "${!KIND[@]}"; do
    [ "${KIND[$i]}" != ci-only ] || continue
    printf '  %-6s %-26s %5ss  [%s]\n' "${RESULT[$i]:-NOTRUN}" "${LEG[$i]}" "${SECS[$i]:--}" "${CTX[$i]}"
    [ "${RESULT[$i]:-NOTRUN}" = PASS ] || failed+=("${LEG[$i]}")
  done
  print_unmeasured
  echo ""
  echo "logs: $GATE_LOGDIR"
  if [ "${#failed[@]}" -eq 0 ] && [ "$rc" -eq 0 ]; then
    echo "verdict: GREEN — the local floor, never a substitute for the required checks: $(IFS=,; echo "${REQUIRED_CONTEXTS[*]}" | sed 's/,/, /g')."
    exit 0
  fi
  echo "verdict: RED — ${#failed[@]} leg(s) did not pass: ${failed[*]:-interrupted}"
  exit 1
}
trap finish EXIT
[ -z "$LEGS_FILE" ] || echo "gate.sh: FIXTURE TABLE $LEGS_FILE — this is not the repository gate"

for i in "${!KIND[@]}"; do
  [ "${KIND[$i]}" != ci-only ] || continue
  leg="${LEG[$i]}" log="$GATE_LOGDIR/$(printf '%02d' "$i")-${LEG[$i]}.log"
  echo "── ${leg} [${CTX[$i]}]"
  start=$SECONDS
  if [ "${KIND[$i]}" = step ]; then
    IFS='#' read -r wf job step <<< "${SRC[$i]}"
    if step_script "$wf" "$job" "$step" > "$log.sh" 2> "$log" && needs_pmat_bin "$(< "$log.sh")" "$log"; then
      if [ -n "$PMAT_BIN" ] && grep -qF "$PMAT_SPELLING" "$log.sh"; then
        text="$(< "$log.sh")"
        printf '%s\n' "${text//"$PMAT_SPELLING"/$(printf '%q' "$PMAT_BIN")}" > "$log.sh"
      fi
      flags=(-e); [ "$(head -n 1 "$log.sh")" != "#shell=bash" ] || flags=(-eo pipefail)
      (cd "$ROOT" && bash --noprofile --norc "${flags[@]}" "$log.sh") >> "$log" 2>&1 < /dev/null
      rc=$?
    else
      rc=$?
    fi
  else
    : > "$log"
    if needs_pmat_bin "${CMD[$i]}" "$log"; then
      (cd "$ROOT" && bash --noprofile --norc -eo pipefail -c "${CMD[$i]}") >> "$log" 2>&1 < /dev/null
      rc=$?
    else
      rc=1
    fi
  fi
  SECS[$i]=$((SECONDS - start))
  if [ "$rc" -eq 0 ]; then
    RESULT[$i]=PASS
  else
    RESULT[$i]=FAIL
    echo "   FAIL rc=$rc — last lines of $log:"
    tail -n 25 "$log" | sed 's/^/   | /'
  fi
done
