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
#   arm 6  RED   forged dependabot address + hostile CI env           exit 1 (env is not a signal)
#   arm 7  RED   human beside a forged bot commit                      exit 1, names the human
#   arm 8  RED   self-chosen bot-looking address                       exit 1
#   arm 9  RED   exact dependabot address, no CI env                  exit 1 (fail closed)
#   arm 10 RED   exact address, actor octocat / other bot / EMPTY      exit 1
#   arm 11 N/A   default branch, forged address, empty actor           exit 0, NO exempt count
#   arm 12 RED   0x1f forged fields in the subject                    exit 1
#   arm 13 RED   0x1e-prefixed 0x1f forged fields in the subject                   exit 1
#   arm 14 RED   G signature by a non-web-flow key, bot author         exit 1
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
# GATE_ENV is a list of VAR=value assignments the gate runs under; it plays a
# HOSTILE or empty CI environment (a PR workflow can set any of them). Unset,
# the ambient GITHUB_* are stripped so a real pipeline can never decide an arm.
# No value of it may ever exempt a commit: only a GitHub web-flow signature can,
# and a signature by that key cannot be minted in a throwaway repo.
GATE_ENV=()
run_gate() {
  RC=0
  env -u GITHUB_BASE_REF -u GITHUB_ACTIONS -u GITHUB_ACTOR ${GATE_ENV[@]+"${GATE_ENV[@]}"} \
    "$PMAT" comply check --checks CB-2113 --path "$repo" --format json \
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

# ── arm 6: RED — a forged dependabot address under a hostile CI env (PMAT-1356) ─
# Measured on paiml-mcp-agent-toolkit#1347: CB-2113 refused a dependabot bump
# for a trailer its author cannot add. The exemption that fixes it is for
# commits GitHub's web-flow key signed. This repo has no such commit, and
# GITHUB_ACTIONS/GITHUB_ACTOR (which a PR workflow can set) are not a signal:
# the exact app address with the perfect actor is STILL judged.
BOT='dependabot[bot] <49699333+dependabot[bot]@users.noreply.github.com>'
fgit switch -q feature
fgit commit -q --amend --allow-empty --author="$BOT" -m "deps(deps): Bump pollster from 0.4.0 to 1.0.1"
GATE_ENV=(GITHUB_ACTIONS=true "GITHUB_ACTOR=dependabot[bot]")
run_gate
[ "$RC" -eq 1 ] || fail_arm 6 "a forged bot address under a matching CI env must exit 1 (env is not a signal)"
[ "$STATUS" = "Fail" ] || fail_arm 6 "CB-2113 must be Fail on a forged bot address"
case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 6 "no exemption may be reported without a web-flow signature: $MESSAGE";; *) ;; esac
echo "traceability-control: arm 6 RED   — forged dependabot address + matching CI env: still judged (exit 1)"

# ── arm 7: RED — the exemption does not widen to the humans beside it ────────
# The falsifier for arm 6. If the exemption ever becomes "skip a range that
# contains a bot commit", arm 6 stays green while the gate stops judging.
fgit commit -q --allow-empty -m "feat: mine, untrailered"
human=$(fgit rev-parse --short=7 HEAD)
run_gate
[ "$RC" -eq 1 ] || fail_arm 7 "a human untrailered commit beside a bot one must still exit 1"
[ "$STATUS" = "Fail" ] || fail_arm 7 "CB-2113 must be Fail on the human commit"
case "$MESSAGE" in *"$human"*"no Pmat-Ticket"*) ;; *) fail_arm 7 "the message must name the human commit $human";; esac
case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 7 "the forged bot commit must not be counted exempt: $MESSAGE";; *) ;; esac
echo "traceability-control: arm 7 RED   — human commit beside a forged bot one: both refused, none exempt (exit 1)"

# ── arm 8: RED — an address a person can choose buys nothing ────────────────
# The exemption is GitHub's app-account namespace, not the word "bot".
fgit reset -q --hard HEAD~1
fgit commit -q --amend --allow-empty --author='robot <robot@bot.example.invalid>' -m "deps: not actually a bot"
run_gate
[ "$RC" -eq 1 ] || fail_arm 8 "a self-chosen bot-looking address must not be exempt"
case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 8 "no exemption may be reported for a non-app address: $MESSAGE";; *) ;; esac
echo "traceability-control: arm 8 RED   — robot@bot.example.invalid is not a GitHub app account (exit 1)"


# ── arm 9: RED — a HUMAN using the exact dependabot address, no CI signal ───
# The defect quorum q-toolkit1496-r1 found: %ae is whatever `--author` said.
# The address is byte-for-byte the real app's; only the missing runner signal
# can tell it from the real thing.
GATE_ENV=()
fgit commit -q --amend --allow-empty --author="$BOT" -m "feat: mine, forged author"
run_gate
[ "$RC" -eq 1 ] || fail_arm 9 "the exact dependabot address with no CI signal must exit 1 (fail closed)"
[ "$STATUS" = "Fail" ] || fail_arm 9 "CB-2113 must be Fail on a forged author"
case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 9 "no exemption may be reported without a CI signal: $MESSAGE";; *) ;; esac
echo "traceability-control: arm 9 RED   — exact dependabot address, no CI actor: still judged (exit 1)"

# ── arm 10: RED — no actor value, including an EMPTY one, exempts the address ─
# r3: GITHUB_ACTOR is process env; a PR workflow sets it. Every value is judged.
for actor in octocat 'renovate[bot]' 'dependabot[bot]' ''; do
  GATE_ENV=(GITHUB_ACTIONS=true "GITHUB_ACTOR=$actor")
  run_gate
  [ "$RC" -eq 1 ] || fail_arm 10 "actor '$actor' running a dependabot-addressed commit must exit 1"
  case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 10 "no exemption for actor '$actor': $MESSAGE";; *) ;; esac
done
echo "traceability-control: arm 10 RED  — actor octocat / renovate[bot] / dependabot[bot] / empty on a dependabot address: still judged (exit 1)"

# ── arm 11: default branch, forged address, EMPTY actor: nothing is exempt ───
# Red under the old %ae-only rule (the message would claim an exemption) and
# under an actor-trusting rule only when the actor is non-empty, so the empty
# actor is the variant that a CI-env rule and an address rule both get wrong.
GATE_ENV=(GITHUB_ACTIONS=true "GITHUB_ACTOR=")
fgit switch -q master
fgit commit -q --allow-empty --author="$BOT" -m "deps: bump on the default branch"
run_gate
[ "$RC" -eq 0 ] || fail_arm 11 "the default branch must exit 0"
case "$MESSAGE" in "not_applicable:"*"authored by a GitHub app account"*) fail_arm 11 "a forged address with an empty actor must not be counted exempt: $MESSAGE";; "not_applicable:"*) ;; *) fail_arm 11 "the default-branch message must say not_applicable: $MESSAGE";; esac
echo "traceability-control: arm 11 N/A  — default branch, forged address, empty actor: nothing exempt (exit 0)"


# ── arms 12-13: RED — separator bytes in the SUBJECT cannot forge the fields ─
# r3 NO-GO (q-toolkit1496-r3): the gate split one `git log` line on 0x1f, so an
# UNSIGNED commit whose subject carried "<ae>\x1fG\x1fnoreply@github.com\x1f<fpr>"
# shifted the real fields and was counted exempt. Each variant below must stay
# judged; they go GREEN (exempt) under the r3 rule, which is what makes them arms.
WEBFLOW_FPR=968479A1AFF927E37D1A566BB5690EEEBB952194
BOTAE='49699333+dependabot[bot]@users.noreply.github.com'
GATE_ENV=()
fgit switch -q feature
fgit reset -q --hard HEAD
inject_arm() { # <arm> <label> <separator-byte> <lead-byte>
  local sep lead subj
  sep=$(printf "$3")
  lead=$(printf "$4")
  subj="x${lead}${sep}${sep}${BOTAE}${sep}G${sep}noreply@github.com${sep}${WEBFLOW_FPR}"
  fgit commit -q --allow-empty --author='human <human@example.invalid>' -m "$subj"
  run_gate
  [ "$RC" -eq 1 ] || fail_arm "$1" "a $2 injection in the subject must not exempt an unsigned commit (exit 1)"
  [ "$STATUS" = "Fail" ] || fail_arm "$1" "CB-2113 must be Fail on a $2-injected subject"
  case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm "$1" "no exemption may be reported for a $2-injected subject: $MESSAGE";; *) ;; esac
  fgit reset -q --hard HEAD~1
  echo "traceability-control: arm $1 RED  — $2 forged fields in the subject: still judged (exit 1)"
}
inject_arm 12 "0x1f" '\037' ''
inject_arm 13 "0x1e-prefixed 0x1f" '\037' '\036'

# ── arm 14: RED — a good signature by ANY OTHER KEY, bot author, noreply committer ─
# The %GF leg: %G? = G and committer noreply@github.com are both settable locally.
# A throwaway gpg key is trusted in its own keyring, so git reports G; only the
# key identity separates it from GitHub's. Green if the fingerprint is not compared.
if ! command -v gpg >/dev/null 2>&1; then
  echo "traceability-control: arm 14 cannot run — gpg is absent (not measured, never a pass)" >&2
  exit 2
fi
gnupg="$work/gnupg"; mkdir -m 700 "$gnupg"
GNUPGHOME="$gnupg" gpg -q --batch --pinentry-mode loopback --passphrase '' \
  --quick-generate-key 'other <other@example.invalid>' ed25519 sign never >/dev/null 2>&1 \
  || { echo "traceability-control: arm 14 cannot mint a throwaway key (not measured)" >&2; exit 2; }
GNUPGHOME="$gnupg" GIT_COMMITTER_EMAIL=noreply@github.com \
  GIT_COMMITTER_NAME=GitHub GIT_AUTHOR_NAME=dependabot GIT_AUTHOR_EMAIL="$BOTAE" \
  git -C "$repo" -c commit.gpgsign=false -c gpg.program=gpg -c user.signingkey=other@example.invalid commit -q --allow-empty -S -m "deps: signed by another key"
sig=$(GNUPGHOME="$gnupg" git -C "$repo" log -1 --format=%G? HEAD)
[ "$sig" = "G" ] || { echo "traceability-control: arm 14 setup: expected %G?=G, got '$sig' (not measured)" >&2; exit 2; }
GNUPGHOME="$gnupg" run_gate
[ "$RC" -eq 1 ] || fail_arm 14 "a good signature by a non-web-flow key must not exempt (exit 1)"
case "$MESSAGE" in *"authored by a GitHub app account"*) fail_arm 14 "no exemption may be reported for another key: $MESSAGE";; *) ;; esac
echo "traceability-control: arm 14 RED  — G signature by another key, bot author, noreply committer: still judged (exit 1)"
fgit reset -q --hard HEAD~1

echo "traceability-control: all 14 arms behaved — CB-2113 can fail, can pass, and no author-settable, env-settable or log-framing-settable signal exempts a commit"
