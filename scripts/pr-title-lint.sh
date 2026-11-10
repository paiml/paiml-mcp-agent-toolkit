#!/usr/bin/env bash
# pr-title-lint.sh — falsifier for the "PR Title Check" job in .github/workflows/pr-checks.yml (GH-1685).
# It reads the allowed types and subjectPattern FROM that workflow (one source) and judges titles
# the way amannn/action-semantic-pull-request does: type(scope)!: subject.
# Usage: scripts/pr-title-lint.sh --self-test | --title "type(scope): subject"
# Exit: 0 accepted / self-test passed · 1 rejected / a fixture misjudged · 2 usage or workflow unreadable
set -uo pipefail
wf="$(cd "$(dirname "$0")" && pwd)/../.github/workflows/pr-checks.yml"
types="$(awk '/types: \|/{f=1;next} f&&/requireScope/{exit} f{gsub(/[ \t]/,"");if($0!="")print}' "$wf")"
pat="$(sed -n 's/^ *subjectPattern: //p' "$wf" | head -1)"
[ -n "$types" ] && [ -n "$pat" ] || { echo "pr-title-lint: cannot read types/subjectPattern from $wf" >&2; exit 2; }
judge() {
  local t="$1" re='^([A-Za-z]+)(\([^)]+\))?(!)?: (.+)$'
  [[ "$t" =~ $re ]] || return 1
  grep -qxF "${BASH_REMATCH[1]}" <<<"$types" || return 1
  [[ "${BASH_REMATCH[4]}" =~ $pat ]]
}
case "${1:-}" in
  --title) [ -n "${2:-}" ] || exit 2; judge "$2" && { echo "ok: $2"; exit 0; } || { echo "rejected: $2" >&2; exit 1; } ;;
  --self-test)
    bad=0
    expect() { judge "$2"; local rc=$?; if { [ "$1" = pass ] && [ $rc -ne 0 ]; } || { [ "$1" = fail ] && [ $rc -eq 0 ]; }; then echo "MISJUDGED ($1): $2" >&2; bad=1; fi; }
    expect pass "train(paiml-mcp-agent-toolkit): #1685 accept the train type"
    expect pass "train: #12 subject starting with a ticket number"
    expect pass "fix(ci): a plain conventional title"
    expect fail "bogus(ci): an unknown type"
    expect fail "train(x): ends with a period."
    expect fail "train(x): #"
    expect fail "no type at all"
    [ $bad -eq 0 ] && echo "pr-title-lint self-test: ok" ;;
  *) echo "usage: $0 --self-test | --title TITLE" >&2; exit 2 ;;
esac
exit $bad
