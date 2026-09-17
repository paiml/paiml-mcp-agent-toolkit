#!/usr/bin/env bash
# pr-body-closing-lint.sh — a PR's title and body close only what a line starting `Closes #N` names
# (PMAT-900001, contracts/pmat-issue-closure-v1.yaml).
#
# GitHub closes an issue when a merged PR body puts close/fix/resolve (any tense, optional colon)
# directly before #N or owner/repo#N — `no-close: #3091` closed aprender#3091, and a PR body closed
# #1339 here (1cdffdcca). The title is judged too: a merge commit carries it. The predicate is the
# ONE snippet every commit-msg hook pmat writes sources: src/services/closing_keywords_lint.sh.
#
# Usage: scripts/pr-body-closing-lint.sh --pr N [--repo OWNER/REPO]   the LIVE title and body (gh)
#        scripts/pr-body-closing-lint.sh --file FILE                    a title+body already on disk
#        scripts/pr-body-closing-lint.sh --self-test                    the RED/GREEN fixtures
# Exit:  0 nothing closes unintentionally · 1 a line would close an issue, or (self-test) a fixture
#        was misjudged · 2 usage, or the PR could not be read (never a silent pass)
set -uo pipefail

here="$(cd "$(dirname "$0")" && pwd)"
# shellcheck source=../src/services/closing_keywords_lint.sh
source "$here/../src/services/closing_keywords_lint.sh" || { echo "pr-body-closing-lint: cannot source the lint snippet" >&2; exit 2; }

lint_file() {
  local hits
  hits=$(pmat_closing_keyword_hits < "$1") || { echo "pr-body-closing-lint: PASS — no line closes an issue"; return 0; }
  echo "pr-body-closing-lint: FAIL — merging this PR would close an issue its author did not name on a Closes line:" >&2
  printf '%s\n' "$hits" | sed 's/^/  line /' >&2
  echo "  to close on purpose, start its own line with:  Closes #N" >&2
  echo "  to mention it, break the adjacency:  fixes issue #N  |  keeps-open #N  |  Refs #N" >&2
  return 1
}

self_test() {
  local tmp bad=0 text want got
  tmp=$(mktemp) || return 2
  # want: 1 = must be refused, 0 = must pass. The brief's fixtures (PMAT-900001), verbatim.
  while IFS='|' read -r want text; do
    printf 'docs: a title\n\n%s\n' "$text" > "$tmp"
    lint_file "$tmp" >/dev/null 2>&1; got=$?
    if [ "$got" = "$want" ]; then echo "  ok    exit $got  $text"; else echo "  WRONG exit $got (want $want)  $text"; bad=1; fi
  done <<'FIXTURES'
1|no-close: #3091
1|No-Close #12
1|this fixes #5
1|re-fixes: #7
0|Closes #1
0|keeps-open #3091
0|prefix #12
0|fixture #3
FIXTURES
  rm -f "$tmp"
  [ "$bad" -eq 0 ] && echo "pr-body-closing-lint --self-test: every fixture as expected"
  return "$bad"
}

case "${1:-}" in
  --self-test) self_test; exit $? ;;
  --file) [ -f "${2:-}" ] || { echo "pr-body-closing-lint: --file needs a readable file" >&2; exit 2; }
          lint_file "$2"; exit $? ;;
  --pr)
    pr="${2:-}"; repo=()
    [ -n "$pr" ] || { echo "pr-body-closing-lint: --pr needs a number" >&2; exit 2; }
    if [ "${3:-}" = "--repo" ]; then repo=(--repo "${4:-}"); fi
    tmp=$(mktemp) || exit 2
    # The LIVE title and body, read now — not the event payload, which is stale after an edit.
    if ! gh pr view "$pr" "${repo[@]}" --json title,body --jq '.title + "\n\n" + .body' > "$tmp"; then
      echo "pr-body-closing-lint: could not read PR $pr — refusing to report a pass" >&2; rm -f "$tmp"; exit 2
    fi
    lint_file "$tmp"; rc=$?; rm -f "$tmp"; exit "$rc" ;;
  -h|--help) sed -n '2,/^set -uo pipefail$/p' "$0" | sed '$d'; exit 0 ;;
  *) sed -n '2,/^set -uo pipefail$/p' "$0" | sed '$d' >&2; exit 2 ;;
esac
