#!/usr/bin/env bash
# AD-03 — commit enforcement (docs/specifications/agentic-delivery-pmat.md section 4.7 / 9.3, #1126).
# The hooks pmat installs must be able to REFUSE a commit, and the record of the link
# between a commit and its work is the `Pmat-Ticket:` trailer git itself can read.
# Legs, each driven through `git commit` (the only oracle that tells a warning from a refusal):
#   1 strict install; a commit with no trailer and no ticket reference is refused, naming the trailer
#   2 the same change with `Pmat-Ticket: PMAT-NNN` is accepted, and git reads the trailer back
#   3 `#NNN` in the body satisfies the rule (repositories without pmat work)
#   4 control: a non-strict install warns and accepts — an upgrade cannot lock a repository out
# PMAT=<binary> overrides the pmat used.
set -euo pipefail
fail(){ echo "FAIL: $*"; exit 1; }
PMAT=${PMAT:-$(command -v pmat)}
T=$(mktemp -d)
cleanup(){ case "${T:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$T" ] && rm -rf -- "$T";; esac; }; trap cleanup EXIT
new_repo(){ # $1 dir
  mkdir -p "$1"; git -C "$1" init -q; git -C "$1" config user.email t@t; git -C "$1" config user.name t
  printf 'one\n' > "$1/README.md"; git -C "$1" add .; git -C "$1" commit -q -m init
}
# ---- 1: strict refuses
R="$T/strict"; new_repo "$R"
( cd "$R" && "$PMAT" hooks install --strict --force >/dev/null 2>&1 ) || fail "1: hooks install --strict failed"
[ -x "$R/.git/hooks/commit-msg" ] || fail "1: no commit-msg hook was installed"
printf 'one\ntwo\n' > "$R/README.md"
if git -C "$R" commit -qam 'no trailer' 2> "$T/err1"; then fail "1: a commit without Pmat-Ticket was ACCEPTED under strict"; fi
grep -q 'Pmat-Ticket' "$T/err1" || fail "1: the refusal does not name the trailer: $(cat "$T/err1")"
[ "$(git -C "$R" rev-list --count HEAD)" = 1 ] || fail "1: the refused commit exists"
# ---- 2: the trailer is accepted and git-readable
git -C "$R" commit -qam 'with trailer' -m 'Pmat-Ticket: PMAT-655' 2> "$T/err2" || fail "2: a commit with the trailer was refused: $(cat "$T/err2")"
[ "$(git -C "$R" log -1 --format='%(trailers:key=Pmat-Ticket,valueonly)' | tr -d '[:space:]')" = "PMAT-655" ] || fail "2: git cannot read the trailer back"
# ---- 3: an issue reference satisfies the rule
printf 'one\ntwo\nthree\n' > "$R/README.md"
git -C "$R" commit -qam 'fix the thing (#1126)' 2> "$T/err3" || fail "3: '#1126' did not satisfy the rule: $(cat "$T/err3")"
# ---- 4: control — non-strict warns and accepts
N="$T/lenient"; new_repo "$N"
( cd "$N" && "$PMAT" hooks install --force >/dev/null 2>&1 ) || fail "4: hooks install failed"
printf 'one\ntwo\n' > "$N/README.md"
git -C "$N" commit -qam 'no trailer' 2> "$T/err4" || fail "4: non-strict REFUSED a commit (an upgrade must not lock a repository out)"
grep -q 'Pmat-Ticket' "$T/err4" || fail "4: non-strict did not warn about the missing trailer"
echo PASS
