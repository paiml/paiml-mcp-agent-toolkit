#!/usr/bin/env bash
# dogfood — sovereign-stack PRE-RELEASE protocol.
#
# Run inside a Rust crate repo BEFORE generating a GitHub + crates.io release.
# It runs every local release gate, then DOGFOODS the crate (uses its own release
# binary on real data), and writes a go/no-go receipt.
#
# Toyota way: ANY red gate STOPS the release — fix the root cause, never bypass.
#
# Usage:  dogfood.sh [REPO_DIR]        (defaults to $PWD)
# Exit:   0 = GO (all gates green) · 1 = NO-GO (a gate failed) · 2 = setup error
#         3 = the receipt this run wrote is unreadable (verdict withheld: the
#             evidence cannot be read back, so no verdict is reported)
set -uo pipefail

# WHERE THIS SKILL LIVES — resolved BEFORE the `cd` below, and that order is
# load-bearing. `${BASH_SOURCE[0]}` is whatever the caller typed: for the
# documented fleet invocation `bash scripts/dogfood.sh ../other-crate` it is
# RELATIVE, so resolving it after `cd "$REPO_DIR"` looks for this skill's helper
# files inside the TARGET repo. That was harmless while the helpers were
# optional; #2640 made verifier_pin.sh load-bearing and fail-closed, and the
# runner then refused to start on every relative invocation — the exact fleet
# path the whole canon/shim arrangement exists to serve. Absolute invocations
# kept working, which is why it looked fine. Gated by PART 3 of
# scripts/check_verifier_pinning.sh.
SKILL_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)

REPO_DIR="${1:-$PWD}"
cd "$REPO_DIR" 2>/dev/null || { echo "dogfood: no such dir: $REPO_DIR" >&2; exit 2; }
[ -f Cargo.toml ] || { echo "dogfood: not a Rust crate (no Cargo.toml) in $REPO_DIR" >&2; exit 2; }

# Resolve identity from cargo itself, NOT by grepping Cargo.toml.
#
# The sed this replaces produced an EMPTY version for any crate using workspace
# inheritance (`version.workspace = true`) — and an empty version made
# version-unpublished PASS, because the crates.io lookup for "" finds nothing.
# Measured 2026-08-20 on pforge-cli: sed gave "", cargo metadata gives 0.2.1,
# and the gate printed "[ OK ] version-unpublished   not yet published" for a
# version string that did not exist. A false pass in the gate whose entire job
# is to stop an immutable double-publish.
#
# The same sed took the FIRST `name =` in the file, which on a virtual workspace
# manifest is whatever member or dependency happens to sort first.
# `cargo metadata --no-deps` lists EVERY workspace member regardless of cwd, so
# it must be filtered by manifest_path — picking ps[0], or requiring len==1,
# silently yields nothing for any member of a workspace.
_meta() {
  cargo metadata --no-deps --format-version 1 2>/dev/null | python3 -c '
import json,os,sys
field=sys.argv[1]
here=os.path.realpath("Cargo.toml")
try:
    d=json.load(sys.stdin)
except Exception:
    print(""); raise SystemExit
for p in d.get("packages",[]):
    if os.path.realpath(p.get("manifest_path","")) == here:
        print(p.get(field,"")); raise SystemExit
print("")' "$1" 2>/dev/null
}
CRATE=$(_meta name)
VERSION=$(_meta version)
# Fall back to the old parse only if cargo could not answer, and NEVER proceed
# with an empty version: an unknown version cannot be checked against crates.io.
CRATE="${CRATE:-$(sed -n 's/^name = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
VERSION="${VERSION:-$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -1)}"
if [ -z "$CRATE" ] || [ -z "$VERSION" ]; then
  echo "dogfood: cannot resolve this crate's identity (name='${CRATE}' version='${VERSION}')." >&2
  echo "  A virtual workspace root has no package of its own. Run dogfood from the" >&2
  echo "  member you are releasing, e.g. crates/<name>/ — pointing it at the root" >&2
  echo "  built the wrong binary (telemetry-server instead of pforge) and cascaded" >&2
  echo "  into 'no package named \'\'' on 2026-08-20." >&2
  ls -d ./*/ crates/*/ 2>/dev/null | head -12 | sed 's/^/    candidate: /' >&2
  exit 2
fi
# Crates that gate their binary behind a `cli` feature need it for test/clippy.
# Look ONLY inside [features]: `cli = { e2e = "…" }` in
# [package.metadata.transports] is not a cargo feature, and matching it made
# every gate below run with `--features cli` against a crate that has no such
# feature ("error: the package does not contain this feature: cli").
#
# A crate may also declare a feature that gates DELIBERATELY-RED tests. pmat's
# Cargo.toml documents `red-phase-tests` as "red-phase TDD tests (expected
# failures, not yet passing) ... NEVER add to any feature bundle" — the same
# contract as `broken-tests`, under a different name. Enabling it turned 24
# by-design failures into a release-blocking NO-GO, which is the harness
# overriding the project's own declaration rather than measuring it.
#
# `--all-features` is NOT a safe fallback. A crate may declare a feature that is
# DELIBERATELY non-compiling: pmat has `broken-tests`, a quarantine with 49
# sites (paiml-mcp-agent-toolkit#1023). Using --all-features there measures the
# quarantine, not the crate, and yields a permanent RED that says nothing about
# release readiness — and a gate that is always red is one everybody learns to
# walk past.
#
# So the fallback is every declared feature MINUS known-broken quarantines, and
# the exclusion is REPORTED. A silent exclusion is how a gate quietly stops
# covering what it claims to.
if awk '/^\[features\]/{f=1;next} /^\[/{f=0} f' Cargo.toml | grep -qE '^cli *='; then
  FEATS="--features cli"; FEAT_NOTE="--features cli"
else
  ALL_FEATS=$(awk '/^\[features\]/{f=1;next} /^\[/{f=0} f' Cargo.toml \
    | grep -oE '^[a-zA-Z0-9_-]+ *=' | tr -d ' =' \
    | grep -vxE 'default|broken-tests|broken|wip|red-phase-tests|red-phase' | sort -u | paste -sd, -)
  EXCLUDED=$(awk '/^\[features\]/{f=1;next} /^\[/{f=0} f' Cargo.toml \
    | grep -oE '^[a-zA-Z0-9_-]+ *=' | tr -d ' =' \
    | grep -xE 'broken-tests|broken|wip|red-phase-tests|red-phase' | sort -u | paste -sd, -)
  if [ -n "$ALL_FEATS" ]; then
    FEATS="--features $ALL_FEATS"
    if [ -n "$EXCLUDED" ]; then FEAT_NOTE="all features EXCEPT quarantined: $EXCLUDED"
    else FEAT_NOTE="all declared features"; fi
  else
    FEATS="--all-features"; FEAT_NOTE="--all-features"
  fi
fi

# $PWD, not $REPO_DIR: the cd above already resolved the target, and a RELATIVE
# argument ("bash dogfood.sh crates/foo" from a repo root) used to concatenate
# into $REPO_DIR/$REPO_DIR/.dogfood — the receipt nested INSIDE the target tree
# one level too deep, where the next run's git-clean counted it as dirt
# (#2644, DF-3). Absolute by construction now.
RECEIPT_DIR="$PWD/.dogfood"
mkdir -p "$RECEIPT_DIR"
TS=$(date -u +%Y%m%dT%H%M%SZ)
# The commit the evidence describes, stamped INTO the receipt: a consumer that
# globs for the newest receipt after a crashed run would otherwise read a
# previous verdict as current with nothing to expose it (#2644, DF-12). With
# the SHA inside, staleness is checkable against HEAD; "unknown" is itself
# evidence (a non-repo cannot carry the tag a receipt gates).
RECEIPT_SHA=$(git rev-parse HEAD 2>/dev/null) || RECEIPT_SHA=unknown
RECEIPT="$RECEIPT_DIR/receipt-$TS.json"
# Written as .partial and atomically renamed on completion: a mid-run crash
# leaves NO receipt — never a complete-looking one (#2644, DF-12 / QUAL-013).
RECEIPT_PARTIAL="$RECEIPT.partial"

# ── gate bookkeeping ─────────────────────────────────────────────────────────
declare -a NAMES=() RESULTS=() NOTES=()
FAILED=0
# Statuses:
#   PASS   — the gate ran and its subject held.
#   FAIL   — the gate ran and its subject did not hold  → NO-GO.
#   SKIP   — the gate's subject does not exist in this crate, and its absence is
#            legitimate. The note MUST record the enumeration that found nothing,
#            so "no subject" can be told apart from "did not look".
#   REPORT — the gate RAN and produced a measurement that is deliberately NOT
#            gating. Every REPORT must carry (a) the number it measured and
#            (b) the upstream issue it is waiting on. A REPORT with no issue
#            number is a WARN wearing a costume; do not add one.
#   MANUAL — a hard gate that runs elsewhere (clean-room).
# WARN is retained only for pre-existing hygiene checks. Do not add new WARNs:
# "a WARN in a release protocol is a step everybody learns to walk past."
# Strip BOTH CSI sequences (\e[0m) and charset selects (\e(B, which cargo fmt
# emits) — a note that renders as "(B[m" tells the reader nothing.
strip_ansi() { sed -e 's/\x1b\[[0-9;]*[A-Za-z]//g' -e 's/\x1b([A-Z]//g'; }
gate() { # gate <name> <cmd...> — runs cmd, records pass/fail
  local name="$1"; shift
  local out rc
  out=$("$@" 2>&1); rc=$?          # command substitution, NOT a pipeline: rc is cmd's
  out=$(printf '%s' "$out" | strip_ansi)
  local note
  note=$(printf '%s' "$out" | grep -iE 'error|fail|warning:|denied|✗|regression' | head -1)
  [ -z "$note" ] && note=$(printf '%s' "$out" | tail -1)
  NAMES+=("$name")
  if [ $rc -eq 0 ]; then RESULTS+=("PASS"); else RESULTS+=("FAIL"); FAILED=1; fi
  NOTES+=("${note:0:120}")
  printf '  [%s] %-26s %s\n' "$([ $rc -eq 0 ] && echo ' OK ' || echo 'FAIL')" "$name" "${note:0:80}"
}
mark() { # mark <name> <PASS|FAIL|SKIP|REPORT|WARN|MANUAL> <note>
  NAMES+=("$1"); RESULTS+=("$2"); NOTES+=("${3:0:200}")
  [ "$2" = FAIL ] && FAILED=1
  printf '  [%s] %-26s %s\n' "$([ "$2" = PASS ] && echo ' OK ' || echo "$2")" "$1" "${3:0:96}"
}
# run_to <logfile> <cmd...> — runs cmd with stdout+stderr to <logfile> and puts
# the command's OWN exit status in $RUN_RC. Never a pipeline: `cmd | tee log`
# yields tee's status, and `cmd | grep -q x || fallback` binds the `||` to grep.
# Both have produced false GREENs in this fleet; use this and read $RUN_RC.
RUN_RC=0
run_to() { local log="$1"; shift; "$@" > "$log" 2>&1; RUN_RC=$?; }
# run_split <outfile> <errfile> <cmd...> — same, but keeps stdout and stderr
# apart (bashrs puts machine-readable JSON on stdout and its scan receipt on
# stderr; merging them destroys the receipt check).
run_split() { local o="$1" e="$2"; shift 2; "$@" > "$o" 2> "$e"; RUN_RC=$?; }

# SKILL_DIR is resolved at the top of this file, before the `cd` — see the note
# there. It must not be recomputed here: from inside the target repo a relative
# ${BASH_SOURCE[0]} points at the wrong tree.
WORKLOG=$(mktemp -d)
trap 'rm -rf "$WORKLOG"' EXIT

# THE VERIFIER-PINNING RULE, and the two pins that implement it, live in exactly
# one file. Read scripts/verifier_pin.sh — the rule is stated there and nowhere
# else, because a rule restated is a rule that drifts. This runner only CALLS it.
if [ -f "$SKILL_DIR/verifier_pin.sh" ]; then
  . "$SKILL_DIR/verifier_pin.sh"
else
  echo "dogfood: $SKILL_DIR/verifier_pin.sh is missing." >&2
  echo "  It carries the rule that decides WHICH pv and WHICH pmat this protocol" >&2
  echo "  is allowed to believe. Without it every verifier would fall back to" >&2
  echo "  PATH, which is the exact defect #2640 closed. Refusing to run." >&2
  exit 2
fi

# THE PINS RESOLVE BEFORE ANY GATE RUNS. The declared gates below execute as
# CHILD processes and consume the pins from the environment (the lib exports
# them) — a pin resolved after they ran is a pin delivered to nobody, while
# pin_audit certified consumption the environment never carried (#2644, VPIN-4).
#
# The pmat pin is called here with NO artifact: for every crate but pmat that
# already yields the final answer (the fleet pmat). For the one crate where the
# artifact matters — releasing pmat itself — it cannot exist before the build,
# so this phase leaves PMAT_BIN EMPTY there and any declared gate consuming it
# fails closed rather than measuring a PATH binary that is not the build being
# released. The post-build call further down upgrades the pin to the artifact.
verifier_pin_pmat "$CRATE" ""
PV=""
verifier_pin_pv
VERIFIER_PIN_PV_RC=$?

echo "══ dogfood pre-release: $CRATE v$VERSION ══"

# ── 1. hygiene ───────────────────────────────────────────────────────────────
# `.dogfood/` (this receipt), `.pmat/` (the index the pmat gate must build before
# CB-200 can run at all) and `.pv/` (pv's lint cache) are artefacts THIS PROTOCOL
# creates. Excluding them is not hiding dirt — it is refusing to let the
# measurement fail itself. Everything else, including untracked shell scripts,
# still counts. Ask the crate to gitignore all three.
if ! git rev-parse --is-inside-work-tree >/dev/null 2>&1; then
  # A failed `git status` used to leave DIRTY empty, and empty read as clean:
  # the absence of the measurement wearing the measurement's PASS (#2644, DF-7).
  # A release is a tag; a directory that cannot carry one fails the gate.
  mark git-clean FAIL "not a git repository — cleanliness cannot be measured and a tag cannot exist; a missing measurement is RED, never clean"
else
  DIRTY=$(git status --porcelain 2>/dev/null | grep -vE '^\?\? ([^ ]*/)?\.(dogfood|pmat|pv)/?$' | head -1)
  if [ -z "$DIRTY" ]; then mark git-clean PASS "working tree clean"
  else mark git-clean WARN "uncommitted changes present (commit before tagging)"; fi
fi

# ── 1b. the crate's OWN release gates, DISCOVERED not duplicated ────────────
#
# A repo that has written its own release guards must not have to get them
# copied into this runner — that is how the runner acquired a second copy of
# itself (#2640). The declaration lives in the crate's Cargo.toml, versioned
# with the code it guards, and reuses the mechanism this runner already
# implements for [package.metadata.transports]:
#
#   [package.metadata.dogfood]
#   gates = ["scripts/check_multiplatform_dogfood.sh", "scripts/dogfood_surfaces.sh"]
#
# Each discovered gate gets its OWN row in the receipt. Rolling them into one
# aggregate verdict is how a red gate hides behind a green neighbour.
#
# TWO VACUITY GUARDS, because a clean sweep over an empty set is this protocol's
# signature failure and it reappears one layer up at DISCOVERY:
#   · a declared script that does not exist is FAIL, never SKIP. "Declared but
#     absent" is the state a deleted guard leaves behind, and a SKIP would make
#     deleting a guard the cheapest way to make it stop complaining.
#   · no [package.metadata.dogfood] block at all, or `gates = []`, is FAIL. A
#     crate with zero release gates of its own is a claim, and an unmade claim
#     reads exactly like a satisfied one.
run_to "$WORKLOG/meta-dogfood.json" cargo metadata --no-deps --format-version 1
DG_META_RC=$RUN_RC
# ONE parser, shared with scripts/check_verifier_pinning.sh — see
# scripts/lib/dogfood_gates.py. This used to be an embedded heredoc while the
# guard scraped the same TOML with awk+grep, so the guard's scan universe could
# be strictly smaller than the set the runner EXECUTES (#2644 audit, CI-3/VP-05).
DG_PLAN=$(CRATE="$CRATE" python3 "$SKILL_DIR/lib/dogfood_gates.py" \
  "$WORKLOG/meta-dogfood.json" 2>/dev/null || echo "META_ERROR")
if [ "$DG_META_RC" -ne 0 ]; then
  mark dogfood-gates FAIL "\`cargo metadata\` failed (exit=$DG_META_RC) — the release gates this crate declares could not be discovered, so none of them ran"
elif [ "$DG_PLAN" = "META_ERROR" ]; then
  # cargo succeeded; the python step died. The old message blamed cargo with
  # "failed (exit=0)" attached — an operator sent to the wrong component
  # (#2644, CI-4).
  mark dogfood-gates FAIL "gate discovery's python3 step failed (cargo metadata itself exited 0) — \`$SKILL_DIR/lib/dogfood_gates.py\` could not parse the declaration (or is missing), so no declared gate ran"
elif [ "$DG_PLAN" = "NOPKG" ]; then
  mark dogfood-gates FAIL "no package named '$CRATE' in cargo metadata — run dogfood from the crate dir, not the virtual workspace root"
elif [ "$DG_PLAN" = "NODECL" ]; then
  mark dogfood-gates FAIL "no [package.metadata.dogfood] in Cargo.toml. Declare the release gates this crate owns, e.g.  [package.metadata.dogfood]  gates = [\"scripts/check_multiplatform_dogfood.sh\"]. A crate that declares none is asserting it has none — say so deliberately, in the manifest, where review can see it."
elif [ "$DG_PLAN" = "EMPTY" ]; then
  mark dogfood-gates FAIL "[package.metadata.dogfood] declares \`gates = []\` — a clean sweep over an empty gate set is the vacuous pass this protocol exists to refuse. Declare the gates, or remove the claim to have any."
elif [ "$DG_PLAN" = "BADSHAPE" ]; then
  mark dogfood-gates FAIL "[package.metadata.dogfood] gates must be a non-empty list of script paths — a malformed declaration verifies nothing"
else
  DG_N=0; DG_BAD=0
  while read -r dg_kind dg_path; do
    [ "$dg_kind" = "GATE" ] || continue
    DG_N=$((DG_N + 1))
    dg_name="declared:$(basename "$dg_path" .sh)"
    if [ ! -f "$dg_path" ]; then
      mark "$dg_name" FAIL "declared in [package.metadata.dogfood] but no such file: $dg_path — a declared gate that does not exist is a deleted gate, not an absent requirement"
      DG_BAD=$((DG_BAD + 1)); continue
    fi
    run_to "$WORKLOG/$(basename "$dg_path").log" bash "$dg_path"
    dg_rc=$RUN_RC
    dg_tail=$(tail -3 "$WORKLOG/$(basename "$dg_path").log" 2>/dev/null | strip_ansi | tr '\n' ' ')
    if [ "$dg_rc" -eq 0 ]; then
      mark "$dg_name" PASS "$dg_path exit=0"
    else
      mark "$dg_name" FAIL "$dg_path exit=$dg_rc — $dg_tail"
      DG_BAD=$((DG_BAD + 1))
    fi
  done <<EOF
$DG_PLAN
EOF
  if [ "$DG_N" -eq 0 ]; then
    mark dogfood-gates FAIL "[package.metadata.dogfood] parsed but yielded no gate — a declaration matching nothing is not a pass"
  elif [ "$DG_BAD" -eq 0 ]; then
    mark dogfood-gates PASS "$DG_N declared gate(s) discovered and all green"
  else
    mark dogfood-gates FAIL "$DG_N declared gate(s) discovered, $DG_BAD RED (each named in its own row above)"
  fi
fi

# DOGFOOD_GATES_ONLY runs the discovery section and stops. It exists so the
# discovery mechanism itself can be exercised — by
# scripts/check_verifier_pinning.sh's fleet-path test, and by hand — without
# paying for a full release sweep, and it runs the SAME
# code the release runs rather than a copy of it, which is the whole point of
# this ticket. It can never print GO: a partial run is not a verdict.
if [ -n "${DOGFOOD_GATES_ONLY:-}" ]; then
  echo "────────────────────────────────────────────────"
  echo "PARTIAL: DOGFOOD_GATES_ONLY — only the declared repo gates above ran."
  echo "         This is NOT a release verdict. A GO comes only from a full run."
  exit "$FAILED"
fi

# ── 2 + 10. version + publish dry-run (authoritative: cargo's own registry
# index, not a flaky crates.io HTTP call). A dry-run SUCCEEDS even when the
# version exists (it only warns), so the "already exists" string — not the exit
# code — is what tells us the version is taken.
DRY=$(env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --allow-dirty 2>&1); DRC=$?
# Here-string, never `printf | grep -q`: with the marker early and more than a
# pipe buffer behind it, grep exits at first match, printf takes SIGPIPE, and
# under pipefail the `if` reads 141 — a PUBLISHED version marked "not yet
# published" (#2644, DF-2; the same construct inverted a verdict the other way
# in the pinning guard, VP-06).
if grep -qiE "already (exists|uploaded)" <<< "$DRY"; then
  mark version-unpublished FAIL "$CRATE $VERSION is ALREADY on crates.io — bump the version"
elif [ "$DRC" -ne 0 ]; then
  # No already-exists marker AND the dry-run itself died: the registry was
  # never consulted, so "not yet published" is an assertion with no source
  # behind it (#2644, DF-8) — same empty-and-zero rule as the advisory gate.
  mark version-unpublished FAIL "cargo publish --dry-run failed (exit=$DRC) with no already-exists marker — the registry was never consulted, so the version's status is UNKNOWN: $(tail -2 <<< "$DRY" | strip_ansi | tr '\n' ' ' | cut -c1-120)"
else
  mark version-unpublished PASS "$VERSION not yet published (dry-run exit 0, no already-exists marker)"
fi

# ── 3. changelog mentions the version ───────────────────────────────────────
# A workspace member keeps its CHANGELOG and Makefile at the repo root. Looking
# only in the crate dir reported "no CHANGELOG.md" for a repo that has one three
# directories up — the same false-negative that made dogfood-use claim a crate
# had no dogfood gate when it did.
REPO_ROOT="$(git rev-parse --show-toplevel 2>/dev/null || echo .)"
find_up() {  # echo the first of $1 in CWD then the repo root, or nothing
  [ -f "$1" ] && { printf '%s' "$1"; return; }
  [ -f "$REPO_ROOT/$1" ] && printf '%s' "$REPO_ROOT/$1"
}
CHANGELOG_PATH=$(find_up CHANGELOG.md)
if [ -n "$CHANGELOG_PATH" ] && grep -qF "$VERSION" "$CHANGELOG_PATH"; then mark changelog PASS "CHANGELOG has $VERSION"
elif [ -n "$CHANGELOG_PATH" ]; then mark changelog WARN "$CHANGELOG_PATH has no entry for $VERSION"
else mark changelog WARN "no CHANGELOG.md (looked in $PWD and $REPO_ROOT)"; fi

# ── 4-8. quality gates ──────────────────────────────────────────────────────
mark feature-scope INFO "clippy/test run with: ${FEAT_NOTE}"
gate fmt              cargo fmt --all -- --check
# shellcheck disable=SC2086
gate clippy           cargo clippy --all-targets $FEATS -- -D warnings
# shellcheck disable=SC2086
gate test             cargo test $FEATS
MAKEFILE_PATH=$(find_up Makefile)
if [ -n "$MAKEFILE_PATH" ] && grep -qE '^coverage-check:' "$MAKEFILE_PATH" 2>/dev/null; then
  gate coverage make -C "$(dirname "$MAKEFILE_PATH")" coverage-check
else mark coverage FAIL "no coverage-check make target in ${MAKEFILE_PATH:-$PWD/Makefile} — the >=95% floor is UNVERIFIED, which is not the same as met (was a WARN, contradicting this skill's own rule that a missing capability is a NO-GO)"; fi
if command -v cargo-deny >/dev/null 2>&1; then gate security cargo deny check advisories
else mark security FAIL "cargo-deny not installed — the advisory scan did not run, and a scan that did not run is not a clean scan"; fi

# ── security, second source: cargo-deny's GREEN is only as wide as RustSec ──
#
# `cargo deny check advisories` answers "is anything in my tree listed in the
# RustSec database". That is NOT the same question as "is anything in my tree
# known-vulnerable", and the difference is not theoretical:
#
#   pmat 3.32.0, 2026-08-21. `cargo deny check advisories` printed
#   "no crate matched advisory criteria / advisories ok" while
#   thrift 0.17.0 sat in the tree (parquet 57.3.1 <- aprender-db) carrying
#   CVE-2026-43868, medium, CVSS 5.3, patched in 0.23.0. RustSec's db, pulled
#   the same day with 1,208 advisories, had no thrift entry at all. GitHub's
#   Advisory Database did.
#
# So cargo-deny was telling the truth about its database and a falsehood about
# the tree — the same shape as every other gate this protocol exists to catch.
# GitHub's Dependabot is an INDEPENDENT database, which is the whole point of
# consulting it: two sources fail differently.
#
# WARN, not FAIL, and deliberately: this reads a remote service over the
# network, it needs `gh` authenticated, and a transitive advisory with no
# semver-reachable fix must not be able to block a release on its own. But it
# must never be silent — an unreported alert is how "advisories ok" gets
# believed. Absence of `gh` is reported too: a source that did not run is not
# a source that found nothing.
if command -v gh >/dev/null 2>&1 && gh auth status >/dev/null 2>&1; then
  # rc is captured, stderr is kept, and a FAILED call routes to the did-not-run
  # branch: `2>/dev/null … || true` made an empty result from a dead call
  # indistinguishable from a clean scan (#2644, DF-1). The rule this encodes:
  # a probe whose empty output means PASS must FAIL on rc!=0 — "empty-and-zero"
  # is the only PASS shape (v1.13 P7).
  DEPA=$(gh api "repos/{owner}/{repo}/dependabot/alerts" --paginate \
           --jq '.[] | select(.state=="open") |
                 "\(.security_advisory.severity)|\(.dependency.package.name)|\(.security_vulnerability.first_patched_version.identifier // "no-fix")"' \
         2>"$WORKLOG/depa.err")
  DEPA_RC=$?
  if [ "$DEPA_RC" -ne 0 ]; then
    mark security-2nd-source WARN "gh api FAILED (exit=$DEPA_RC: $(tail -1 "$WORKLOG/depa.err" 2>/dev/null | strip_ansi | cut -c1-100)) — the second advisory source did NOT run; an empty result from a failed call is not a clean scan"
  elif [ -z "$DEPA" ]; then
    mark security-2nd-source PASS "GitHub Advisory DB (independent of RustSec): 0 open Dependabot alerts"
  else
    DN=$(printf '%s\n' "$DEPA" | grep -c . )
    DCRIT=$(printf '%s\n' "$DEPA" | grep -c '^critical\|^high' || true)
    DLIST=$(printf '%s\n' "$DEPA" | awk -F'|' '{printf "%s %s(fix:%s) ", $1, $2, $3}')
    if [ "${DCRIT:-0}" -gt 0 ]; then
      mark security-2nd-source FAIL "GitHub Advisory DB reports $DN open alert(s), $DCRIT of them high/critical, that cargo-deny's RustSec scan did not: $DLIST"
    else
      mark security-2nd-source WARN "GitHub Advisory DB reports $DN open alert(s) cargo-deny did not see (none high/critical): $DLIST — RustSec and GHSA are different databases; a green cargo-deny does not mean the tree is clean"
    fi
  fi
else
  mark security-2nd-source WARN "gh unavailable or unauthenticated — the second advisory source did not run, so \`advisories ok\` rests on RustSec alone"
fi

# ── build the release binary ONCE, HERE ─────────────────────────────────────
# Everything below verifies the ARTIFACT: renacer traces it, cli-surface reads
# its --help, the interface gates spawn it, dogfood-use runs it. It used to be
# built two-thirds of the way down, AFTER the renacer gate already referenced
# $BINPATH — and under `set -u` that is a fatal unbound-variable abort, not a
# skipped gate. Reproduced: a crate with a renacer.toml (trueno has one, and
# renacer is installed here) aborted at `dogfood.sh: line 181: BINPATH: unbound
# variable`, exit 1 — the SAME exit code this script uses for NO-GO. A crashed
# protocol was indistinguishable from a considered verdict, and it never
# reached the dogfood step, the contracts step, or the receipt.
# shellcheck disable=SC2086
BUILD_JSON=$(cargo build --release $FEATS --message-format=json 2>/dev/null)
# Prefer an executable named after the crate...
BINPATH=$(printf '%s' "$BUILD_JSON" \
  | sed -n 's/.*"executable":"\([^"]*'"$CRATE"'\)".*/\1/p' | tail -1)
# ...but a crate's binary need not share its name (aprender ships `apr`), so
# fall back to whatever executable the build actually produced.
if [ -z "$BINPATH" ]; then
  BINPATH=$(printf '%s' "$BUILD_JSON" \
    | sed -n 's/.*"executable":"\([^"]*\)".*/\1/p' | grep -v null | tail -1)
fi
BINPATH="${BINPATH:-}"
# Which pmat runs the pmat gates. The rule and its measured evidence are in
# scripts/verifier_pin.sh; this is the post-build call site — the early phase
# above already delivered the policy answer to the declared gates, and this one
# upgrades the pin to the just-built artifact for the case where the crate IS
# pmat. Its verdict gets a row in EVERY receipt: a pin that reports only on
# failure is the version-conditional vanish this audit just closed (DF-9 class).
verifier_pin_pmat "$CRATE" "$BINPATH"
VERIFIER_PIN_PMAT_RC=$?
if [ "$VERIFIER_PIN_PMAT_RC" -ne 0 ]; then
  mark pmat-pin FAIL "releasing pmat but the built artifact is missing or failed behavior verification (--version) — refusing the PATH fallback: a stale fleet pmat measuring a pmat release is the recorded 3.32.0-vs-3.32.0 incident (see scripts/verifier_pin.sh). Every pmat gate below is unverifiable against the artifact being released"
elif [ "$CRATE" = "pmat" ]; then
  mark pmat-pin PASS "releasing pmat: gates measure the BUILT artifact ($PMAT_BIN), behavior-verified"
else
  mark pmat-pin PASS "fleet pmat is the correct verifier for a non-pmat crate (exported to declared gates)"
fi
readonly PMAT_BIN
if [ -n "$BINPATH" ] && [ -x "$BINPATH" ]; then
  mark release-binary PASS "built: $BINPATH"
else
  mark release-binary FAIL "cargo build --release $FEATS produced no executable — every artifact gate below is unverifiable"
fi

# ── 8b. Sovereign deterministic tooling ─────────────────────────────────────
# These are the fleet's own tools, and they are deterministic: same input, same
# verdict, no model in the loop. They are gated here rather than left to a
# human because every one of them has caught something CI did not.
#
# A MISSING TOOL IS REPORTED, NEVER SILENTLY SKIPPED. A gate that goes quiet
# when its subject is absent is indistinguishable from a gate that passed, which
# is the failure mode this whole protocol exists to prevent.

# ── DETERMINISTIC TOOLS ARE MANDATORY ───────────────────────────────────────
#
# pv, bashrs and probador are the verifiers this protocol is built on. They are
# DETERMINISTIC — the same input yields the same verdict on any machine, unlike
# reading the diff and forming an opinion.
#
# Their ABSENCE is a NO-GO, not a WARN. This used to WARN, which contradicted
# this skill's own rule two sections up: "a WARN in a release protocol is a step
# everybody learns to walk past." A release verified by tools that were not
# installed is an unverified release that says GO.
#
# Concretely, on 2026-08-16 `pv validate` was run across forjar's contracts for
# the first time and SIX of them had never passed it — including the contract
# `forjar prove` maps into its own "N/N proofs passed" line. Nothing had ever
# run the validator, so nothing knew.
#
# pv is PINNED, never PATH-resolved. The rule, and the measurement behind it,
# are in scripts/verifier_pin.sh; `verifier_pin_pv` is the pin. A repo that
# ships no pin leaves PV empty and the contract gates below REPORT that rather
# than falling back to whatever PATH offers.
MISSING_TOOLS=""
# pv was pinned ONCE, in the early phase before any gate ran (the declared gates
# consume it from the environment); this consumes that verdict rather than
# resolving a second time — two resolutions can disagree, and the one the gates
# already used is the only one that matters.
case "$VERIFIER_PIN_PV_RC" in
  0) : ;;                                              # pinned, exported, behavior-verified
  1) MISSING_TOOLS="$MISSING_TOOLS pv" ;;              # pin present, failed to resolve
  *) : ;;                                              # this repo ships no pin
esac
readonly PV
# bashrs and probador are unpinned in this repo, so PATH is the only answer
# there and asking it is correct. pmat is NOT in that list: it is PINNED, and
# `command -v pmat` would ask PATH about a tool whose answer the pin has already
# decided — for the one crate where the pin matters (releasing pmat itself, with
# the built artifact and possibly nothing on PATH) it reports the wrong thing.
for t in bashrs probador; do
  command -v "$t" >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS $t"
done
command -v "$PMAT_BIN" >/dev/null 2>&1 || MISSING_TOOLS="$MISSING_TOOLS pmat"
if [ -n "$MISSING_TOOLS" ]; then
  mark deterministic-tools FAIL "NOT INSTALLED:$MISSING_TOOLS — install them; a release cannot be verified by absent verifiers"
else
  # Every version here comes from the binary the gates will actually run. This
  # line used a bare `pmat --version` inside the quoted note — the ONE row whose
  # job is to say which pmat measured this release named a pmat off PATH, which
  # may be a different build at the same version string (verifier_pin.sh records
  # the 3.32.0-vs-3.32.0 case). It read as inert text to the guard, too: a
  # command substitution inside double quotes is still command position.
  mark deterministic-tools PASS "pv $("${PV:-:}" --version 2>/dev/null | head -1 | awk '{print $2}'), bashrs $(bashrs --version 2>/dev/null | head -1 | awk '{print $2}'), pmat $("$PMAT_BIN" --version 2>/dev/null | head -1 | awk '{print $2}'), probador $(probador --version 2>/dev/null | head -1 | awk '{print $2}')"
fi

# pv — provable-contract validation (YAML contracts, verification ladder).
# Validates EVERY contract, and treats a directory that yields no contracts as a
# failure: a glob matching nothing passes vacuously, which is the shape this
# whole protocol exists to catch.
#
# NEVER swap this per-file loop for `pv lint <FILE>`. Verified three ways, all
# exit 0 with "Result: PASS": on a contract that `pv validate` rejects, on
# binding.yaml, and on a path that DOES NOT EXIST. `pv lint` only reads
# DIRECTORIES; handed a file it lints zero contracts and calls that a pass.
# `pv lint <DIR>` is a real gate and is run separately below.
if [ -n "${PV:-}" ] && [ -x "$PV" ]; then
  # POSITIVE CONTROL: a deliberately malformed contract must be rejected. If it
  # is not, "N contracts valid" is a count of files, not a verdict.
  printf 'id: BOGUS\nnot_a_real_field: 1\n' > "$WORKLOG/bogus-contract.yaml"
  run_to "$WORKLOG/pv-pc.log" "$PV" validate "$WORKLOG/bogus-contract.yaml"
  PV_PC_RC=$RUN_RC
  if [ -d contracts ]; then
    if [ "$PV_PC_RC" -eq 0 ]; then
      mark pv-contracts FAIL "POSITIVE CONTROL FAILED: \`pv validate\` accepted a malformed contract (exit 0) — its verdicts cannot be trusted this run"
    else
    PV_FAILED=""
    PV_N=0
    # `find`, not a top-level glob: 549 of aprender's contracts live in
    # subdirectories and were silently outside the gate that carries the word
    # "contracts" in its name (#2644, DF-6). Sorted for a deterministic
    # receipt; NUL-delimited so no path shape can split.
    while IFS= read -r -d '' c; do
      case "$(basename "$c")" in
        binding.yaml) continue ;;   # a binding REGISTRY, not a contract
      esac
      PV_N=$((PV_N + 1))
      "$PV" validate "$c" >/dev/null 2>&1 || PV_FAILED="$PV_FAILED ${c#contracts/}"
    done < <(find contracts -name '*.yaml' -type f -print0 | sort -z)
    if [ "$PV_N" -eq 0 ]; then
      mark pv-contracts FAIL "contracts/ exists but contains no validatable contract — a glob matching nothing is not a pass"
    elif [ -n "$PV_FAILED" ]; then
      mark pv-contracts FAIL "$PV_N checked, FAILED:$PV_FAILED"
    else
      mark pv-contracts PASS "$PV_N contract(s) valid (positive control fired)"
    fi
    fi

    # pv lint over the DIRECTORY — the 8-gate schema sweep (duplicate ids,
    # dangling refs, reverse coverage, composition). Directory form only.
    PV_BINDARG=()
    [ -f contracts/binding.yaml ] && PV_BINDARG=(--binding contracts/binding.yaml --crate-dir .)
    run_to "$WORKLOG/pv-lint.log" "$PV" lint contracts "${PV_BINDARG[@]+"${PV_BINDARG[@]}"}"
    PV_LINT_RC=$RUN_RC
    PV_LINT_SUM=$(grep -m1 '^Summary:' "$WORKLOG/pv-lint.log" | strip_ansi)
    if [ "$PV_LINT_RC" -eq 0 ]; then
      mark pv-lint PASS "${PV_LINT_SUM:-8/8 gates passed}"
    else
      mark pv-lint FAIL "${PV_LINT_SUM:-lint failed} — see $WORKLOG/pv-lint.log"
    fi

    # R3 RESOLVES — every binding names a symbol that exists in this source
    # tree. `pv proof-status --binding` does NOT do this: verified against a
    # binding pointing at `ThisFunctionDoesNotExistAnywhere::nope`, it printed
    # "Bindings 2/2" and exited 0. It counts entries; it does not resolve them.
    # `pv verify-bindings` does resolve, and is CWD-sensitive (run from the
    # crate root or every binding reads as a ghost).
    #
    # REPORT, not FAIL, and here is the number to re-arm against: in pv 0.49.0
    # verify-bindings misses `pub(super) fn` (rmedia: `apply_loudnorm` at
    # crates/rmedia-core/src/audio_cleanup/normalize.rs:27 is reported a ghost
    # and is real) and matches prose-qualified `function:` fields literally
    # ("all_entries (slug field)"). It is also module-blind: a binding with the
    # right function name under a module that does not exist verifies clean.
    # Making it blocking today would make it permanently red on false
    # positives, and a permanently red gate is bypassed for substance.
    # Re-arm when paiml/provable-contracts fixes visibility + module matching.
    # A crate-local R1-R4 linter (rmedia's scripts/lint-contract-bindings.sh)
    # is the enforcing gate meanwhile — run it from the crate's own Makefile.
    if [ -f contracts/binding.yaml ]; then
      run_to "$WORKLOG/pv-vb.log" "$PV" verify-bindings contracts/binding.yaml --crate-name "$CRATE"
      PV_VB_RC=$RUN_RC
      PV_VB_LINE=$(grep -m1 'binding functions verified in source' "$WORKLOG/pv-vb.log" | strip_ansi)
      if [ -z "$PV_VB_LINE" ]; then
        # No verification line at all = the tool did not run (bad path, bad
        # crate name, parse error). That is a FAIL: an unrun verifier is not
        # a clean one, and it is the exact shape this protocol exists to catch.
        mark pv-bindings FAIL "\`pv verify-bindings\` produced no verification line (exit=$PV_VB_RC): $(head -1 "$WORKLOG/pv-vb.log" | strip_ansi | head -c 100)"
      elif [ "$PV_VB_RC" -eq 0 ]; then
        mark pv-bindings PASS "$PV_VB_LINE"
      else
        PV_GHOSTS=$(grep -c '^  - ' "$WORKLOG/pv-vb.log")
        mark pv-bindings REPORT "$PV_VB_LINE; $PV_GHOSTS ghost(s) — NOT gating: pv 0.49.0 misses pub(super) fn and is module-blind (paiml/provable-contracts, re-arm when fixed). Triage each ghost by hand."
      fi
    else
      mark pv-bindings SKIP "no contracts/binding.yaml — nothing to resolve"
    fi
  else
    mark pv-contracts SKIP "no contracts/ directory in this crate"
  fi
else
  # Branch on WHY pv is absent: rc=1 (pin present, FAILED) and rc=2 (no pin
  # shipped) used to share one note claiming "no scripts/pv_bin.sh" — a wrong
  # diagnosis whenever the file exists and the build failed (#2644, DIV-4).
  if [ "${VERIFIER_PIN_PV_RC:-2}" -eq 1 ]; then
    mark pv-contracts REPORT "scripts/pv_bin.sh is PRESENT but the pin FAILED to build/resolve pv (its diagnostics are on stderr above) — contracts NOT validated. A PATH-resolved pv is refused on purpose: 0.49.0 and 0.63.0 disagree on the binding gate."
  else
    mark pv-contracts REPORT "pv is not pinned in this repo (no scripts/pv_bin.sh) — contracts NOT validated. A PATH-resolved pv is refused on purpose: 0.49.0 and 0.63.0 disagree on the binding gate, so a verdict from an unknown pv is not a verdict."
  fi
fi

# bashrs — shell purification (bashrs 6.66.2).
#
# Gated on the rules that are TRUSTWORTHY, reported on the rules that are not.
# SC1020/SC1035/SC1140 fire inside string literals (paiml/bashrs#226, still OPEN
# in 6.66.2 — verified: even `echo "done"` trips SC1035), so gating on them
# would make this unpassable, and an unpassable gate trains people to bypass the
# whole protocol. SEC*/DET*/IDEM* do not have that defect and are exactly the
# ones that matter: SEC011 caught a genuine unguarded `rm -rf "$tmp"` in a
# forjar resource. Those FAIL the release.
#
# THREE THINGS THIS GATE PROVES BEFORE IT BELIEVES A CLEAN RESULT — because a
# clean bashrs result has three different causes and only one of them is "the
# shell is clean":
#
#  1. POSITIVE CONTROL. A sentinel script containing a known DET002 is linted
#     first. If bashrs does not flag it, bashrs is not enforcing anything here
#     and a green verdict on the real surface means nothing. Measured cost: 83ms.
#  2. THE SCAN RECEIPT. bashrs prints `Linted N file(s): …` on STDERR, and this
#     gate asserts N equals the number of files it enumerated itself. Without
#     that assertion a `.bashrsignore` silently zeroes the gate — verified:
#     a repo-root `.bashrsignore` containing `*.sh` made
#     `bashrs lint --level error sub/bad.sh` (a file with DET002 in it) print
#     `Skipped: sub/bad.sh` and exit 0, with NO receipt at all. `--no-ignore`
#     restores the finding (exit 2). Both defences are load-bearing.
#     The receipt is only emitted for N>=2, so a clean sentinel is appended to
#     the argv to guarantee it exists — for a one-script crate the receipt would
#     otherwise be absent and unassertable.
#  3. NO PIPELINE, NO xargs. `xargs bashrs lint` remaps any exit in 1..125 to
#     123, so "warnings" and "errors" become indistinguishable — the same class
#     of exit-code laundering as the PIPESTATUS problem. argv is built with a
#     read loop instead.
#
# Exit codes are overloaded and must NOT be read as a 3-value ladder:
#   0 = nothing at/above --level  OR everything was .bashrsignore'd
#   1 = warning-tier findings     OR "No lintable files found"
#   2 = error-tier findings       OR the path does not exist
# Only the receipt disambiguates. `--level` (not `--fail-on`) is what drives the
# exit code: verified, `--fail-on error` on a warning-only file still exits 1.
if command -v bashrs >/dev/null 2>&1; then
  BR_DIR="$WORKLOG/bashrs"; mkdir -p "$BR_DIR"
  # bashrs >= 6.67.0's DET002 does sink analysis: it flags a timestamp that
  # reaches a REPRODUCIBLE sink (an artifact name, a hash, a build id, a
  # truncating redirect) and deliberately does NOT flag one that only reaches
  # stdout/stderr or an append-only log — that distinction is the fix for
  # paiml/bashrs#230, adversarially reviewed before release. A sentinel that
  # only echoes a timestamp is therefore CORRECTLY silent on >=6.67.0 and is
  # not a valid positive control; the sentinel must land the timestamp in an
  # artifact's own name, which is what DET002 actually exists to catch.
  printf '#!/bin/sh\nSTAMP=$(date +%%s%%N)\ncp build.log "out/report_$STAMP.log"\n' > "$BR_DIR/dirty-sentinel.sh"
  printf '#!/bin/sh\necho ok\n'                              > "$BR_DIR/clean-sentinel.sh"

  run_split "$BR_DIR/pc.json" "$BR_DIR/pc.err" \
    bashrs lint --no-ignore --level error --format json \
      "$BR_DIR/dirty-sentinel.sh" "$BR_DIR/clean-sentinel.sh"
  BR_PC_RC=$RUN_RC
  BR_PC_HIT=1; grep -q 'DET002' "$BR_DIR/pc.json" && BR_PC_HIT=0

  if [ "$BR_PC_RC" -ne 2 ] || [ "$BR_PC_HIT" -ne 0 ]; then
    mark bashrs FAIL "POSITIVE CONTROL FAILED: a sentinel with a known DET002 did not fire (exit=$BR_PC_RC, DET002 found=$([ $BR_PC_HIT -eq 0 ] && echo yes || echo no)). bashrs is not enforcing anything here — do not read a clean result as clean."
  else
    # Enumerate the surface OURSELVES. bashrs cannot certify a non-empty scan.
    git ls-files -z '*.sh' '*.bash' 'Makefile' '*/Makefile' '**/*.sh' > "$BR_DIR/surface.z" 2>/dev/null
    BR_N=0; BR_ARGS=()
    while IFS= read -r -d '' f; do BR_ARGS+=("$f"); BR_N=$((BR_N + 1)); done < "$BR_DIR/surface.z"

    if [ "$BR_N" -eq 0 ]; then
      # Empty is only legitimate if the tree really has no shell. A .gitignore
      # that hides the shell surface from `git ls-files` is the same hole as a
      # .bashrsignore, one layer up.
      BR_HIDDEN=$(git ls-files --others --ignored --exclude-standard -- '*.sh' 2>/dev/null | head -3 | tr '\n' ' ')
      if [ -n "$BR_HIDDEN" ]; then
        mark bashrs FAIL "shell scripts exist but are INVISIBLE to \`git ls-files\` (gitignored): $BR_HIDDEN— the gate's subject is being hidden from it"
      else
        mark bashrs SKIP "0 files from: git ls-files '*.sh' '*.bash' Makefile — positive control fired, so the tool works; this tree has no shell surface"
      fi
    else
      BR_EXPECT=$((BR_N + 1))   # +1 for the clean sentinel that forces the receipt
      run_split "$BR_DIR/out.json" "$BR_DIR/out.err" \
        bashrs lint --no-ignore --level error --format json \
          "${BR_ARGS[@]}" "$BR_DIR/clean-sentinel.sh"
      BR_RC=$RUN_RC
      grep -q "Linted $BR_EXPECT file(s)" "$BR_DIR/out.err"; BR_RECEIPT=$?

      # Classify by CODE PREFIX from the JSON, not by grepping rendered text.
      # (`--format json` prepends an ANSI tracing line when exactly ONE file is
      # linted; the sentinel guarantees >=2, and the parser skips it anyway.)
      BR_CLASS=$(python3 - "$BR_DIR/out.json" <<'PY' 2>/dev/null || echo "PARSE_ERROR"
import json, sys
raw = open(sys.argv[1]).read()
i = raw.find('{')
raw = raw[i:] if i >= 0 else ''
dec, pos, gating, soft, other, rules = json.JSONDecoder(), 0, 0, 0, 0, set()
while pos < len(raw):
    while pos < len(raw) and raw[pos] in ' \t\r\n': pos += 1
    if pos >= len(raw): break
    obj, pos = dec.raw_decode(raw, pos)
    for d in obj.get('diagnostics', []):
        if d.get('severity') != 'error': continue
        c = d.get('code', '')
        if c.startswith(('SEC', 'DET', 'IDEM')): gating += 1; rules.add(c)
        elif c in ('SC1020', 'SC1035', 'SC1140'): soft += 1
        else: other += 1
print(f"{gating} {soft} {other} {' '.join(sorted(rules))}")
PY
)
      BR_GATING=$(printf '%s' "$BR_CLASS" | awk '{print $1}')
      BR_SOFT=$(printf '%s'   "$BR_CLASS" | awk '{print $2}')
      BR_OTHER=$(printf '%s'  "$BR_CLASS" | awk '{print $3}')
      BR_RULES=$(printf '%s'  "$BR_CLASS" | cut -d' ' -f4-)

      if [ "$BR_CLASS" = "PARSE_ERROR" ]; then
        mark bashrs FAIL "could not parse bashrs --format json (exit=$BR_RC) — a gate that cannot read its own tool's output has not run"
      elif [ "$BR_RECEIPT" -ne 0 ]; then
        mark bashrs FAIL "NO SCAN RECEIPT: expected \`Linted $BR_EXPECT file(s)\` on stderr, got '$(head -c 60 "$BR_DIR/out.err" | tr -d '\n')' (exit=$BR_RC) — bashrs did not lint the $BR_N file(s) enumerated; check for a .bashrsignore"
      elif [ "${BR_GATING:-1}" -eq 0 ]; then
        mark bashrs PASS "$BR_N file(s) linted (receipt confirmed), 0 SEC/DET/IDEM errors (${BR_SOFT:-0} SC10xx suppressed — bashrs#226; ${BR_OTHER:-0} other)"
      else
        mark bashrs FAIL "$BR_GATING SEC/DET/IDEM error(s) over $BR_N file(s): ${BR_RULES} — real findings, not #226 false positives"
      fi
    fi
  fi
fi

# renacer — golden tracing. Mandatory for transpilers, multi-process workflows
# and cross-language integrations, where a unit test cannot see the seam.
if command -v renacer >/dev/null 2>&1 && [ -f renacer.toml ]; then
  gate renacer renacer validate --baseline .renacer -- "$BINPATH" --version
elif [ -f renacer.toml ]; then
  mark renacer WARN "renacer.toml present but renacer not installed"
else
  mark renacer SKIP "no renacer.toml — golden tracing not configured for this crate"
fi

# pmat — the fleet's own quality gate, run against the crate under release.
# `verify` is the CI-faithful one (format, complexity, satd, clippy, tests).
# The probe asks about "$PMAT_BIN", NOT about a bare `pmat`. Those differ in
# exactly the case the pin exists for: releasing pmat itself, where PMAT_BIN is
# the freshly built artifact and PATH may hold an older pmat or none at all. A
# bare `command -v pmat` there either skips every gate below or answers about a
# binary none of them run.
if command -v "$PMAT_BIN" >/dev/null 2>&1; then
  # `pmat verify` runs cargo underneath and hard-errors on a crate with no lib
  # target: "error: no library targets found in package `pforge-cli`". That is a
  # gate that CANNOT PASS for any bin-only crate, and an unpassable gate trains
  # people to bypass the whole protocol (the same trap that had cohete's pre-push
  # running `cargo test --lib` on a bin-only crate, so every push used
  # --no-verify). So detect the absence of a lib target structurally and record
  # it as a SKIP that names the enumeration, rather than a FAIL nobody can fix.
  #
  # Note pmat-comply below still runs and still gates CB-200, so a bin-only crate
  # is NOT unmeasured — it loses one of two pmat gates, not both.
  HAS_LIB_TARGET=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
    | python3 -c '
import json,os,sys
here=os.path.realpath("Cargo.toml")
try:
    d=json.load(sys.stdin)
except Exception:
    print("unknown"); raise SystemExit
for p in d.get("packages",[]):
    if os.path.realpath(p.get("manifest_path","")) == here:
        ks=[k for t in p.get("targets",[]) for k in t["kind"]]
        print("yes" if ("lib" in ks or "rlib" in ks) else "no"); raise SystemExit
print("unknown")' 2>/dev/null)
  if [ "$HAS_LIB_TARGET" = "no" ]; then
    TGTS=$(cargo metadata --no-deps --format-version 1 2>/dev/null \
      | python3 -c '
import json,os,sys
here=os.path.realpath("Cargo.toml")
try:
    d=json.load(sys.stdin)
except Exception:
    print("?"); raise SystemExit
for p in d.get("packages",[]):
    if os.path.realpath(p.get("manifest_path","")) == here:
        print(",".join(sorted({k for t in p.get("targets",[]) for k in t["kind"]}))); raise SystemExit
print("?")' 2>/dev/null)
    mark pmat-verify SKIP "package has no lib target (kinds present: ${TGTS:-?}) — \`pmat verify\` requires one and hard-errors otherwise; pmat-comply/CB-200 below still gates this crate"
  else
    gate pmat-verify "$PMAT_BIN" verify --format json
  fi

  # ── pmat comply: gate on WHAT ACTUALLY RAN, not on the exit code ──────────
  # `pmat comply check` runs 155 checks and its exit code CANNOT SEE A SKIP.
  # Measured on a clean tiny crate: {"pass":26,"warn":13,"fail":0,"skip":116},
  # "is_compliant": true, exit 0 — and THREE of those 116 skipped checks carry
  # "severity": "Error". `--strict` does not help; it only adds a warnings
  # tri-state (exit 2), skips appear nowhere in it. So a release that runs bare
  # `comply check` prints green while the checks that matter never executed.
  #
  # The sharpest case is CB-200 (TDG Grade Gate). In a fresh `git clone` with no
  # .pmat/ it reports Skip: "Not measured: no .pmat/context.db. `comply check`
  # will not build one … Run `pmat query \"x\"` to create the index."  Measured
  # on rmedia: fresh clone {"fail":3,"skip":95} with CB-200=Skip; after ONE
  # `pmat query`, {"fail":4,"skip":93} with CB-200=Fail — "24 function(s) below
  # minimum grade A". The green was the index's absence, not the tree's quality.
  #
  # So: BUILD THE INDEX FIRST, then gate on CB-200 specifically (Skip and Fail
  # are both NO-GO — unmeasured is not a pass), and REPORT the rest. The rest
  # stays non-gating because 17 further checks want state a release checkout
  # structurally cannot have — a gitignored .pmat-work/ ticket dir, a SIBLING
  # ../provable-contracts/proof-status.json, a ledger — which is genuinely a
  # property of the workstation (paiml/paiml-mcp-agent-toolkit#1008). CB-200 is
  # NOT in that category: it is fixable with one command, so it is gated.
  run_to "$WORKLOG/pmat-index.log" timeout 900 "$PMAT_BIN" query "x" --limit 1
  PMAT_IDX_RC=$RUN_RC
  # run_SPLIT, not run_to: comply writes JSON to stdout and per-group progress
  # to stderr, so merging the two streams corrupts the JSON and the gate would
  # report "could not parse" for a run that worked perfectly.
  run_split "$WORKLOG/comply.json" "$WORKLOG/comply.err" timeout 900 "$PMAT_BIN" comply check --format json
  PMAT_COMPLY_RC=$RUN_RC
  PMAT_SUM=$(python3 - "$WORKLOG/comply.json" <<'PY' 2>/dev/null || echo "PARSE_ERROR"
import json, sys
d = json.load(open(sys.argv[1]))
s = d['summary']
checks = {c['name']: c for c in d['checks']}
cb = next((c for n, c in checks.items() if n.startswith('CB-200')), None)
dark = [n for n, c in checks.items()
        if c.get('status') == 'Skip' and c.get('severity') == 'Error']
print(f"{s['fail']} {s['skip']} {cb['status'] if cb else 'ABSENT'} {len(dark)}")
PY
)
  if [ "$PMAT_SUM" = "PARSE_ERROR" ]; then
    mark pmat-comply FAIL "could not parse \`pmat comply check --format json\` (exit=$PMAT_COMPLY_RC, index build exit=$PMAT_IDX_RC) — the fleet gate did not run"
  else
    CM_FAIL=$(printf '%s' "$PMAT_SUM" | awk '{print $1}')
    CM_SKIP=$(printf '%s' "$PMAT_SUM" | awk '{print $2}')
    CM_CB200=$(printf '%s' "$PMAT_SUM" | awk '{print $3}')
    CM_DARK=$(printf '%s' "$PMAT_SUM" | awk '{print $4}')
    case "$CM_CB200" in
      Pass)
        mark pmat-comply PASS "CB-200 measured and passing; ${CM_FAIL} other fail(s), ${CM_SKIP} skip(s) of which ${CM_DARK} are Error-severity (workstation state, #1008)" ;;
      # CB-200 is a RATCHET on a recorded baseline: `Warn` means the count is AT
      # or under it, so no NEW definition has dropped below the floor. That is a
      # GO — holding debt flat is the entire point — but it is not a clean tree,
      # and the note must carry the absolute count so nobody reads it as one.
      #
      # `Warn` rather than `Pass` is deliberate on pmat's side:
      # `retain_blocking_checks` switches on CheckStatus alone and drops `Pass`
      # unconditionally, so under `--failures-only` — which is what
      # quality-gate.yml runs — a `Pass` would have hidden the debt from the only
      # place anyone reads it. `Warn` lands in `summary.warn`, which is tallied
      # before the list is narrowed.
      #
      # `Fail` (the count went UP) and `Skip` (nothing was measured) remain
      # NO-GO below, unchanged.
      Warn)
        mark pmat-comply PASS "CB-200 at or under its recorded baseline — debt held flat, NOT a clean tree; ${CM_FAIL} other fail(s), ${CM_SKIP} skip(s) of which ${CM_DARK} are Error-severity (#1008). Run \`pmat comply check\` (without --failures-only) for the absolute count." ;;
      Skip)
        mark pmat-comply FAIL "CB-200 (TDG Grade Gate) is UNMEASURED, not passing — run \`pmat query \"x\"\` in this repo to build .pmat/context.db, then re-run. ${CM_DARK} Error-severity checks went dark; comply's own exit code (${PMAT_COMPLY_RC}) cannot see a skip." ;;
      ABSENT)
        mark pmat-comply FAIL "CB-200 absent from comply's check list — this pmat build does not run the TDG grade gate" ;;
      *)
        mark pmat-comply FAIL "CB-200 (TDG Grade Gate) = $CM_CB200 — see \`pmat comply check\`; ${CM_FAIL} total fail(s), ${CM_DARK} Error-severity checks dark (#1008, not gated)" ;;
    esac
  fi
  # reachability: code the build never compiles. New in pmat 3.32.0.
  #
  # These two lines were BARE `pmat` in BOTH copies of the runner — the
  # user-scope copy that invented PMAT_BIN never applied it here, because the
  # subcommand landed after the hardening did. Found by
  # scripts/check_verifier_pinning.sh on its first run, which is the argument
  # for the gate: the rule's own author missed a site, and only a mechanical
  # sweep of the whole file noticed. `--help` on a stale pmat also decides
  # whether the gate runs AT ALL, so an unpinned probe silently skips it.
  if "$PMAT_BIN" analyze reachability --help >/dev/null 2>&1; then
    RO=$(timeout 900 "$PMAT_BIN" analyze reachability -p . -f json 2>/dev/null | python3 -c "import sys,json;d=json.load(sys.stdin);print(d['orphan_count'],d['orphan_tests'])" 2>/dev/null || echo "? ?")
    mark reachability WARN "unreachable files/tests: $RO (a file the build never compiles cannot be tested)"
  else
    # The if-with-no-else made this row VANISH on any pmat without the
    # subcommand — violating this script's own receipt-completeness rule
    # (#2644, DF-9): a gate that vanishes reads as a gate that passed.
    mark reachability SKIP "this pmat has no \`analyze reachability\` (pre-3.32.0) — the orphan sweep did NOT run"
  fi
else
  mark pmat-verify WARN "pmat not installed — the fleet quality gate did NOT run"
fi

# ── 9. provable contracts (the sovereign differentiator) ────────────────────
#
# `make contracts` runs what this script cannot generically replicate — the Lean
# (L4) proofs and the falsification suites. It is NOT the authority on the pv
# layer: pv-contracts / pv-lint / pv-bindings above run pv directly, so their
# exit codes are ours.
#
# EXIT-CODE INTEGRITY, checked statically on the recipe. A POSIX `for` loop exits
# with its LAST iteration's status, so
#     @for c in contracts/*.yaml; do pv validate "$c"; done
# reports whatever the alphabetically-last contract did. That is rmedia's actual
# recipe; running its body against rmedia's real contracts exits 0 with 17 of 47
# contracts FAILING validation, because the last name in the glob
# (visual-quality-v1.yaml) passes — the single largest false GREEN found in this
# fleet. The same recipe's second line, `pv lint contracts/binding.yaml
# 2>/dev/null || true`, is dead twice over: `pv lint` on a FILE passes over zero
# contracts, and `|| true` would swallow it anyway.
# Correct shape (copia's): `do pv validate "$$c" || exit 1; done`.
# The Makefile is discovered like the CHANGELOG is: a workspace member keeps
# its Makefile at the repo root, and looking only in the crate dir silently
# degraded the sovereign differentiator to a non-gating WARN for every member
# (#2644, DF-5 — changelog and coverage were fixed this way; this gate was not).
# First Makefile that DECLARES the target, not the first that exists: a member
# Makefile without `contracts:` must not shadow the root one that has it
# (residual found verifying DF-5).
MKPATH=""
for mk_cand in Makefile "$REPO_ROOT/Makefile"; do
  if [ -f "$mk_cand" ] && grep -qE '^contracts:' "$mk_cand" 2>/dev/null; then
    MKPATH="$mk_cand"; break
  fi
done
if [ -n "$MKPATH" ]; then
  MK_RECIPE=$(awk '/^contracts:/{f=1;next} /^[^\t]/{f=0} f' "$MKPATH")
  MK_SMELL=""
  printf '%s' "$MK_RECIPE" | grep -qE 'for .*; *do' \
    && ! printf '%s' "$MK_RECIPE" | grep -qE '\|\| *exit' \
    && MK_SMELL="a for-loop with no \`|| exit\` (exits with its LAST iteration)"
  printf '%s' "$MK_RECIPE" | grep -qE '\|\| *true' \
    && MK_SMELL="$MK_SMELL${MK_SMELL:+; }\`|| true\` swallows a real failure"
  if [ -n "$MK_SMELL" ]; then
    mark contracts-exit-integrity FAIL "the \`contracts:\` recipe launders its exit code: $MK_SMELL. Its GREEN cannot be believed — fix the recipe before trusting it."
  else
    mark contracts-exit-integrity PASS "\`contracts:\` recipe propagates failure (no bare for-loop, no \`|| true\`)"
  fi
  gate contracts make -C "$(dirname "$MKPATH")" contracts
else
  mark contracts WARN "no contracts make target"
fi

# ── 10. publish dry-run (already run above; verdict is its exit code) ───────
if [ $DRC -eq 0 ]; then mark publish-dry-run PASS "packages cleanly"
else mark publish-dry-run FAIL "$(printf '%s' "$DRY" | grep -iE 'error' | head -1)"; fi

# ── 11. DOGFOOD: use the crate's own release binary on real data ────────────
# The repo supplies scripts/dogfood-use.sh (given $BIN = built release binary +
# $WORK = a scratch dir). It must exit non-zero if the tool misbehaves on real data.
# Native gate first. It lives with the code, is versioned with it, is itself
# unit-tested, and can enumerate its own surface exhaustively — a shell script
# beside the repo cannot do any of that, and the one forjar had covered only
# `file` resources while two new resource types shipped broken.
# (The release binary was built near the top; $BINPATH is set there.)

# ── RELEASE INTERFACE VERIFICATION (CLI / HTTP / MCP) ───────────────────────
#
# WHY THIS IS NOT probador. probador 1.0.3 is "Playwright-Compatible Testing for
# WASM + TUI Applications". Its subcommands are: test, record, report, coverage,
# init, config, serve, build, watch, playbook, comply, av-sync, audio, video,
# animation, stress, llm. NONE of them drives another process's CLI, an
# arbitrary HTTP endpoint, or a JSON-RPC/MCP stdio server:
#   · `serve` is a DEV SERVER for the WASM app under test (axum + extract::ws,
#     hot reload, COOP/COEP headers) — it hosts a page, it is not an HTTP client.
#   · `llm` IS a real HTTP client, and `llm bench --start <cmd>` even has the
#     right shape, but the protocol is hardwired: crates/probar/src/llm/client.rs
#     builds `{base_url}/v1/chat/completions` and POSTs a typed ChatRequest.
#     There is no way to say "POST /v1/<verb> with this body and diff the bytes".
#   · MCP is INERT: `jsonrpc` appears in exactly two files, both
#     generated_contracts.rs, and the generated `contract_jsonrpc_framing!`
#     macros have ZERO call sites in probar or probar-cli. It has the
#     vocabulary, not the capability.
#   · `probador comply` is vacuous off-WASM: on a plain 4-line Rust CLI crate
#     with no WASM, no HTML and no browser it scored 8/10, passing "Custom
#     elements tested", "Threading modes tested", "WASM size limit". Gating on
#     that would be exactly the defect this protocol exists to catch.
# So probador's honest contribution to a Rust CLI release today is its own
# suite where a crate configures it, and NOTHING for CLI/HTTP/MCP parity. That
# gap is named in SKILL.md as work, not papered over with an invocation that
# would pass without verifying anything.
#
# WHAT ENFORCES IT INSTEAD — cargo, which is already here, in three gates:
#
# (a) cli-surface: enumerate the subcommands the binary ADVERTISES in its own
#     --help and require each to answer `--help` with exit 0. Catches a release
#     shipping a subcommand that panics, was renamed, or is listed but
#     unimplemented. Read from the artifact, so it cannot drift.
#
# (b) transport-decl + interface-parity: every non-CLI transport must be
#     DECLARED in Cargo.toml, and every declared transport must have an e2e test
#     target that SPAWNS THE SHIPPED BINARY and passes. This exists because
#     agreement cannot falsify reachability: rmedia's four-way parity suite was
#     GREEN for the whole period `mcp::serve_stdio` and `http::serve` had no
#     caller from main.rs (GH-247). The transports agreed with each other
#     perfectly and were unreachable from the process entry point.
#
#     The `--test <target>` naming is the load-bearing detail. Verified:
#       cargo test --test e2e_http_t              (feature not enabled)
#         → error: target `e2e_http_t` … requires the features: `http`, EXIT=101
#       cargo test                                (bare)
#         → EXIT=0, and the target is not mentioned ONCE in the output.
#     Naming the target converts "absent" into exit 101. A bare `cargo test`
#     silently EXCLUDES every required-features target, which is a gate-shaped
#     silence.
#
# (c) undeclared-transport absence: if the binary advertises an `mcp`/`serve`
#     subcommand that no declaration covers, that is a FAIL. An undeclared
#     transport is an unverified one, and this turns "we have no HTTP surface"
#     from an assumption into an enforced fact.
if [ -x "$BINPATH" ]; then
  CLI_HELP=$("$BINPATH" --help 2>&1); CLI_HELP_RC=$?
  SUBS=$(printf '%s\n' "$CLI_HELP" \
    | awk '/[Cc]ommands:/{f=1;next} /^[A-Za-z].*:[[:space:]]*$/{f=0} f' \
    | grep -oE '^[[:space:]]+[a-z][a-z0-9_-]+' | tr -d ' ' | sort -u)
  if [ "$CLI_HELP_RC" -ne 0 ]; then
    # `|| true` used to discard this exit; a binary that CRASHES on --help
    # then read as "advertises no subcommands" — a SKIP cascading into
    # transport-absence PASSing over an empty surface (#2644, DF-4).
    mark cli-surface FAIL "the release binary cannot answer --help (exit=$CLI_HELP_RC) — the advertised surface is unknowable: $(tail -1 <<< "$CLI_HELP" | strip_ansi | cut -c1-100)"
  elif [ -z "$SUBS" ]; then
    mark cli-surface SKIP "binary advertises no subcommands in --help (--help exit 0; Commands section empty or absent)"
  else
    CLI_BAD=""
    CLI_N=0
    for sub in $SUBS; do
      case "$sub" in help) continue ;; esac
      CLI_N=$((CLI_N + 1))
      "$BINPATH" "$sub" --help >/dev/null 2>&1 || CLI_BAD="$CLI_BAD $sub"
    done
    if [ -n "$CLI_BAD" ]; then
      mark cli-surface FAIL "advertised but unusable:$CLI_BAD (of $CLI_N checked)"
    else
      mark cli-surface PASS "$CLI_N advertised subcommand(s) answer --help"
    fi
  fi
else
  mark cli-surface FAIL "no release binary at $BINPATH — nothing to verify"
  SUBS=""
fi

# ── transport declaration + interface parity ────────────────────────────────
# Declaration lives in Cargo.toml so it is versioned with the code it describes:
#
#   [package.metadata.transports]
#   cli  = { e2e = "e2e_cli_t" }
#   mcp  = { e2e = "e2e_mcp_stdio_t", features = ["mcp"] }
#   http = { e2e = "e2e_http_serve_t", features = ["http", "lua"] }
#
# Each named target must (1) exist, (2) reference CARGO_BIN_EXE_ — i.e. spawn
# the SHIPPED BINARY rather than call the library, which is the reachability
# property a library-level parity suite structurally cannot see — and (3) run
# at least one test that passes. "0 tests, ok" is a vacuous pass and FAILs here.
run_to "$WORKLOG/meta.json" cargo metadata --no-deps --format-version 1
META_RC=$RUN_RC
TP_PLAN=$(CRATE="$CRATE" python3 - "$WORKLOG/meta.json" <<'PY' 2>/dev/null || echo "META_ERROR"
import json, os, sys
d = json.load(open(sys.argv[1]))
name = os.environ.get('CRATE', '')
pkgs = d.get('packages', [])
pkg = next((p for p in pkgs if p['name'] == name), None)
if pkg is None and len(pkgs) == 1:
    pkg = pkgs[0]
if pkg is None:
    print("NOPKG " + ",".join(sorted(p['name'] for p in pkgs))); raise SystemExit
tp = (pkg.get('metadata') or {}).get('transports')
tests = {t['name']: t['src_path'] for t in pkg['targets'] if 'test' in t['kind']}
if not tp:
    print("NODECL"); raise SystemExit
man = tp.get('manifest')
if isinstance(man, dict):
    print(f"MANIFEST {man.get('path') or '-'} {man.get('regen') or '-'}")
for k, v in sorted(tp.items()):
    if k == 'manifest':
        continue
    if isinstance(v, bool):
        print(f"BADSHAPE {k}"); continue
    tgt = (v or {}).get('e2e', '')
    feats = ",".join((v or {}).get('features', []))
    print(f"DECL {k} {tgt or '-'} {feats or '-'} {tests.get(tgt, '-')}")
PY
)
TP_BLOCKED=""
if [ "$TP_PLAN" = "META_ERROR" ] || [ "$META_RC" -ne 0 ]; then
  mark transport-decl FAIL "\`cargo metadata\` failed (exit=$META_RC) — the transport declaration could not be read"
  TP_BLOCKED="cargo metadata failed"
elif [ "${TP_PLAN%% *}" = "NOPKG" ]; then
  mark transport-decl FAIL "no package named '$CRATE' in cargo metadata (found: ${TP_PLAN#NOPKG }) — run dogfood from the crate dir, not the virtual workspace root"
  TP_BLOCKED="package not found in cargo metadata"
elif [ "$TP_PLAN" = "NODECL" ]; then
  mark transport-decl FAIL "no [package.metadata.transports] in Cargo.toml. Declare every interface this release ships, e.g.  [package.metadata.transports]  cli = { e2e = \"e2e_cli_t\" }  mcp = { e2e = \"e2e_mcp_stdio_t\", features = [\"mcp\"] }. An undeclared transport is an unverified one."
  TP_BLOCKED="no [package.metadata.transports] declaration"
fi
if [ -n "$TP_BLOCKED" ]; then
  # Both downstream gates still appear in the receipt, each naming what blocked
  # it. A gate that vanishes from the receipt reads as one that passed.
  mark transport-absence  SKIP "not evaluated: $TP_BLOCKED (transport-decl is RED above)"
  mark interface-parity   SKIP "not evaluated: $TP_BLOCKED (transport-decl is RED above)"
else
  TP_NAMES=$(printf '%s\n' "$TP_PLAN" | awk '$1=="DECL"{print $2}' | tr '\n' ' ')
  TP_BAD=$(printf '%s\n' "$TP_PLAN" | awk '$1=="BADSHAPE"{print $2}' | tr '\n' ' ')
  if [ -n "$TP_BAD" ]; then
    mark transport-decl FAIL "transport(s) declared as a bare bool: $TP_BAD — each needs { e2e = \"<test target>\", features = [...] }; a declaration with no e2e target verifies nothing"
  else
    mark transport-decl PASS "declared: $TP_NAMES"
  fi

  # (c) absence of UNDECLARED transports, probed on the real binary.
  if [ "${CLI_HELP_RC:-0}" -ne 0 ]; then
    mark transport-absence SKIP "not evaluated: the binary could not answer --help (cli-surface is RED above), so the advertised surface is unknown — asserting absence over an unreadable surface is a vacuous pass (#2644, DF-4)"
    TP_INTRUDERS=""
  else
  TP_INTRUDERS=""
  for probe in mcp serve http api rpc; do
    grep -qx "$probe" <<< "$SUBS" || continue
    # `serve` names no protocol. A subcommand called `serve` may be an HTTP
    # listener OR an MCP stdio server — pforge's is the latter — so demanding an
    # `http` declaration for it forces a crate to declare a transport it does
    # not ship. A false declaration is worse than a missing one: it makes the
    # gate assert an interface nobody can exercise.
    #
    # So `serve` is satisfied by EITHER mcp or http, while the protocol-specific
    # names still demand their own.
    case "$probe" in
      mcp|rpc)   want=mcp ;;
      http|api)  want=http ;;
      serve)     want="mcp http" ;;
    esac
    satisfied=""
    for w in $want; do
      printf '%s' "$TP_NAMES" | grep -qw "$w" && satisfied=yes
    done
    [ -n "$satisfied" ] || TP_INTRUDERS="$TP_INTRUDERS $probe(→${want// //})"
  done
  if [ -n "$TP_INTRUDERS" ]; then
    mark transport-absence FAIL "binary advertises undeclared transport surface:$TP_INTRUDERS — declare it in [package.metadata.transports] with an e2e target, or remove it. An undeclared transport is an unverified one."
  else
    mark transport-absence PASS "no undeclared mcp/http surface advertised by the binary"
  fi
  fi

  # (b) run each declared transport's e2e target, naming it with --test.
  IP_BAD=""; IP_OK=0; IP_TESTS=0
  while read -r kind tname ttarget tfeats tsrc; do
    [ "$kind" = "DECL" ] || continue
    if [ "$ttarget" = "-" ]; then
      IP_BAD="$IP_BAD $tname(no e2e target declared)"; continue
    fi
    if [ "$tsrc" = "-" ]; then
      IP_BAD="$IP_BAD $tname(target '$ttarget' does not exist)"; continue
    fi
    # Reachability: the target must spawn the shipped binary, not the library.
    if ! grep -q 'CARGO_BIN_EXE_' "$tsrc" 2>/dev/null; then
      IP_BAD="$IP_BAD $tname(e2e '$ttarget' never references CARGO_BIN_EXE_ — it exercises the library, not the release artifact)"
      continue
    fi
    IP_FEATARG=()
    [ "$tfeats" != "-" ] && IP_FEATARG=(--features "$(printf '%s' "$tfeats" | tr ',' ' ')")
    run_to "$WORKLOG/iface-$tname.log" \
      cargo test -p "$CRATE" "${IP_FEATARG[@]+"${IP_FEATARG[@]}"}" --test "$ttarget"
    IP_RC=$RUN_RC
    IP_PASSED=$(grep -oE 'test result: ok\. [0-9]+ passed' "$WORKLOG/iface-$tname.log" \
                | grep -oE '[0-9]+' | awk '{s+=$1} END{print s+0}')
    if [ "$IP_RC" -ne 0 ]; then
      IP_BAD="$IP_BAD $tname(exit=$IP_RC)"
    elif [ "${IP_PASSED:-0}" -eq 0 ]; then
      IP_BAD="$IP_BAD $tname(target ran 0 tests — a vacuous pass)"
    else
      IP_OK=$((IP_OK + 1)); IP_TESTS=$((IP_TESTS + IP_PASSED))
    fi
  done <<< "$TP_PLAN"
  if [ -n "$IP_BAD" ]; then
    mark interface-parity FAIL "declared transport(s) not verified:$IP_BAD — each must have an e2e target that spawns the release binary and passes (see rmedia e2e_mcp_stdio_t / e2e_http_serve_t)"
  else
    mark interface-parity PASS "$IP_OK transport(s), $IP_TESTS e2e test(s) against the spawned release binary"
  fi
fi

# ── transport invariance: every transport LIVE AT ONCE, verbs DERIVED ───────
# interface-parity proves each transport is reachable and green in its own e2e.
# Necessary, not sufficient: three transports can each pass their own test file
# and still disagree about what a verb RETURNS, because nothing compares them.
# This gate invokes the SAME verb through every declared transport while all of
# them are standing, and requires byte-identical results.
#
# Two properties, both deliberate:
#
#   DERIVED, not hand-written. The verb list comes from the BINARY, never from a
#   probe list in this script. A hand-written probe tests the verbs someone
#   remembered; a derived one tests the surface that shipped, and grows when the
#   surface grows. It FAILS when the binary lists nothing, because a parity
#   check over an empty surface is vacuously true.
#
#   SIMULTANEOUS, not sequential. One transport at a time cannot distinguish
#   "they agree" from "they share a process-global only one may hold at a time".
#   All of them up together is what a real client fleet produces, and what
#   surfaces a shared listener, lock, or runtime.
#
# Declare it in Cargo.toml beside [package.metadata.transports]; see SKILL.md.
#
# NOT a pipeline: `python3 ... | head` reports head's status, and this gate's
# whole value is its own exit code. run_to keeps the command's status in
# $RUN_RC — see run_to's comment and the two false GREENs that earned it.
UI_DECL=$(CRATE="$CRATE" python3 - "$WORKLOG/meta.json" <<'UIPY' 2>/dev/null || echo ""
import json, os, sys
try:
    d = json.load(open(sys.argv[1]))
    pkg = next((p for p in d["packages"] if p["name"] == os.environ["CRATE"]), None)
    us = ((pkg or {}).get("metadata") or {}).get("unified_surface")
    print(json.dumps(us) if us else "")
except Exception:
    print("")
UIPY
)
if [ ! -x "$BINPATH" ]; then
  mark transport-invariance FAIL "no release binary at $BINPATH — nothing to invoke"
elif [ -z "$UI_DECL" ]; then
  mark transport-invariance SKIP "no [package.metadata.unified_surface] — declare list/cli/http/probe to enable the simultaneous cross-transport check"
elif [ ! -f "$SKILL_DIR/invariance.py" ]; then
  mark transport-invariance FAIL "invariance.py missing from $SKILL_DIR — a gate that cannot run is not a SKIP"
else
  run_to "$WORKLOG/invariance.log" python3 "$SKILL_DIR/invariance.py" "$BINPATH" "$UI_DECL"
  INV_RC=$RUN_RC
  INV_MSG=$(head -1 "$WORKLOG/invariance.log" 2>/dev/null || echo "")
  case "$INV_RC:$INV_MSG" in
    0:INVARIANCE_SKIP*) mark transport-invariance SKIP "${INV_MSG#INVARIANCE_SKIP }" ;;
    0:INVARIANCE_PASS*) mark transport-invariance PASS "${INV_MSG#INVARIANCE_PASS }" ;;
    0:*)                mark transport-invariance FAIL "checker exited 0 with no verdict line — an unverdicted run is a failure, never a pass: $INV_MSG" ;;
    1:*)                mark transport-invariance FAIL "$(sed -n '1,3p' "$WORKLOG/invariance.log" | tr '\n' ' ')" ;;
    *)                  mark transport-invariance FAIL "checker error (exit $INV_RC): $INV_MSG" ;;
  esac
fi

# (e) surface-derivation — is the interface DERIVED from one source, or three
# hand-maintained lists that happen to agree today?
#
# interface-parity above proves each transport RUNS. It cannot prove they are
# projections of a single registry rather than parallel hand-written surfaces,
# and agreement is not single-sourcing: three lists edited by the same person in
# the same hour agree perfectly and drift next week. rmedia's four-way parity
# suite was GREEN for the whole period its MCP and HTTP transports had no caller
# (paiml/rmedia#247) — the surfaces agreed and were unreachable.
#
# The check: the crate declares a manifest and the generator that emits it. The
# gate REGENERATES it and requires the tree to be unchanged. A hand-maintained
# list cannot survive that — either the generator does not exist, or
# regenerating rewrites the file. Docs as a projection make drift
# unrepresentable; a manifest maintained BESIDE the code is the drift.
#
#   [package.metadata.transports]
#   manifest = { path = "docs/surface.json", regen = "regenerate_surface" }
#
# Declaring no manifest is a REPORT, not a FAIL: a crate whose CLI is still
# hand-rolled is in a known, common state, and failing it here would make this
# gate unpassable for every crate that has not migrated yet — which is how a
# gate gets bypassed wholesale. It becomes a FAIL once a manifest IS declared
# and does not regenerate cleanly, because that is a broken claim rather than an
# absent one.
if [ -n "${TP_PLAN:-}" ]; then
  SD_PATH=$(printf '%s\n' "$TP_PLAN" | awk '$1=="MANIFEST"{print $2}' | head -1)
  SD_REGEN=$(printf '%s\n' "$TP_PLAN" | awk '$1=="MANIFEST"{print $3}' | head -1)
  if [ -z "$SD_PATH" ] || [ "$SD_PATH" = "-" ]; then
    mark surface-derivation REPORT "no manifest declared in [package.metadata.transports] — the interface may be hand-rolled per transport; nothing here can tell single-sourcing from three lists that agree (declare manifest = { path, regen } to gate it)"
  elif [ ! -f "$SD_PATH" ]; then
    mark surface-derivation FAIL "declared manifest '$SD_PATH' does not exist — a declared surface that is not committed cannot be diffed against a regeneration"
  else
    SD_BEFORE=$(sha256sum "$SD_PATH" | cut -d" " -f1)
    run_to "$WORKLOG/surface-regen.log" \
      cargo test -p "$CRATE" --all-targets "$SD_REGEN" -- --ignored
    SD_RC=$RUN_RC
    SD_AFTER=$(sha256sum "$SD_PATH" | cut -d" " -f1)
    # A generator that matched NO test leaves the file untouched and exits 0,
    # so before==after would read as "regenerates identically" when nothing
    # regenerated at all. Verified on forjar: `cargo test -p forjar <name>`
    # without --all-targets printed `0 passed; 14 filtered out`, exit 0, hashes
    # equal — a vacuous PASS on a manifest that could be arbitrarily stale.
    # interface-parity already guards this shape; this gate must too.
    SD_RAN=$(grep -oE 'test result: ok\. [0-9]+ passed' "$WORKLOG/surface-regen.log" \
             | grep -oE '[0-9]+' | awk '{s+=$1} END{print s+0}')
    if [ "$SD_RC" -ne 0 ]; then
      mark surface-derivation FAIL "manifest generator '$SD_REGEN' failed (exit=$SD_RC) — the surface cannot be regenerated, so it is maintained by hand"
    elif [ "${SD_RAN:-0}" -eq 0 ]; then
      mark surface-derivation FAIL "manifest generator '$SD_REGEN' matched NO test (0 passed) — an unchanged file proves nothing when nothing ran; check the test name and that it is #[ignore]d"
    elif [ "$SD_BEFORE" != "$SD_AFTER" ]; then
      mark surface-derivation FAIL "regenerating '$SD_PATH' CHANGED it — the committed surface was stale, so at least one transport has drifted from the registry"
    else
      mark surface-derivation PASS "'$SD_PATH' regenerates byte-identical via '$SD_REGEN' ($SD_RAN test(s) ran) — the surface is a projection, not a parallel list"
    fi
  fi
fi

# probador — its OWN suite, where the crate configures it. This is all probador
# can gate for a Rust CLI today; see the block above for why it is not the
# CLI/HTTP/MCP gate. Missing CONFIGURATION is a SKIP with its reason; a missing
# TOOL is already a NO-GO in deterministic-tools.
if command -v probador >/dev/null 2>&1; then
  if [ -f probar.toml ] || [ -f probador.toml ] || grep -q 'jugar-probar' Cargo.toml 2>/dev/null; then
    gate probador-suite probador test
  else
    mark probador-suite SKIP "crate does not configure probador (no probar.toml, no jugar-probar dep) — probador tests WASM/TUI apps; it has no CLI/HTTP/MCP interface verb (see interface-parity)"
  fi
fi

NATIVE_DOGFOOD=""
if [ -n "$BINPATH" ] && [ -x "$BINPATH" ] && "$BINPATH" dogfood --help >/dev/null 2>&1; then
  NATIVE_DOGFOOD="yes"
fi

if [ -z "$BINPATH" ] || [ ! -x "$BINPATH" ]; then
  mark dogfood-use FAIL "could not build the release binary"
elif [ -n "$NATIVE_DOGFOOD" ]; then
  gate dogfood-use "$BINPATH" dogfood
elif [ -f scripts/dogfood-use.sh ] || [ -f "$(git rev-parse --show-toplevel 2>/dev/null)/scripts/dogfood-use.sh" ]; then
  # A workspace member legitimately shares repo-level scripts. Looking only in
  # the crate dir reported "no dogfood gate" for a crate that HAS one three
  # directories up, which is a false NO-GO — and a gate that is red for the
  # wrong reason is one people learn to bypass.
  if [ -f scripts/dogfood-use.sh ]; then
    DF_SCRIPT="scripts/dogfood-use.sh"
  else
    DF_SCRIPT="$(git rev-parse --show-toplevel)/scripts/dogfood-use.sh"
  fi
  WORK=$(mktemp -d)
  gate dogfood-use env BIN="$BINPATH" WORK="$WORK" bash "$DF_SCRIPT"
  rm -rf "$WORK"
else
  mark dogfood-use FAIL "no dogfood gate — add a native \`<bin> dogfood\` subcommand (preferred) or scripts/dogfood-use.sh. A released tool nobody ran is not dogfooded, and a WARN here is a step everyone learns to walk past."
fi

# ── clean-room reminder (heavy, runs on the CI box; not automated here) ─────
mark clean-room MANUAL "run \`make -C ../infra/machines/clean-room clean-room-$CRATE\` (MANDATORY release gate)"

# ── receipt + verdict ───────────────────────────────────────────────────────
#
# The receipt is the artifact this protocol tells you to attach to the release
# as its evidence, so it has to survive being read back.
#
# It did not. Notes were escaped with `sed 's/"/\\"/g'`, which handles the quote
# and nothing else — no newline, no backslash, no tab, no control character.
# `pmat verify --format json` embeds a multi-line JSON blob in its note, and
# that one note made the whole receipt unparseable:
#
#   json.decoder.JSONDecodeError: Expecting ',' delimiter: line 22 column 147
#
# An unreadable receipt is the same defect this protocol exists to catch: a
# file that exists, looks like evidence, and answers no question. Encoded with
# a real JSON encoder now, and the result is parsed back before the script will
# claim it wrote one.
NAMES_TSV=$(for i in "${!NAMES[@]}"; do
  printf '%s\t%s\t%s\n' "${NAMES[$i]}" "${RESULTS[$i]}" "$(printf '%s' "${NOTES[$i]}" | tr '\n' '\r')"
done)
CRATE="$CRATE" VERSION="$VERSION" TS="$TS" SHA="$RECEIPT_SHA" \
VERDICT="$([ $FAILED -eq 0 ] && echo GO || echo NO-GO)" \
ROWS="$NAMES_TSV" python3 > "$RECEIPT_PARTIAL" <<'PY'
import json, os
gates = []
for line in os.environ.get("ROWS", "").split("\n"):
    if not line.strip():
        continue
    parts = line.split("\t")
    name, result = parts[0], parts[1] if len(parts) > 1 else ""
    # rejoin: a literal TAB inside a gate note (log tails carry them) used to
    # silently drop everything after it (#2644, DF-10)
    note = "\t".join(parts[2:]).replace("\r", "\n") if len(parts) > 2 else ""
    gates.append({"gate": name, "result": result, "note": note})
print(json.dumps({
    "crate": os.environ["CRATE"], "version": os.environ["VERSION"],
    "timestamp": os.environ["TS"], "commit": os.environ["SHA"], "gates": gates,
    "verdict": os.environ["VERDICT"],
}, indent=2))
PY
if ! python3 -c 'import json,sys; json.load(open(sys.argv[1]))' "$RECEIPT_PARTIAL" 2>/dev/null; then
  echo "FATAL: the receipt this run just wrote is not valid JSON ($RECEIPT_PARTIAL)." >&2
  echo "       Refusing to report a verdict backed by evidence that cannot be read." >&2
  echo "       The .partial is kept as-is; no completed receipt exists for this run." >&2
  exit 3
fi
# Atomic completion: the receipt EXISTS only once it is whole and parseable.
mv "$RECEIPT_PARTIAL" "$RECEIPT"

echo "────────────────────────────────────────────────"
echo "receipt: $RECEIPT"
if [ $FAILED -eq 0 ]; then
  echo "VERDICT: ✅ GO — all automated gates green. Complete clean-room (MANDATORY) then release."
  exit 0
else
  # Name every failing gate and its note. A verdict that says only "a gate
  # failed" makes the reader hunt for it, and the hunt is where bypasses start.
  echo "NO-GO — these gates are RED (fix the root cause; never bypass):"
  for i in "${!NAMES[@]}"; do
    [ "${RESULTS[$i]}" = FAIL ] || continue
    printf '  · %-24s %s\n' "${NAMES[$i]}" "${NOTES[$i]}"
  done
  echo "VERDICT: ❌ NO-GO — fix the ROOT CAUSE (Toyota way), never bypass. Re-run dogfood."
  exit 1
fi
