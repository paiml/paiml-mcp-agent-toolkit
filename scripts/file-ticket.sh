#!/usr/bin/env bash
# file-ticket.sh — file a ticket complete, in one step (GH-1808).
#
# Creates the issue with the parent's milestone and the `triaged` label,
# attaches it to the parent as a sub-issue, then reads the link back.
# Refuses (exit 2, nothing written) when no --parent is given or the parent has
# no milestone. Exit 1 when a write or the read-back failed after creation.
#
# Rule 1 first: if an open group/master/train ticket covers the work, add a
# checklist line there instead of running this.
#
# Usage: file-ticket.sh --parent N --title T (--body B | --body-file F)
#                       [--label L]... [--repo OWNER/NAME]
# Seam: FILE_TICKET_GH names the gh binary (the self-test points it at a stub).
set -euo pipefail

gh_bin="${FILE_TICKET_GH:-gh}"
repo="paiml/paiml-mcp-agent-toolkit"
parent=""
title=""
body=""
labels=(triaged)

die() { echo "file-ticket: $1" >&2; exit "${2:-2}"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --parent|--title|--body|--body-file|--label|--repo) [ $# -ge 2 ] || die "refusing: $1 needs a value" ;;
  esac
  case "$1" in
    --parent) parent="${2:-}"; shift 2 ;;
    --title) title="${2:-}"; shift 2 ;;
    --body) body="${2:-}"; shift 2 ;;
    --body-file)
      bf="${2:-}"
      case "${bf}" in
        /* | *..*) die "refusing: --body-file must be a relative path without '..'" ;;
      esac
      body="$(cat "${bf}")" || die "cannot read --body-file ${bf}"; shift 2 ;;
    --label) labels+=("${2:-}"); shift 2 ;;
    --repo) repo="${2:-}"; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done

case "${parent}" in
  "" ) die "refusing: a ticket needs a parent (--parent N, a master or epic)" ;;
  *[!0-9]*) die "refusing: --parent must be an issue number, got '${parent}'" ;;
esac
[ -n "${title}" ] || die "refusing: --title is required"

milestone="$("${gh_bin}" api "repos/${repo}/issues/${parent}" | jq -r '.milestone.number // empty')" \
  || die "could not read parent #${parent}" 1
[ -n "${milestone}" ] || die "refusing: parent #${parent} has no milestone; set it first"

args=(api -X POST "repos/${repo}/issues" -f "title=${title}" -f "body=${body}" -F "milestone=${milestone}")
for l in "${labels[@]}"; do args+=(-f "labels[]=${l}"); done
created="$("${gh_bin}" "${args[@]}")" || die "create failed" 1
number="$(printf '%s' "${created}" | jq -r '.number // empty')"
id="$(printf '%s' "${created}" | jq -r '.id // empty')"
if [ -z "${number}" ] || [ -z "${id}" ]; then
  die "create returned no issue object: ${created}" 1
fi

"${gh_bin}" api -X POST "repos/${repo}/issues/${parent}/sub_issues" -F "sub_issue_id=${id}" >/dev/null \
  || die "#${number} created but NOT linked to #${parent}; link it by hand" 1
subs="$("${gh_bin}" api --paginate "repos/${repo}/issues/${parent}/sub_issues" --jq '.[].number')" \
  || die "#${number} created but sub-issue read-back of #${parent} failed" 1
printf '%s\n' "${subs}" | grep -qx "${number}" || die "#${number} link to #${parent} did not read back" 1

echo "filed #${number} under #${parent} (milestone ${milestone}, labels: ${labels[*]})"
