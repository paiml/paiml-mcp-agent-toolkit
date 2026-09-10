#!/usr/bin/env bash
# traceability-control.sh — prove CB-2113 can fail before trusting it
# (goal-mode.md §7.1 "CONTROL FIRST"; PMAT-719).
#
#   scripts/traceability-control.sh <path-to-pmat-binary>
#
# A gate that cannot fail is theater. This script plants the defect CB-2113
# exists to catch in throwaway git repositories and asserts the verdict on
# every arm — the RED arms AND the GREEN one, because a rule that always fails
# would pass the RED arms for the wrong reason:
#
#   arm 1  RED   feature commit with no Pmat-Ticket trailer        exit 1, Fail, names the commit
#   arm 2  GREEN the same commit trailed `Pmat-Ticket: PMAT-001`     exit 0, Pass
#   arm 3  RED   trailer naming PMAT-999, absent from the roadmap    exit 1, Fail, names PMAT-999
#   arm 4  RED   trailer naming PMAT-002, a completed item           exit 1, Fail, names PMAT-002
#   arm 5  N/A   HEAD on master                                      exit 0, Pass, message says not_applicable
#
# Each arm asserts BOTH the process exit code and the CB-2113 entry in the JSON
# report: the exit code is what CI acts on, the entry is what proves the verdict
# came from this rule and not from another one. An entry that is missing is
# under-discovery and fails the control (exit 2) — "we found no CB-2113 row"
# must never render as "CB-2113 passed".
#
# Exit: 0 every arm behaved · 1 an arm did not (named on stderr) · 2 usage,
# missing binary, or a report the script could not read.
#
# The binary is an ARGUMENT, never resolved from PATH: the control must judge
# the pmat built from this tree, not whichever one is installed.
set -uo pipefail

PMAT="${1:-}"
if [ -z "$PMAT" ] || [ ! -x "$PMAT" ]; then
  echo "traceability-control: usage: $0 <path-to-pmat-binary> (got '${PMAT}')" >&2
  exit 2
fi
command -v jq >/dev/null 2>&1 || { echo "traceability-control: jq is required" >&2; exit 2; }
command -v git >/dev/null 2>&1 || { echo "traceability-control: git is required" >&2; exit 2; }

work="$(mktemp -d "${TMPDIR:-/tmp}/traceability-control.XXXXXX")"
trap 'rm -rf "${work:?}"' EXIT
repo="$work/repo"
report="$work/report.json"

# Fixture git runs with NO host configuration: a global hooksPath or
# commit.gpgsign on the operator's machine must not reach these commits, and a
# pipeline's GITHUB_BASE_REF must not decide which arm is on which branch.
fgit() {
  GIT_CONFIG_GLOBAL=/dev/null GIT_CONFIG_SYSTEM=/dev/null \
  GIT_AUTHOR_NAME=cb2113 GIT_AUTHOR_EMAIL=cb2113@example.invalid \
  GIT_COMMITTER_NAME=cb2113 GIT_COMMITTER_EMAIL=cb2113@example.invalid \
    git -C "$repo" -c commit.gpgsign=false "$@"
}

# One run of the gate against the fixture. Sets RC (process exit), STATUS and
# MESSAGE (the CB-2113 entry). A report with no CB-2113 entry is exit 2.
run_gate() {
  RC=0
  env -u GITHUB_BASE_REF "$PMAT" comply check --checks CB-2113 --path "$repo" --format json \
    >"$report" 2>"$work/stderr" || RC=$?
  local entry
  entry=$(jq -c '[.checks[] | select(.name | startswith("CB-2113"))] | first // empty' "$report" 2>/dev/null || true)
  if [ -z "$entry" ]; then
    echo "traceability-control: the report carries no CB-2113 entry (exit $RC) — under-discovery is a finding, not a pass" >&2
    echo "--- stderr" >&2; head -20 "$work/stderr" >&2
    echo "--- report" >&2; head -c 2000 "$report" >&2; echo >&2
    exit 2
  fi
  STATUS=$(printf '%s' "$entry" | jq -r '.status')
  MESSAGE=$(printf '%s' "$entry" | jq -r '.message')
}

fail_arm() {
  echo "traceability-control: ARM $1 FAILED — $2" >&2
  echo "  exit=$RC status=$STATUS" >&2
  echo "  message=$MESSAGE" >&2
  exit 1
}

# ── fixture ──────────────────────────────────────────────────────────────────
mkdir -p "$repo/docs/roadmaps"
fgit init -q -b master
cat >"$repo/docs/roadmaps/roadmap.yaml" <<'YAML'
roadmap_version: "1.0"
github_enabled: false
github_repo: null
roadmap:
  - id: PMAT-001
    title: planned work
    status: planned
  - id: PMAT-002
    title: finished work
    status: completed
YAML
fgit add .
fgit commit -q -m "roadmap"
fgit tag v0.1.0
fgit switch -q -c feature

fingerprint="$(cd "$repo" && sha256sum docs/roadmaps/roadmap.yaml | cut -c1-12) roadmap=2 items, tag v0.1.0, branch feature"
echo "traceability-control: fixture $fingerprint; pmat=$("$PMAT" --version 2>/dev/null | head -1)"

# ── arm 1: RED — no trailer ──────────────────────────────────────────────────
fgit commit -q --allow-empty -m "feat: no trailer"
bad=$(fgit rev-parse --short=7 HEAD)
run_gate
[ "$RC" -eq 1 ] || fail_arm 1 "an untrailered commit must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 1 "CB-2113 must be Fail on an untrailered commit"
case "$MESSAGE" in *"$bad"*"no Pmat-Ticket"*) ;; *) fail_arm 1 "the message must name commit $bad and the missing trailer";; esac
echo "traceability-control: arm 1 RED   — untrailered commit $bad refused (exit 1)"

# ── arm 2: GREEN — trailer names a planned item ─────────────────────────────
fgit commit -q --amend --allow-empty -m "feat: trailed" -m "Pmat-Ticket: PMAT-001"
run_gate
[ "$RC" -eq 0 ] || fail_arm 2 "a trailered commit naming a planned item must exit 0"
[ "$STATUS" = "Pass" ] || fail_arm 2 "CB-2113 must be Pass"
case "$MESSAGE" in *"carry a Pmat-Ticket"*) ;; *) fail_arm 2 "the pass message must say the trailer was found";; esac
echo "traceability-control: arm 2 GREEN — trailed commit accepted (exit 0)"

# ── arm 3: RED — trailer names an id the roadmap does not have ──────────────
fgit commit -q --amend --allow-empty -m "feat: unknown" -m "Pmat-Ticket: PMAT-999"
run_gate
[ "$RC" -eq 1 ] || fail_arm 3 "an unknown ticket id must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 3 "CB-2113 must be Fail on an unknown id"
case "$MESSAGE" in *"PMAT-999"*"not in docs/roadmaps/roadmap.yaml"*) ;; *) fail_arm 3 "the message must name PMAT-999 as absent";; esac
echo "traceability-control: arm 3 RED   — PMAT-999 (not in the roadmap) refused (exit 1)"

# ── arm 4: RED — trailer names a completed item ─────────────────────────────
fgit commit -q --amend --allow-empty -m "feat: terminal" -m "Pmat-Ticket: PMAT-002"
run_gate
[ "$RC" -eq 1 ] || fail_arm 4 "a completed ticket must exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 4 "CB-2113 must be Fail on a terminal item"
case "$MESSAGE" in *"PMAT-002"*"completed"*) ;; *) fail_arm 4 "the message must name PMAT-002 as completed";; esac
echo "traceability-control: arm 4 RED   — PMAT-002 (completed) refused (exit 1)"

# ── arm 5: not applicable — HEAD on the default branch ──────────────────────
fgit switch -q master
run_gate
[ "$RC" -eq 0 ] || fail_arm 5 "the default branch must exit 0 (§8.4: counted, not judged)"
[ "$STATUS" = "Pass" ] || fail_arm 5 "CB-2113 must be Pass with a stated reason on the default branch"
case "$MESSAGE" in "not_applicable:"*) ;; *) fail_arm 5 "the message must start with not_applicable and name the reason";; esac
echo "traceability-control: arm 5 N/A   — default branch reported not_applicable (exit 0)"

echo "traceability-control: all 5 arms behaved — CB-2113 can fail, and can pass"
