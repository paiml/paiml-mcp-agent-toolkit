#!/usr/bin/env bash
#
# dependabot-alert-gate.sh — the SECOND advisory source, alongside `cargo deny`.
#
# WHY THIS EXISTS (paiml-mcp-agent-toolkit#1074)
#
#   `cargo deny check advisories` resolves against the RustSec database. GitHub's
#   advisory database is a SUPERSET of it: RustSec, plus GHSA-native advisories,
#   plus CVEs mapped to crates that were never filed with RustSec. So there is a
#   permanent, structural class of vulnerability that cargo-deny cannot see, and
#   it reports that blind spot as `advisories ok` with exit 0 — identical output
#   to a comprehensive clean run. Absence rendered as success, in the security
#   gate.
#
#   Live example, twice on the same crate: GHSA-2f9f-gq7v-9h6m / CVE-2026-43868
#   in `thrift 0.17.0`. Dependabot: open moderate alert. `cargo deny check
#   advisories`: "advisories ok", exit 0. 1,235 advisories in the local RustSec
#   checkout, no thrift entry, and no `ignore` line in deny.toml — cargo-deny
#   cannot suppress what it has never heard of.
#
#   The first occurrence was worked around by hand and nothing was added to CI,
#   so it recurred. This script is the thing that was missing.
#
#   It does NOT replace cargo-deny. Neither source subsumes the other: RustSec
#   carries unmaintained/yanked/unsound findings that Dependabot does not model
#   at all. Run both. Block on both.
#
# THE GATE MUST NOT INHERIT THE DEFECT IT FIXES
#
#   Every path that cannot produce a trustworthy answer exits 2, not 0. Missing
#   token, 401/403/404, Dependabot disabled, unparseable response, a severity
#   string this script does not know how to rank, a paginated list that might be
#   truncated, an acknowledgement file that does not parse — all of them are
#   FAILURES. "We could not measure it" must never render as "nothing was found".
#
# EXIT CODES
#   0  measured, and nothing at or above the threshold is unacknowledged
#   1  measured, and something blocks (open finding, or an expired/stale ack)
#   2  COULD NOT MEASURE — the answer is unknown, which is not the same as clean
#
# USAGE
#   scripts/dependabot-alert-gate.sh [--repo OWNER/NAME] [--severity LEVEL]
#                                    [--ack-file PATH]
#   scripts/dependabot-alert-gate.sh --self-test
#
#   --severity   low | medium (alias: moderate) | high | critical   [default: medium]
#   --ack-file   TOML acknowledgement file    [default: .github/dependabot-acknowledgements.toml]
#
#   Self-test-only flags, each of which REQUIRES DEPENDABOT_GATE_SELFTEST=1 in the
#   environment. A single stray flag must never be able to silence the gate:
#   --alerts-file PATH   read alerts from a fixture instead of the API
#   --today YYYY-MM-DD   pin "now" so expiry tests are deterministic
#
set -Eeuo pipefail

readonly EXIT_OK=0
readonly EXIT_BLOCKED=1
readonly EXIT_UNMEASURED=2

# A cursor walk that never terminates is a hang, not a measurement. 200 pages at
# 100/page is 20,000 alerts; a repo past that needs a human, not a longer loop.
readonly MAX_PAGES=200
readonly PER_PAGE=100
# An acknowledgement that expires in 2099 is not an acknowledgement.
readonly MAX_ACK_DAYS=180

SEVERITY_THRESHOLD="medium"
ACK_FILE=".github/dependabot-acknowledgements.toml"
REPO=""
ALERTS_FILE=""
TODAY_OVERRIDE=""
SELF_TEST=0

WORKDIR=""
# NOT `[ -n "$WORKDIR" ] && rm -rf "$WORKDIR"`. Under `set -e`, an EXIT trap whose
# LAST command exits non-zero makes bash leave the trap early and report 1 — so
# with an empty WORKDIR that one-liner rewrote every `exit 2` in this script into
# `exit 1`, collapsing "could not measure" into "blocked". Non-zero either way,
# so CI would still have gone red, but the diagnosis would have been wrong every
# single time. The self-test below is what caught it. Verified:
#   bash -c 'set -e; f(){ [ -n "" ] && echo x; }; trap f EXIT; exit 2'   -> 1
#   bash -c 'set -e; f(){ if [ -n "" ]; then :; fi; }; trap f EXIT; exit 2' -> 2
#
# The `if` form matters twice over: a false condition makes `if` return 0, and a
# successful `rm` returns 0, so the trap can never end non-zero either way.
cleanup() {
    if [ -n "$WORKDIR" ] && [ "$WORKDIR" != "/" ] && [ -d "$WORKDIR" ]; then
        rm -rf "$WORKDIR"
    fi
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Reporting
# ---------------------------------------------------------------------------

log()  { printf '%s\n' "$*" >&2; }
rule() { printf '%s\n' "--------------------------------------------------------------------------" >&2; }

# The only way this script is allowed to end without a verdict.
unmeasured() {
    rule
    log "CANNOT MEASURE — FAILING."
    log ""
    log "  $*"
    log ""
    log "  This is a failure and not a pass on purpose. The whole reason this gate"
    log "  exists is that a security check which cannot see something must not"
    log "  report that as 'nothing found'. See #1074."
    rule
    exit "$EXIT_UNMEASURED"
}

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

require_selftest_env() {
    if [ "${DEPENDABOT_GATE_SELFTEST:-0}" != "1" ]; then
        log "REFUSING: $1 is a self-test flag and requires DEPENDABOT_GATE_SELFTEST=1."
        log "It exists so the gate can be proven against fixtures. It is not a way to"
        log "point a production security gate at a file of your choosing."
        exit "$EXIT_UNMEASURED"
    fi
}

parse_args() {
    while [ $# -gt 0 ]; do
        case "$1" in
            --repo)        REPO="${2:?--repo needs a value}"; shift 2 ;;
            --severity)    SEVERITY_THRESHOLD="${2:?--severity needs a value}"; shift 2 ;;
            --ack-file)    ACK_FILE="${2:?--ack-file needs a value}"; shift 2 ;;
            --alerts-file) require_selftest_env "--alerts-file"
                           ALERTS_FILE="${2:?--alerts-file needs a value}"; shift 2 ;;
            --today)       require_selftest_env "--today"
                           TODAY_OVERRIDE="${2:?--today needs a value}"; shift 2 ;;
            --self-test)   SELF_TEST=1; shift ;;
            # Stops at the first non-comment line rather than at a hardcoded
            # line number, which drifts the moment the header grows.
            -h|--help)     awk 'NR > 1 { if ($0 !~ /^#/) exit; sub(/^# ?/, ""); print }' "$0"; exit 0 ;;
            *)             log "unknown argument: $1"; exit "$EXIT_UNMEASURED" ;;
        esac
    done
}

# ---------------------------------------------------------------------------
# Severity
# ---------------------------------------------------------------------------

# GitHub's API emits low|medium|high|critical. Its web UI says "moderate" where
# the API says "medium", so both spellings are accepted on input.
severity_rank() {
    case "$1" in
        low)               echo 1 ;;
        medium|moderate)   echo 2 ;;
        high)              echo 3 ;;
        critical)          echo 4 ;;
        # NOT 0. A severity this script does not recognise must not silently sort
        # below the threshold and vanish; the caller turns this into exit 2.
        *)                 echo -1 ;;
    esac
}

# ---------------------------------------------------------------------------
# Preflight
# ---------------------------------------------------------------------------

preflight() {
    command -v jq  >/dev/null 2>&1 || unmeasured "jq is not installed; the gate cannot parse the alert list."
    command -v python3 >/dev/null 2>&1 || unmeasured "python3 is not installed; the gate cannot parse $ACK_FILE."
    python3 -c 'import tomllib' >/dev/null 2>&1 \
        || unmeasured "python3 has no tomllib (needs >= 3.11); the gate cannot parse $ACK_FILE."

    THRESHOLD_RANK="$(severity_rank "$SEVERITY_THRESHOLD")"
    [ "$THRESHOLD_RANK" -ge 0 ] \
        || unmeasured "--severity '$SEVERITY_THRESHOLD' is not one of low|medium|high|critical."

    if [ -n "$TODAY_OVERRIDE" ]; then
        TODAY="$TODAY_OVERRIDE"
    else
        TODAY="$(date -u +%F)"
    fi

    # gh and a repo are only needed for a live measurement.
    if [ -z "$ALERTS_FILE" ]; then
        command -v gh >/dev/null 2>&1 \
            || unmeasured "the GitHub CLI (gh) is not installed, so the Dependabot alert list cannot be read."
        if [ -z "$REPO" ]; then
            REPO="${GITHUB_REPOSITORY:-}"
        fi
        if [ -z "$REPO" ]; then
            REPO="$(gh repo view --json nameWithOwner --jq .nameWithOwner 2>/dev/null)" || REPO=""
        fi
        [ -n "$REPO" ] || unmeasured "could not determine which repository to measure. GITHUB_REPOSITORY is unset and \`gh repo view\` failed, which usually means gh is not authenticated (no GH_TOKEN, or a token gh rejected). Pass --repo OWNER/NAME."
    fi
}

# ---------------------------------------------------------------------------
# Fetching, with an explicit proof that the list is complete
# ---------------------------------------------------------------------------
#
# `gh api` returns 30 items per page by default and, without --paginate, returns
# ONLY the first page — with no marker in the JSON to say so. On this repository
# that is 30 of 66 alerts: a truncated list that looks exactly like a complete
# short one. Issue #1074 calls that out by name.
#
# The Dependabot alerts endpoint is CURSOR-paginated: its Link header carries
# rel="next" and never rel="last", so there is no page count to compare against
# and no total in the body. What IS authoritative is the absence of rel="next" —
# GitHub emits it if and only if more results exist. So the walk below terminates
# on that condition and nothing else, and then the result is cross-checked
# against an independent `gh api --paginate` walk. Two walks that disagree means
# one of them truncated, and that is exit 2.

# Reads `gh api -i` output and echoes the rel="next" URL, or nothing.
#
# Deliberately one awk and no grep. The obvious `grep -i '^link:' | sed ... | head -1`
# spelling exits 1 whenever there is NO Link header — which is precisely the
# last page of every walk — and under `set -o pipefail` that killed the caller
# with a bare exit 1 and no output at all. `head -1` also SIGPIPEs its upstream
# (rc 141), which pipefail equally objects to. This awk always exits 0, and the
# "no next link" answer is an empty string rather than a failure.
extract_next_link() {
    tr -d '\r' | awk '
        found != ""             { next }
        inbody == 1             { next }
        /^[[:space:]]*$/        { inbody = 1; next }
        tolower($0) ~ /^link:/ {
            n = split($0, parts, ",")
            for (i = 1; i <= n; i++) {
                if (tolower(parts[i]) ~ /rel="next"/ && match(parts[i], /<[^>]*>/)) {
                    found = substr(parts[i], RSTART + 1, RLENGTH - 2)
                }
            }
        }
        END { if (found != "") print found }'
}

fetch_alerts_live() {
    local url page raw body next rc
    local acc="$WORKDIR/acc.json"
    printf '[]\n' > "$acc"

    url="repos/$REPO/dependabot/alerts?per_page=$PER_PAGE"
    page=0
    while : ; do
        page=$((page + 1))
        if [ "$page" -gt "$MAX_PAGES" ]; then
            unmeasured "the alert list did not terminate within $MAX_PAGES pages; refusing to guess at the tail."
        fi

        # NOT in a pipeline: `gh api ... | head` reports head's exit status, and
        # a 401 read through a pipe looks exactly like a success. That mistake is
        # how a broken security check stays green.
        set +e
        raw="$(gh api -i "$url" 2>&1)"
        rc=$?
        set -e
        if [ "$rc" -ne 0 ]; then
            log "gh api failed on page $page of $REPO:"
            # Status line and the API's own message only — a full header dump
            # buries the one line a reader needs.
            printf '%s\n' "$raw" | tr -d '\r' | sed -n '/^HTTP\//p' >&2
            printf '%s\n' "$raw" | sed -n 's/.*"message":"\([^"]*\)".*/  message: \1/p' >&2
            printf '%s\n' "$raw" | sed -n '/^gh: /p' >&2
            unmeasured "the Dependabot alerts API returned an error (gh exit $rc). Common causes: no token in GH_TOKEN; a token without the 'security_events' scope / 'Dependabot alerts: read' permission; no admin access to $REPO; or Dependabot alerts disabled on the repository. Every one of those is an unknown, so every one of them fails."
        fi

        body="$(printf '%s\n' "$raw" | sed '1,/^\r\{0,1\}$/d')"
        printf '%s\n' "$body" | jq -e 'type == "array"' >/dev/null 2>&1 \
            || unmeasured "page $page of the alert list is not a JSON array; the API response was not what this gate knows how to read."

        printf '%s\n' "$body" > "$WORKDIR/page.json"
        jq -s '.[0] + .[1]' "$acc" "$WORKDIR/page.json" > "$WORKDIR/acc.next.json"
        mv "$WORKDIR/acc.next.json" "$acc"

        next="$(printf '%s\n' "$raw" | extract_next_link)"
        [ -n "$next" ] || break
        url="$next"
    done

    local walk_count paginate_out paginate_count
    walk_count="$(jq 'length' "$acc")"

    # Independent second walk. If gh's --paginate and the hand walk disagree,
    # one of them stopped early and neither answer can be trusted.
    set +e
    paginate_out="$(gh api --paginate "repos/$REPO/dependabot/alerts?per_page=$PER_PAGE" 2>&1)"
    rc=$?
    set -e
    [ "$rc" -eq 0 ] || unmeasured "the cross-check walk (gh api --paginate) failed with exit $rc: $paginate_out"
    paginate_count="$(printf '%s\n' "$paginate_out" | jq -s 'map(length) | add // 0')" \
        || unmeasured "the cross-check walk did not return parseable JSON."

    if [ "$walk_count" != "$paginate_count" ]; then
        unmeasured "the two independent pagination walks disagree ($walk_count vs $paginate_count alerts). One of them truncated; a truncated list that looks complete is exactly the failure this gate exists to prevent."
    fi

    log "Measured $REPO: $walk_count alerts over $page page(s); the last response carried no rel=\"next\", and an independent --paginate walk agreed at $paginate_count."
    cat "$acc"
}

fetch_alerts_fixture() {
    [ -f "$ALERTS_FILE" ] || unmeasured "fixture $ALERTS_FILE does not exist."
    jq -e 'type == "array"' "$ALERTS_FILE" >/dev/null 2>&1 \
        || unmeasured "fixture $ALERTS_FILE is not a JSON array of alerts."
    log "!!! FIXTURE MODE — reading $ALERTS_FILE. This is NOT a live measurement of any repository. !!!"
    cat "$ALERTS_FILE"
}

# ---------------------------------------------------------------------------
# Acknowledgements
# ---------------------------------------------------------------------------
#
# Mirrors deny.toml's `[advisories] ignore`, with the two things that file lacks:
# an expiry date and a named owner. Accepting a finding is then a recorded,
# decaying decision rather than a deleted check.

load_acknowledgements() {
    local out rc
    set +e
    out="$(python3 - "$ACK_FILE" "$TODAY" "$MAX_ACK_DAYS" <<'PY'
import datetime
import json
import re
import sys
import tomllib

path, today_s, max_days = sys.argv[1], sys.argv[2], int(sys.argv[3])

REQUIRED = ("ghsa", "package", "expires", "owner", "reason")
GHSA_RE = re.compile(r"^GHSA-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4}-[23456789cfghjmpqrvwx]{4}$")


def die(msg):
    print(msg, file=sys.stderr)
    raise SystemExit(2)


try:
    today = datetime.date.fromisoformat(today_s)
except ValueError:
    die(f"--today {today_s!r} is not a YYYY-MM-DD date")

try:
    with open(path, "rb") as fh:
        doc = tomllib.load(fh)
except FileNotFoundError:
    # No file is a legitimate state: nothing has been accepted.
    print("[]")
    raise SystemExit(0)
except tomllib.TOMLDecodeError as exc:
    die(f"{path} does not parse as TOML: {exc}")

unknown_tables = sorted(set(doc) - {"acknowledged"})
if unknown_tables:
    die(f"{path}: unknown top-level key(s) {unknown_tables}; the only one read is [[acknowledged]]. "
        "A typo here would silently be a no-op, so it is an error instead.")

entries = doc.get("acknowledged", [])
if not isinstance(entries, list):
    die(f"{path}: [[acknowledged]] must be an array of tables")

out, seen = [], set()
for i, e in enumerate(entries):
    at = f"{path} entry #{i + 1}"
    if not isinstance(e, dict):
        die(f"{at}: not a table")

    missing = [k for k in REQUIRED if k not in e]
    if missing:
        die(f"{at}: missing required key(s) {missing}. Every acknowledgement needs all of {list(REQUIRED)}.")
    unknown = sorted(set(e) - set(REQUIRED))
    if unknown:
        die(f"{at}: unknown key(s) {unknown}. Rejected rather than ignored, because a misspelled "
            "'expires' that is silently dropped is an acknowledgement that never expires.")

    for k in ("ghsa", "package", "owner", "reason"):
        if not isinstance(e[k], str) or not e[k].strip():
            die(f"{at}: '{k}' must be a non-empty string")

    if not GHSA_RE.match(e["ghsa"]):
        die(f"{at}: ghsa={e['ghsa']!r} is not a GHSA id (GHSA-xxxx-xxxx-xxxx)")
    if len(e["reason"].strip()) < 20:
        die(f"{at}: 'reason' is {len(e['reason'].strip())} chars. Write a real one — this is the "
            "record of why a known vulnerability was accepted.")

    expires_raw = e["expires"]
    # tomllib turns a bare 2026-10-24 into a datetime.date; a quoted one stays a str.
    if isinstance(expires_raw, datetime.date) and not isinstance(expires_raw, datetime.datetime):
        expires = expires_raw
    elif isinstance(expires_raw, str):
        try:
            expires = datetime.date.fromisoformat(expires_raw)
        except ValueError:
            die(f"{at}: expires={expires_raw!r} is not a YYYY-MM-DD date")
    else:
        die(f"{at}: expires must be a YYYY-MM-DD date, got {type(expires_raw).__name__}")

    horizon = (expires - today).days
    if horizon > max_days:
        die(f"{at}: expires={expires} is {horizon} days out, over the {max_days}-day maximum. "
            "An acknowledgement that outlives the release it unblocked is a deleted check with extra steps.")

    key = (e["ghsa"], e["package"])
    if key in seen:
        die(f"{at}: duplicate acknowledgement for {key[0]} / {key[1]}")
    seen.add(key)

    out.append({
        "ghsa": e["ghsa"],
        "package": e["package"],
        "expires": expires.isoformat(),
        "owner": e["owner"],
        "reason": e["reason"],
        "expired": today > expires,
        "days_left": horizon,
    })

print(json.dumps(out))
PY
)"
    rc=$?
    set -e
    if [ "$rc" -ne 0 ]; then
        log "$out"
        unmeasured "the acknowledgement file could not be read. It is not treated as 'no acknowledgements' — a config file that does not parse is an unknown, not a clean bill of health."
    fi
    printf '%s\n' "$out"
}

# ---------------------------------------------------------------------------
# Evaluation
# ---------------------------------------------------------------------------

# Rejects any alert whose severity string this script cannot rank, BEFORE the
# threshold comparison. An unknown severity that sorted below the threshold
# would be a silent pass, which is the #1074 defect in miniature.
assert_severities_known() {
    local alerts="$1" sev unknown
    unknown=""
    while IFS= read -r sev; do
        [ -n "$sev" ] || continue
        if [ "$(severity_rank "$sev")" -lt 0 ]; then
            unknown="$unknown $sev"
        fi
    done <<EOF
$(printf '%s\n' "$alerts" | jq -r '.[] | (.security_vulnerability.severity // .security_advisory.severity // "<<missing>>")' | sort -u)
EOF
    [ -z "$unknown" ] \
        || unmeasured "the API returned severity value(s) this gate cannot rank:$unknown. Refusing to assume they are below the threshold."
}

main() {
    parse_args "$@"
    if [ "$SELF_TEST" -eq 1 ]; then
        self_test
        exit $?
    fi
    preflight

    WORKDIR="$(mktemp -d)"

    local alerts acks
    if [ -n "$ALERTS_FILE" ]; then
        alerts="$(fetch_alerts_fixture)"
    else
        alerts="$(fetch_alerts_live)"
    fi

    printf '%s\n' "$alerts" | jq -e 'all(.[]; has("state") and has("security_vulnerability"))' >/dev/null 2>&1 \
        || unmeasured "at least one alert is missing 'state' or 'security_vulnerability'; the response shape is not what this gate knows how to read."

    assert_severities_known "$alerts"
    acks="$(load_acknowledgements)"

    local report
    report="$(printf '%s\n' "$alerts" | jq \
        --argjson acks "$acks" \
        --argjson threshold "$THRESHOLD_RANK" \
        --arg today "$TODAY" '
        def rank: {"low":1,"medium":2,"moderate":2,"high":3,"critical":4}[.];
        def norm:
          { number, state,
            ghsa:      .security_advisory.ghsa_id,
            cve:       (.security_advisory.cve_id // "-"),
            package:   .dependency.package.name,
            manifest:  (.dependency.manifest_path // "-"),
            summary:   (.security_advisory.summary // "-"),
            severity:  (.security_vulnerability.severity // .security_advisory.severity),
            range:     (.security_vulnerability.vulnerable_version_range // "-"),
            patched:   (.security_vulnerability.first_patched_version.identifier // "none available"),
            url:       (.html_url // "-") };
        def ack_for($a): ($acks | map(select(.ghsa == $a.ghsa and .package == $a.package)) | first);

        map(norm) as $all
        | ($all | map(select(.state == "open" and (.severity | rank) >= $threshold))) as $blocking_pool
        | ($all | map(select(.state == "dismissed" and (.severity | rank) >= $threshold))) as $dismissed
        | ($blocking_pool | map(. + {ack: ack_for(.)})) as $judged
        | ($judged | map(select(.ack == null or .ack.expired))) as $blocked
        | ($judged | map(select(.ack != null and (.ack.expired | not)))) as $accepted
        | ($acks | map(. as $k | select([$blocking_pool[] | select(.ghsa == $k.ghsa and .package == $k.package)] | length == 0))) as $stale
        | { total: ($all | length),
            open: ($all | map(select(.state == "open")) | length),
            at_or_above: ($blocking_pool | length),
            blocked: $blocked, accepted: $accepted, stale: $stale, dismissed: $dismissed }
    ')" || unmeasured "the alert list could not be evaluated by jq."

    print_report "$report"
}

print_report() {
    local report="$1" n_blocked n_stale n_dismissed
    n_blocked="$(printf '%s\n' "$report" | jq '.blocked | length')"
    n_stale="$(printf '%s\n' "$report" | jq '.stale | length')"
    n_dismissed="$(printf '%s\n' "$report" | jq '.dismissed | length')"

    rule
    log "Dependabot advisory gate — threshold: $SEVERITY_THRESHOLD and above (as of $TODAY)"
    log "$(printf '%s\n' "$report" | jq -r '"  \(.total) alert(s) known to GitHub; \(.open) open; \(.at_or_above) open at or above threshold."')"
    rule

    printf '%s\n' "$report" | jq -r '.accepted[] |
        "ACCEPTED  #\(.number)  \(.severity)  \(.package)  \(.ghsa)\n" +
        "          vulnerable: \(.range)   first patched: \(.patched)\n" +
        "          acknowledged by \(.ack.owner), expires \(.ack.expires) (\(.ack.days_left) days left)\n" +
        "          \(.ack.reason)"' >&2

    printf '%s\n' "$report" | jq -r '.dismissed[] |
        "DISMISSED #\(.number)  \(.severity)  \(.package)  \(.ghsa) — dismissed in the GitHub UI, which has no\n" +
        "          expiry. Printed here so it is visible to this gate'"'"'s readers rather than silent."' >&2

    printf '%s\n' "$report" | jq -r '.blocked[] |
        "BLOCKING  #\(.number)  \(.severity)  \(.package)  \(.ghsa)  \(.cve)\n" +
        "          \(.summary)\n" +
        "          manifest: \(.manifest)\n" +
        "          vulnerable: \(.range)   first patched: \(.patched)\n" +
        "          \(.url)" +
        (if .ack != null then "\n          NOTE: its acknowledgement EXPIRED on \(.ack.expires) (owner: \(.ack.owner)). Re-decide it; do not just push the date out." else "" end)' >&2

    printf '%s\n' "$report" | jq -r '.stale[] |
        "STALE ACK \(.ghsa) / \(.package) in '"$ACK_FILE"' matches no open alert at or above the\n" +
        "          threshold. Delete the entry. An allowlist that outlives its finding is how a\n" +
        "          gate quietly becomes broader than anyone intended (#1074, closing note)."' >&2

    if [ "$n_dismissed" -gt 0 ]; then
        rule
        log "$n_dismissed alert(s) at or above the threshold are dismissed in the GitHub UI."
        log "That path has no expiry and no owner recorded here. Prefer an entry in $ACK_FILE."
    fi

    rule
    if [ "$n_blocked" -gt 0 ] || [ "$n_stale" -gt 0 ]; then
        log "FAIL: $n_blocked unacknowledged/expired finding(s) at or above '$SEVERITY_THRESHOLD', $n_stale stale acknowledgement(s)."
        log ""
        log "To accept a finding, add to $ACK_FILE:"
        log ""
        log "  [[acknowledged]]"
        log "  ghsa    = \"GHSA-xxxx-xxxx-xxxx\""
        log "  package = \"crate-name\""
        log "  expires = 2026-01-01   # at most $MAX_ACK_DAYS days out"
        log "  owner   = \"github-handle\""
        log "  reason  = \"why this is accepted, and what closes it\""
        rule
        exit "$EXIT_BLOCKED"
    fi
    log "PASS: no unacknowledged Dependabot alert at or above '$SEVERITY_THRESHOLD'."
    log "This says nothing about RustSec-only findings — that is \`cargo deny check\`'s half."
    rule
    exit "$EXIT_OK"
}

# ---------------------------------------------------------------------------
# Self-test — the named mutation, runnable by anyone, in CI or locally
# ---------------------------------------------------------------------------
#
# A gate nobody has watched fail is not evidence. Every case below asserts an
# exact exit code, and the suite includes counter-tests: a low-severity alert and
# an empty list must PASS, so "flag everything" cannot be mistaken for working.

st_pass=0
st_fail=0

# expect <want-exit> <name> -- <args...>
expect() {
    local want="$1" name="$2" got
    shift 3  # want, name, and the literal --
    set +e
    DEPENDABOT_GATE_SELFTEST=1 "$0" "$@" >/dev/null 2>&1
    got=$?
    set -e
    if [ "$got" = "$want" ]; then
        printf '  ok    %-46s exit=%s\n' "$name" "$got"
        st_pass=$((st_pass + 1))
    else
        printf '  FAIL  %-46s exit=%s (wanted %s)\n' "$name" "$got" "$want"
        st_fail=$((st_fail + 1))
    fi
}

write_alert_fixture() {
    local dest="$1" severity="$2" state="$3" ghsa="$4" pkg="$5"
    cat > "$dest" <<JSON
[ { "number": 66,
    "state": "$state",
    "html_url": "https://example.invalid/alerts/66",
    "dependency": { "package": { "ecosystem": "rust", "name": "$pkg" }, "manifest_path": "Cargo.lock" },
    "security_advisory": { "ghsa_id": "$ghsa", "cve_id": "CVE-2026-43868",
                           "summary": "Memory Allocation with Excessive Size Value" },
    "security_vulnerability": { "package": { "ecosystem": "rust", "name": "$pkg" },
                                "severity": "$severity",
                                "vulnerable_version_range": "< 0.23.0",
                                "first_patched_version": { "identifier": "0.23.0" } } } ]
JSON
}

write_ack_fixture() {
    local dest="$1" ghsa="$2" pkg="$3" expires="$4"
    cat > "$dest" <<TOML
[[acknowledged]]
ghsa    = "$ghsa"
package = "$pkg"
expires = $expires
owner   = "selftest"
reason  = "fixture entry used only by the gate's own self-test suite"
TOML
}

self_test() {
    local d
    d="$(mktemp -d)"
    # shellcheck disable=SC2064  # $d must expand now, not at trap time
    trap "rm -rf '$d'" RETURN

    local GHSA="GHSA-2f9f-gq7v-9h6m"
    local NONE="$d/none.toml"           # deliberately never created

    write_alert_fixture "$d/vuln.json"     medium   open      "$GHSA" thrift
    write_alert_fixture "$d/low.json"      low      open      "$GHSA" thrift
    write_alert_fixture "$d/fixed.json"    critical fixed     "$GHSA" thrift
    write_alert_fixture "$d/weird.json"    spicy    open      "$GHSA" thrift
    printf '[]\n' > "$d/empty.json"

    # Inside the 180-day horizon relative to the pinned --today 2026-08-25.
    write_ack_fixture "$d/ack-live.toml"    "$GHSA" thrift 2026-10-24
    write_ack_fixture "$d/ack-expired.toml" "$GHSA" thrift 2020-01-01
    write_ack_fixture "$d/ack-other.toml"   "GHSA-cccc-cccc-cccc" someothercrate 2026-10-24
    printf 'this is not toml [[[\n' > "$d/ack-broken.toml"

    # An alert whose severity cannot be read at all. `.security_vulnerability`
    # is present but null, so `.severity` yields null and the rank lookup would
    # otherwise return nothing — the exact shape that must not sort below the
    # threshold and vanish.
    cat > "$d/nullsev.json" <<'JSON'
[ { "number": 66, "state": "open",
    "dependency": { "package": { "name": "thrift" } },
    "security_advisory": { "ghsa_id": "GHSA-2f9f-gq7v-9h6m" },
    "security_vulnerability": null } ]
JSON

    # Two entries for the same GHSA+crate: one of them is unreachable, and which
    # one wins would decide the expiry date. Rejected rather than resolved.
    cat "$d/ack-live.toml" "$d/ack-expired.toml" > "$d/ack-dup.toml"
    cat > "$d/ack-typo.toml" <<'TOML'
[[acknowledged]]
ghsa    = "GHSA-2f9f-gq7v-9h6m"
package = "thrift"
expries = 2026-10-24
owner   = "selftest"
reason  = "a misspelled expires key must be rejected, not silently ignored"
TOML

    echo
    echo "=== dependabot-alert-gate self-test ==============================="
    echo "--- the mutation: a known-vulnerable finding must go RED"
    expect 1 "open medium finding, no ack"            -- --alerts-file "$d/vuln.json"  --ack-file "$NONE" --today 2026-08-25
    expect 1 "open medium finding, EXPIRED ack"       -- --alerts-file "$d/vuln.json"  --ack-file "$d/ack-expired.toml" --today 2026-08-25
    expect 1 "stale ack matching no open alert"       -- --alerts-file "$d/empty.json" --ack-file "$d/ack-other.toml" --today 2026-08-25

    echo "--- restore: the same finding, consciously accepted, must go GREEN"
    expect 0 "open medium finding, live ack"          -- --alerts-file "$d/vuln.json"  --ack-file "$d/ack-live.toml" --today 2026-08-25

    echo "--- counter-tests: over-correction must NOT pass as working"
    expect 0 "empty alert list"                       -- --alerts-file "$d/empty.json" --ack-file "$NONE" --today 2026-08-25
    expect 0 "low finding under a medium threshold"   -- --alerts-file "$d/low.json"   --ack-file "$NONE" --today 2026-08-25
    expect 0 "already-fixed critical alert"           -- --alerts-file "$d/fixed.json" --ack-file "$NONE" --today 2026-08-25
    expect 1 "low finding WITH a low threshold"       -- --alerts-file "$d/low.json"   --ack-file "$NONE" --today 2026-08-25 --severity low

    echo "--- cannot-measure must FAIL, never pass"
    expect 2 "unrankable severity string"             -- --alerts-file "$d/weird.json"    --ack-file "$NONE" --today 2026-08-25
    expect 2 "alert with no readable severity at all" -- --alerts-file "$d/nullsev.json"  --ack-file "$NONE" --today 2026-08-25
    expect 2 "two acks for the same GHSA + crate"     -- --alerts-file "$d/vuln.json"     --ack-file "$d/ack-dup.toml" --today 2026-08-25
    expect 2 "ack file that does not parse"           -- --alerts-file "$d/vuln.json"     --ack-file "$d/ack-broken.toml" --today 2026-08-25
    expect 2 "ack with a misspelled 'expires' key"    -- --alerts-file "$d/vuln.json"     --ack-file "$d/ack-typo.toml" --today 2026-08-25
    expect 2 "ack expiring beyond the max horizon"    -- --alerts-file "$d/vuln.json"     --ack-file "$d/ack-live.toml" --today 2026-01-01
    expect 2 "fixture file that does not exist"       -- --alerts-file "$d/nope.json"     --ack-file "$NONE" --today 2026-08-25
    expect 2 "unknown --severity value"               -- --alerts-file "$d/vuln.json"     --ack-file "$NONE" --today 2026-08-25 --severity urgent
    expect 2 "repository that does not exist (live)"  -- --repo "paiml/dependabot-gate-selftest-no-such-repo" --ack-file "$NONE"

    echo "--- the fixture escape hatch must not be usable without the env var"
    set +e
    DEPENDABOT_GATE_SELFTEST=0 "$0" --alerts-file "$d/empty.json" >/dev/null 2>&1
    local got=$?
    set -e
    if [ "$got" = "2" ]; then
        printf '  ok    %-46s exit=%s\n' "--alerts-file refused without env" "$got"
        st_pass=$((st_pass + 1))
    else
        printf '  FAIL  %-46s exit=%s (wanted 2)\n' "--alerts-file refused without env" "$got"
        st_fail=$((st_fail + 1))
    fi

    echo "=================================================================="
    echo "  $st_pass passed, $st_fail failed"
    [ "$st_fail" -eq 0 ] || return 1
    return 0
}

main "$@"
