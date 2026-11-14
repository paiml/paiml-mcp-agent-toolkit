#!/usr/bin/env bash
# mutation-lab-control.sh — the nightly mutation lane is a LAB check, and this
# proves the file says so (fix-set for #1689, infra#3828).
#
# Operator rules this pins:
#   "LAB checks never run inside a PR, merge-queue or release job. They run at
#    night on idle machines. ... A red opens or updates one ticket with one
#    owner, then stays silent. Red or unmeasured for 7 days, it stops running
#    until its owner brings it back."
#   "NO build can do mutation outside of nightly; the end. FOREVER."
#
#   arm 1 GREEN  triggers are schedule (+ optional workflow_dispatch) only
#   arm 2 GREEN  top-level permissions grant issues: write
#   arm 3 GREEN  the 7-day stop exists and gates the mutation job
#   arm 4 GREEN  the ticket job is guarded by failure(), labelled, owned, deduped
#   arm 5 GREEN  no continue-on-error anywhere in the lane (verdict step included)
#   arms 6-10 RED  a fixture carrying each defect is refused by the same predicate
#
# The RED arms are the point: without them this asserts that a file contains
# what somebody just wrote into it. Each plants one defect into a copy of the
# real workflow and requires the predicate to refuse it.
#
# Usage: bash scripts/mutation-lab-control.sh            # the committed workflow
#        MUTATION_WF=path bash scripts/mutation-lab-control.sh  # judge another copy
#        (with MUTATION_WF set, only the GREEN arms run, against that file)
set -uo pipefail
cd "$(dirname "$0")/.." || exit 9
FAIL=0
fail_arm() { echo "mutation-lab-control: ARM $1 FAILED — $2"; FAIL=1; }
pass_arm() { echo "mutation-lab-control: arm $1 ok — $2"; }

WF="${MUTATION_WF:-.github/workflows/mutation-diff.yml}"
[ -r "$WF" ] || { echo "mutation-lab-control: cannot read $WF"; exit 2; }

# Print the body of a top-level YAML key ($2) of file $1: the lines after `$2:`
# up to the next column-0 key.
top_block() {
  awk -v k="$2" '
    $0 ~ "^" k ":" { inb = 1; next }
    inb && /^[A-Za-z_"'\''][^:]*:/ { exit }
    inb { print }
  ' "$1"
}

# Print the body of job $2 (two-space indented key under `jobs:`) of file $1.
job_block() {
  awk -v j="$2" '
    /^jobs:/ { injobs = 1; next }
    injobs && /^[^ #]/ { injobs = 0 }
    injobs && $0 ~ "^  " j ":[[:space:]]*$" { inb = 1; next }
    inb && /^  [A-Za-z0-9_-]+:[[:space:]]*$/ { exit }
    inb { print }
  ' "$1"
}

# Name of the job whose body contains literal $2.
job_containing() {
  awk -v s="$2" '
    /^jobs:/ { injobs = 1; next }
    injobs && /^  [A-Za-z0-9_-]+:[[:space:]]*$/ { j = $1; sub(":", "", j) }
    injobs && j != "" && $0 !~ /^[[:space:]]*#/ && index($0, s) { print j; exit }
  ' "$1"
}

# ── predicates ───────────────────────────────────────────────────────────────
triggers_nightly_only() {  # $1 = workflow
  # Inline forms (`on: push`, `on: [push, schedule]`) are refused outright.
  grep -qE '^(on|"on"|'\''on'\''):[[:space:]]*[^[:space:]#]' "$1" && return 1
  grep -qE '^(on|"on"|'\''on'\''):' "$1" || return 1
  local keys
  keys="$(top_block "$1" on | grep -oE '^  [A-Za-z_]+:' | tr -d ' :')"
  printf '%s\n' "$keys" | grep -qx schedule || return 1
  printf '%s\n' "$keys" | grep -vxE 'schedule|workflow_dispatch' | grep -q . && return 1
  return 0
}

issues_write() {  # $1 = workflow
  top_block "$1" permissions | grep -qE '^  issues:[[:space:]]*write[[:space:]]*(#.*)?$'
}

seven_day_stop() {  # $1 = workflow
  local job body mjob
  job="$(job_containing "$1" 604800)"
  [ -n "$job" ] || return 1
  body="$(job_block "$1" "$job")"
  printf '%s\n' "$body" | grep -qF -- '--label lab-mutation' || return 1
  printf '%s\n' "$body" | grep -qF 'run=false' || return 1
  printf '%s\n' "$body" | grep -qF 'stopped: owner must re-arm' || return 1
  # The job that runs the mutation gate must be gated on that job's verdict.
  mjob="$(job_containing "$1" 'scripts/mutation-diff-gate.sh run')"
  [ -n "$mjob" ] || return 1
  job_block "$1" "$mjob" | grep -qE "^    if:.*needs\.$job\.outputs\.run == 'true'" || return 1
  job_block "$1" "$mjob" | grep -qE "^    needs:.*\b$job\b" || return 1
}

ticket_guarded_by_failure() {  # $1 = workflow
  local job body
  job="$(job_containing "$1" 'gh issue create')"
  [ -n "$job" ] || return 1
  body="$(job_block "$1" "$job")"
  printf '%s\n' "$body" | grep -qE '^    if:[[:space:]]*(\$\{\{[[:space:]]*)?failure\(\)' || return 1
  printf '%s\n' "$body" | grep -qE '^    needs:.*mutation-diff' || return 1
  printf '%s\n' "$body" | grep -qF 'Owner: ' || return 1
  printf '%s\n' "$body" | grep -qF -- '--label lab-mutation' || return 1
  # Dedup before create: an open ticket means silence, not a second ticket.
  printf '%s\n' "$body" | grep -qE 'gh issue list --state open --label lab-mutation' || return 1
  ! printf '%s\n' "$body" | grep -qE 'gh issue comment'
}

no_continue_on_error() {  # $1 = workflow
  grep -qF 'scripts/mutation-diff-gate.sh run' "$1" || return 1
  ! grep -qE '^[[:space:]]*continue-on-error:' "$1"
}

# ── GREEN arms: the workflow under test ─────────────────────────────────────
if triggers_nightly_only "$WF"; then pass_arm 1 "triggers: schedule/workflow_dispatch only"
else fail_arm 1 "$WF has a trigger other than schedule/workflow_dispatch (or no schedule)"; fi
if issues_write "$WF"; then pass_arm 2 "permissions: issues: write"
else fail_arm 2 "$WF lacks top-level 'issues: write' — a red could not open its ticket"; fi
if seven_day_stop "$WF"; then pass_arm 3 "7-day stop gates the mutation job"
else fail_arm 3 "$WF has no 7-day stop (604800 s, lab-mutation, run=false) gating the mutation job"; fi
if ticket_guarded_by_failure "$WF"; then pass_arm 4 "one ticket, one owner, on failure() only"
else fail_arm 4 "$WF has no failure()-guarded, owned, deduped lab-mutation ticket job"; fi
if no_continue_on_error "$WF"; then pass_arm 5 "no continue-on-error; the verdict step decides the run"
else fail_arm 5 "$WF softens the lane with continue-on-error (or lost its verdict step)"; fi

# ── RED arms: planted defects, same predicates ─────────────────────────────
if [ -z "${MUTATION_WF:-}" ]; then
  T="$(mktemp -d)"
  trap 'rm -rf "${T:?}"' EXIT
  plant() {  # $1 = arm, $2 = predicate, $3 = description, stdin = fixture
    cat > "$T/f.yml"
    if "$2" "$T/f.yml"; then fail_arm "$1" "predicate $2 ACCEPTED a fixture with: $3"
    else pass_arm "$1" "refused: $3"; fi
  }
  sed 's/^  schedule:$/  pull_request:\n  schedule:/' "$WF" | plant 6 triggers_nightly_only "a pull_request trigger"
  grep -vE '^  issues:[[:space:]]*write' "$WF" | plant 7 issues_write "no issues: write"
  sed 's/604800/86400000/' "$WF" | plant 8 seven_day_stop "no 7-day stop"
  sed 's/^    if: failure()$/    if: always()/' "$WF" | plant 9 ticket_guarded_by_failure "ticket on always(), not failure()"
  sed 's#^\( *\)run: scripts/mutation-diff-gate.sh run#\1continue-on-error: true\n\1run: scripts/mutation-diff-gate.sh run#' "$WF" \
    | plant 10 no_continue_on_error "continue-on-error on the verdict step"
fi

if [ "$FAIL" = 0 ]; then echo "mutation-lab-control: PASS ($WF)"; exit 0; fi
echo "mutation-lab-control: FAIL ($WF)"; exit 1
