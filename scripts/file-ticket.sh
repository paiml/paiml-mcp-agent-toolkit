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
body_file=""
labels="triaged"

die() { echo "file-ticket: $1" >&2; exit "${2:-2}"; }

gh_api() { "${gh_bin}" "$@"; }
parent_issue() { gh_api api "repos/$1/issues/$2"; }
create_issue() { gh_api api -X POST "repos/$1/issues" --input -; }
sub_issues() { gh_api api --paginate "repos/$1/issues/$2/sub_issues" --jq ".[].number"; }

while [ $# -gt 0 ]; do
  case "$1" in
    --parent|--title|--body|--body-file|--label|--repo) [ $# -ge 2 ] || die "refusing: $1 needs a value" ;;
  esac
  case "$1" in
    --parent) parent="${2:-}"; shift 2 ;;
    --title) title="${2:-}"; shift 2 ;;
    --body) body="${2:-}"; shift 2 ;;
    --body-file) body_file="${2:-}"; shift 2 ;;
    --label) labels="${labels},${2:-}"; shift 2 ;;
    --repo) repo="${2:-}"; shift 2 ;;
    *) die "unknown argument: $1" ;;
  esac
done

if [ -n "${body_file}" ]; then
  case "${body_file}" in
    /* | *..*) die "refusing: --body-file must be a relative path with no dot-dot segment" ;;
  esac
  [ -r "${body_file}" ] || die "cannot open --body-file ${body_file}"
  IFS= read -r -d '' body < "${body_file}" || :
fi

case "${parent}" in
  "" ) die "refusing: a ticket needs a parent (--parent N, a master or epic)" ;;
  *[!0-9]*) die "refusing: --parent must be an issue number, got ${parent}" ;;
esac
[ -n "${title}" ] || die "refusing: --title is required"

parent_json="$(parent_issue "${repo}" "${parent}")" || die "could not fetch parent ${parent}" 1
milestone="$(printf '%s' "${parent_json}" | jq -r ".milestone.number // empty")"
[ -n "${milestone}" ] || die "refusing: parent ${parent} has no milestone; set it first"

payload="$(jq -cn --arg t "${title}" --arg b "${body}" --argjson m "${milestone}" --arg l "${labels}" \
  '{title: $t, body: $b, milestone: $m, labels: ($l | split(",") | map(select(length > 0)))}')" \
  || die "could not build the issue payload" 1
created="$(printf '%s' "${payload}" | create_issue "${repo}")" || die "create failed" 1
number="$(printf '%s' "${created}" | jq -r ".number // empty")"
id="$(printf '%s' "${created}" | jq -r ".id // empty")"
if [ -z "${number}" ] || [ -z "${id}" ]; then
  die "create returned no issue object: ${created}" 1
fi

gh_api api -X POST "repos/${repo}/issues/${parent}/sub_issues" -F "sub_issue_id=${id}" >/dev/null \
  || die "issue ${number} created but NOT linked to ${parent}; link it by hand" 1
subs="$(sub_issues "${repo}" "${parent}")" \
  || die "issue ${number} created but sub-issue list fetch for ${parent} failed" 1
printf '%s\n' "${subs}" | grep -qx "${number}" || die "issue ${number} link to ${parent} is missing from the sub-issue list" 1

echo "filed issue ${number} under ${parent} (milestone ${milestone}, labels: ${labels})"
