#!/usr/bin/env bash
# board-check.sh — the board is cleared: every open PR/issue/ticket has an
# enacted disposition. READ-ONLY: never mutates GitHub or the roadmap.
# Prints one `GAP <kind> <id> <reason>` line per gap found, then a summary
# line `board-check: <n> gap(s)`. Exits 1 iff n>0, 0 otherwise. (PMAT-693)
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: board-check.sh [--ledger PATH] [--repo OWNER/NAME] [--roadmap PATH] [--offline]

  --ledger PATH      use this dispositions ledger instead of the lexically
                      newest ledger under docs/audits
  --repo OWNER/NAME  GitHub repo to query (default: taken from `gh repo view`)
  --roadmap PATH     roadmap.yaml to read tickets from (default:
                      docs/roadmaps/roadmap.yaml)
  --offline          skip every `gh` call; only the ticket leg runs
USAGE
}

ledger_path=""
repo=""
roadmap_path="docs/roadmaps/roadmap.yaml"
offline=0

while [ $# -gt 0 ]; do
  case "$1" in
    --ledger)
      ledger_path="$2"
      shift 2
      ;;
    --repo)
      repo="$2"
      shift 2
      ;;
    --roadmap)
      roadmap_path="$2"
      shift 2
      ;;
    --offline)
      offline=1
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "board-check: unrecognized argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

# ── Find the newest ledger (lexically-last dispositions ledger under
# docs/audits), unless one was given explicitly. No `ls` parsing: a glob
# array, sorted.
find_newest_ledger() {
  local -a candidates=()
  shopt -s nullglob
  candidates=(docs/audits/dispositions-*.json)
  shopt -u nullglob
  if [ "${#candidates[@]}" -eq 0 ]; then
    return 1
  fi
  printf '%s\n' "${candidates[@]}" | sort | tail -n 1
}

if [ -z "${ledger_path}" ]; then
  if ! ledger_path="$(find_newest_ledger)"; then
    echo "board-check: no ledger found under docs/audits and none given via --ledger" >&2
    exit 2
  fi
fi

if [ ! -f "${ledger_path}" ]; then
  echo "board-check: ledger file does not exist: ${ledger_path}" >&2
  exit 2
fi

if [ ! -f "${roadmap_path}" ]; then
  echo "board-check: roadmap file does not exist: ${roadmap_path}" >&2
  exit 2
fi

if [[ "${offline}" -eq 0 && -z "${repo}" ]]; then
  repo="$(gh repo view --json nameWithOwner --jq '.nameWithOwner')"
fi

gap_count=0

# report_gap: print one GAP line and bump the running total.
# $1=kind $2=id $3=reason
report_gap() {
  echo "GAP $1 $2 $3"
  ((gap_count++)) || true
}

# report_gap_lines: print every non-empty line of $1 (one GAP line per line)
# and bump the running total by however many lines were printed.
report_gap_lines() {
  local text="$1"
  [ -n "${text}" ] || return 0
  printf '%s\n' "${text}"
  local n
  n="$(printf '%s\n' "${text}" | grep -c '^GAP ')"
  ((gap_count += n)) || true
}

# ── Ticket leg: raw-text parse of roadmap.yaml. Every top-level `- id: PMAT-N`
# row, paired with the first `status:` line that follows it (before the next
# top-level `- id:` row). Never shells out to `pmat`.
ticket_table="$(awk '
  /^- id: PMAT-[0-9]+/ {
    current_id = $3
    capturing = 1
    next
  }
  /^- id:/ {
    capturing = 0
    next
  }
  capturing && /^  status:/ {
    print current_id "\t" $2
    capturing = 0
  }
' "${roadmap_path}")"

# One ledger read for every ticket row, keyed by ticket id, built once so the
# per-ticket loop below never forks a subprocess.
declare -A ledger_ticket_disposition=()
while IFS=$'\t' read -r row_id row_disposition; do
  [ -n "${row_id}" ] || continue
  ledger_ticket_disposition["${row_id}"]="${row_disposition}"
done < <(jq -r '.[] | select(.kind == "ticket") | "\(.id)\t\(.disposition)"' "${ledger_path}")

is_completed_status() {
  case "$1" in
    completed|cancelled|rejected) return 0 ;;
    *) return 1 ;;
  esac
}

is_enactable_disposition() {
  case "$1" in
    complete|reject) return 0 ;;
    defer\(*\)) return 0 ;;
    *) return 1 ;;
  esac
}

if [ -n "${ticket_table}" ]; then
  while IFS=$'\t' read -r ticket_id ticket_status; do
    [ -n "${ticket_id}" ] || continue
    if is_completed_status "${ticket_status}"; then
      continue
    fi
    disposition="${ledger_ticket_disposition[${ticket_id}]:-}"
    if [ -z "${disposition}" ]; then
      report_gap ticket "${ticket_id}" "not in ledger"
      continue
    fi
    if ! is_enactable_disposition "${disposition}"; then
      report_gap ticket "${ticket_id}" "disposition ${disposition} is not enactable"
      continue
    fi
    if [ "${disposition}" = "complete" ]; then
      report_gap ticket "${ticket_id}" "marked complete in the ledger but roadmap status is still ${ticket_status}"
    fi
  done <<TICKETS
${ticket_table}
TICKETS
fi

if [ "${offline}" -eq 1 ]; then
  echo "board-check: --offline set — PR and issue legs skipped, only the ticket leg ran"
  echo "board-check: ${gap_count} gap(s)"
  if [ "${gap_count}" -gt 0 ]; then
    exit 1
  fi
  exit 0
fi

# ── PR leg: one jq pass over open PRs against the ledger's "pr" rows.
open_pr_table="$(gh pr list --repo "${repo}" --state open --json number,title,headRefName)"
pr_gap_text="$(jq -r \
  --argjson prs "${open_pr_table}" \
  --slurpfile ledger_rows "${ledger_path}" \
  -n '
    (($ledger_rows | .[0]) // []) as $rows
    | ($rows | map(select(.kind == "pr")) | map(.id)) as $covered
    | ($prs | .[])
    | ("#" + (.number | tostring)) as $pr_id
    | select(($covered | index($pr_id)) | not)
    | "GAP pr " + $pr_id + " no disposition row"
  ')"
report_gap_lines "${pr_gap_text}"

# ── Issue leg: one jq pass over open issues against the ledger's "issue" rows.
# A newer ledger row may carry an `evidence` or `evidenced` key; a non-null
# value there is treated as an evidence-closure that changes the missing-
# milestone wording (see PMAT-693 brief).
open_issue_table="$(gh issue list --repo "${repo}" --state open --limit 200 --json number,title,milestone,labels,state)"
issue_gap_text="$(jq -r \
  --argjson issues "${open_issue_table}" \
  --slurpfile ledger_rows "${ledger_path}" \
  -n '
    (($ledger_rows | .[0]) // []) as $rows
    | ($issues | .[])
    | . as $issue
    | ("#" + (.number | tostring)) as $issue_id
    | ($rows | map(select(.kind == "issue" and .id == $issue_id)) | first) as $row
    | (($row.evidence // $row.evidenced // null) != null) as $has_evidence
    | (
        if $issue.milestone == null then
          if $has_evidence then
            "GAP issue " + $issue_id + " milestone missing and no evidence-closure"
          else
            "GAP issue " + $issue_id + " no milestone"
          end
        else
          empty
        end
      ),
      (
        if ($row.disposition // "") == "complete" then
          "GAP issue " + $issue_id + " ledger says complete but the issue is still open"
        else
          empty
        end
      )
  ')"
report_gap_lines "${issue_gap_text}"

echo "board-check: ${gap_count} gap(s)"
if [ "${gap_count}" -gt 0 ]; then
  exit 1
fi
exit 0
