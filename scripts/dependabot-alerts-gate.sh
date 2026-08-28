#!/usr/bin/env bash
# Blocking security gate over GitHub's Dependabot alerts (#1074).
#
# WHY THIS EXISTS ALONGSIDE `cargo deny`, NOT INSTEAD OF IT.
#
# `cargo deny check advisories` resolves against the RustSec database. GitHub's
# advisory database is a SUPERSET: it ingests RustSec *and* GHSA-native
# advisories, CVEs mapped to crates, and maintainer-reported issues never filed
# with RustSec. So `cargo deny` has a permanent, structural blind spot — and it
# is silent about it. Measured on this repository at 3.32.0:
#
#   $ cargo deny check advisories
#   advisories ok                                   exit=0
#
#   $ gh api .../dependabot/alerts?state=open
#   #66  medium  thrift  GHSA-2f9f-gq7v-9h6m  < 0.23.0  patched: 0.23.0
#
# Both read the same `Cargo.lock`. The one that blocked CI said clean. There is
# no "N advisories consulted, M sources unavailable" line anywhere in cargo-deny
# output: `advisories ok` is printed identically whether the database is
# comprehensive, stale, or empty. That is absence rendered as success, sitting in
# the security gate.
#
# The two sources genuinely differ and neither subsumes the other — RustSec
# carries unmaintained/yanked findings Dependabot does not model — so this runs
# BESIDE `cargo deny`, and both must pass.
#
# THIS GATE FAILS WHEN IT CANNOT MEASURE. A missing `gh`, an API error, a
# missing token, Dependabot disabled on the repository, or an unparseable body
# are all FAILURES. "We could not look" must never render as "nothing found",
# which is the exact defect being fixed.
#
# Usage:
#   scripts/dependabot-alerts-gate.sh                 # gate this repository
#   THRESHOLD=high scripts/dependabot-alerts-gate.sh  # only high/critical block
#   scripts/dependabot-alerts-gate.sh --fixture F     # read alerts from F, not the API
#   scripts/dependabot-alerts-gate.sh --self-test     # prove the gate can fail
set -euo pipefail

THRESHOLD="${THRESHOLD:-medium}"
ACK_FILE="${ACK_FILE:-.github/dependabot-acknowledged.txt}"
FIXTURE=""
SELF_TEST=0

while [ $# -gt 0 ]; do
    case "$1" in
        --fixture)   FIXTURE="${2:?--fixture needs a path}"; shift 2 ;;
        --self-test) SELF_TEST=1; shift ;;
        --threshold) THRESHOLD="${2:?--threshold needs a level}"; shift 2 ;;
        -h|--help)   sed -n '1,44p' "$0"; exit 0 ;;
        *) echo "unknown argument: $1" >&2; exit 2 ;;
    esac
done

# Both streams on purpose: `::error::` on stdout so GitHub annotates the run, and
# a plain line on stderr so the message survives when this function is called
# from inside a command substitution (which captures stdout).
die() { echo "::error::$*" >&2; echo "SECURITY GATE FAILED: $*" >&2; exit 1; }

# Rank severities so a threshold is a comparison rather than a list to keep in
# sync. Anything GitHub reports that is not in this table is treated as ABOVE
# the threshold: an unrecognised severity is not a reason to pass.
severity_rank() {
    case "$1" in
        low) echo 1 ;;
        medium) echo 2 ;;
        high) echo 3 ;;
        critical) echo 4 ;;
        *) echo 99 ;;
    esac
}

THRESHOLD_RANK="$(severity_rank "$THRESHOLD")"
if [ "$THRESHOLD_RANK" = "99" ]; then
    die "THRESHOLD='$THRESHOLD' is not one of low|medium|high|critical"
fi

# ---------------------------------------------------------------------------
# Fetch
# ---------------------------------------------------------------------------
fetch_alerts() {
    if [ -n "$FIXTURE" ]; then
        [ -f "$FIXTURE" ] || die "fixture '$FIXTURE' does not exist, so nothing was measured"
        cat "$FIXTURE"
        return
    fi

    command -v gh >/dev/null 2>&1 || die "\`gh\` is not on PATH, so Dependabot alerts could not be read. A missing tool is not a clean result."

    local repo="${GATE_REPO:-}"
    if [ -z "$repo" ]; then
        repo="${GITHUB_REPOSITORY:-}"
    fi
    if [ -z "$repo" ]; then
        repo="$(gh repo view --json nameWithOwner -q .nameWithOwner 2>/dev/null || true)"
    fi
    [ -n "$repo" ] || die "could not determine the repository (set GATE_REPO or GITHUB_REPOSITORY)"

    # `--paginate` is load-bearing: `gh api` returns 30 items per page by
    # default, so a repository with more alerts than one page reports a
    # TRUNCATED list that looks complete. #1074 names this explicitly.
    local out
    if ! out="$(gh api --paginate "repos/$repo/dependabot/alerts?state=open&per_page=100" 2>&1)"; then
        case "$out" in
            *"Dependabot alerts are disabled"*|*"403"*)
                die "Dependabot alerts are disabled or inaccessible on $repo: $out" ;;
            *)
                die "the Dependabot alerts API call failed for $repo: $out" ;;
        esac
    fi
    printf '%s' "$out"
}

# ---------------------------------------------------------------------------
# Acknowledgements — a recorded decision with an expiry, never a deleted check
#
# Mirrors `deny.toml`'s `[advisories] ignore`, with one thing deny.toml lacks:
# every entry MUST carry an expiry date, and an expired entry FAILS rather than
# silently continuing to suppress. An allowlist with no expiry is how it quietly
# becomes broader than anyone intended.
#
# Format, one per line:  GHSA-xxxx-xxxx-xxxx  YYYY-MM-DD  reason text
# ---------------------------------------------------------------------------
acknowledged_until() {
    [ -f "$ACK_FILE" ] || return 1
    awk -v id="$1" '
        /^[[:space:]]*#/ { next }
        /^[[:space:]]*$/ { next }
        $1 == id { print $2; found=1; exit }
        END { exit(found ? 0 : 1) }
    ' "$ACK_FILE"
}

TODAY="$(date -u +%Y-%m-%d)"

# ---------------------------------------------------------------------------
# Evaluate
# ---------------------------------------------------------------------------
evaluate() {
    local body="$1"
    local total blocking=0 expired=0 suppressed=0

    total="$(printf '%s' "$body" | python3 -c '
import json,sys
try:
    d = json.load(sys.stdin)
except Exception as e:
    print("PARSE_ERROR", e); sys.exit(0)
print(len(d) if isinstance(d, list) else "PARSE_ERROR not-a-list")
')"
    case "$total" in
        PARSE_ERROR*) die "the alerts payload could not be parsed ($total), so nothing was measured" ;;
    esac

    echo "Dependabot: ${total} open alert(s) fetched; blocking threshold is '${THRESHOLD}'"

    if [ "$total" -eq 0 ]; then
        echo "✅ no open Dependabot alerts"
        return 0
    fi

    # One line per alert: ghsa<TAB>severity<TAB>package<TAB>range<TAB>patched
    local rows
    rows="$(printf '%s' "$body" | python3 -c '
import json,sys
for a in json.load(sys.stdin):
    adv = a.get("security_advisory") or {}
    vul = a.get("security_vulnerability") or {}
    pkg = (vul.get("package") or {}).get("name", "?")
    patched = (vul.get("first_patched_version") or {}).get("identifier", "none")
    print("\t".join([
        adv.get("ghsa_id", "?"),
        adv.get("severity", "unknown"),
        pkg,
        vul.get("vulnerable_version_range", "?"),
        patched,
    ]))
')"

    while IFS=$'\t' read -r ghsa sev pkg range patched; do
        [ -n "$ghsa" ] || continue
        local rank
        rank="$(severity_rank "$sev")"
        if [ "$rank" -lt "$THRESHOLD_RANK" ]; then
            echo "  · ${sev} ${pkg} ${ghsa} — below threshold, not blocking"
            continue
        fi
        local ack_until
        if ack_until="$(acknowledged_until "$ghsa")"; then
            if [ "$ack_until" \< "$TODAY" ]; then
                echo "::error::${ghsa} (${sev}, ${pkg}): acknowledgement EXPIRED on ${ack_until}"
                expired=$((expired + 1))
            else
                echo "  · ${sev} ${pkg} ${ghsa} — acknowledged until ${ack_until}"
                suppressed=$((suppressed + 1))
                continue
            fi
        fi
        echo "::error::${ghsa} ${sev} ${pkg}: vulnerable ${range}, first patched ${patched}"
        blocking=$((blocking + 1))
    done <<< "$rows"

    [ "$suppressed" -eq 0 ] || echo "note: ${suppressed} alert(s) suppressed by a live acknowledgement in ${ACK_FILE}"

    if [ "$blocking" -gt 0 ] || [ "$expired" -gt 0 ]; then
        die "${blocking} unacknowledged and ${expired} expired alert(s) at or above '${THRESHOLD}'"
    fi
    echo "✅ no blocking Dependabot alerts"
}

# ---------------------------------------------------------------------------
# Self-test — the gate proves it can fail, every run
#
# #1074: "The gate is proven by a named mutation ... A gate nobody has seen fail
# is not evidence." A one-off manual mutation decays the moment someone edits the
# script; running the control on every invocation does not. The fixture is a
# real alert this repository actually had — thrift 0.17.0, GHSA-2f9f-gq7v-9h6m —
# which `cargo deny` reported as `advisories ok`.
# ---------------------------------------------------------------------------
self_test() {
    local tmp
    tmp="$(mktemp -d)"
    # Validate before any `rm -rf`: an empty or `/` value here would be
    # catastrophic, and `mktemp` failing is exactly how it becomes empty.
    case "$tmp" in
        ""|"/"|"/*") echo "::error::mktemp -d returned '$tmp'; refusing to continue"; return 1 ;;
    esac
    trap 'rm -rf -- "$tmp"' RETURN

    cat > "$tmp/vulnerable.json" <<'JSON'
[
  {
    "number": 66,
    "state": "open",
    "security_advisory": {
      "ghsa_id": "GHSA-2f9f-gq7v-9h6m",
      "severity": "medium",
      "summary": "Apache Thrift has a Memory Allocation with Excessive Size Value Vulnerability"
    },
    "security_vulnerability": {
      "package": { "ecosystem": "rust", "name": "thrift" },
      "vulnerable_version_range": "< 0.23.0",
      "first_patched_version": { "identifier": "0.23.0" }
    }
  }
]
JSON
    echo '[]' > "$tmp/clean.json"

    echo "── self-test: a known-vulnerable fixture must be RED"
    if ACK_FILE="$tmp/none.txt" "$0" --fixture "$tmp/vulnerable.json" >/dev/null 2>&1; then
        echo "::error::the gate PASSED a lockfile with a known open medium advisory — it cannot fail, so it is not a gate"
        return 1
    fi
    echo "   ✅ RED, as required"

    echo "── self-test: a clean fixture must be GREEN"
    if ! ACK_FILE="$tmp/none.txt" "$0" --fixture "$tmp/clean.json" >/dev/null 2>&1; then
        echo "::error::the gate FAILED an empty alert list — it fails on everything, which is equally useless"
        return 1
    fi
    echo "   ✅ GREEN, as required"

    echo "── self-test: an EXPIRED acknowledgement must not suppress"
    printf 'GHSA-2f9f-gq7v-9h6m 2000-01-01 expired on purpose\n' > "$tmp/expired.txt"
    if ACK_FILE="$tmp/expired.txt" "$0" --fixture "$tmp/vulnerable.json" >/dev/null 2>&1; then
        echo "::error::an expired acknowledgement still suppressed the finding"
        return 1
    fi
    echo "   ✅ expired acknowledgement does not suppress"

    echo "── self-test: a LIVE acknowledgement suppresses"
    printf 'GHSA-2f9f-gq7v-9h6m 2999-01-01 accepted for the control\n' > "$tmp/live.txt"
    if ! ACK_FILE="$tmp/live.txt" "$0" --fixture "$tmp/vulnerable.json" >/dev/null 2>&1; then
        echo "::error::a live acknowledgement did not suppress the finding"
        return 1
    fi
    echo "   ✅ live acknowledgement suppresses"

    echo "── self-test: a missing fixture is a FAILURE, not an empty pass"
    if "$0" --fixture "$tmp/does-not-exist.json" >/dev/null 2>&1; then
        echo "::error::an unreadable source was reported as clean"
        return 1
    fi
    echo "   ✅ unmeasurable is a failure"

    echo "✅ self-test passed: the gate fails when it should and passes when it should"
}

if [ "$SELF_TEST" = "1" ]; then
    self_test
    exit $?
fi

# NOT `evaluate "$(fetch_alerts)"`. `die` inside a command substitution exits the
# SUBSHELL, not this script, so a failed fetch printed its message and then let
# an empty payload flow into `evaluate` — which failed too, but on a parse error
# whose text says nothing about the 403 that actually happened. Two errors, the
# useful one buried. Capture, check the status, then evaluate.
if ! ALERTS="$(fetch_alerts)"; then
    exit 1
fi
evaluate "$ALERTS"
