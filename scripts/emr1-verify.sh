#!/bin/sh
# emr1-verify.sh - EMR-1 read-only verifier (EMR-001 rev1 section 7, issue #1528).
#
# For every ledger row whose disposition is "landed":
#   - the commit named by int_sha must have a -U0 patch-id (parent -> commit,
#     diff-algorithm myers) equal to the row's key column;
#   - the row's review column must name a receipt file that carries
#     "EMR-Key: <that patch-id>" and "Verdict: PASS" on its own lines.
#
# Usage: emr1-verify.sh <ledger.tsv> [repo] [receipts_dir]
#   repo          git work tree (default: .)
#   receipts_dir  directory that a relative review path is resolved against (default: .)
#
# Output: one "PASS <ticket>" or "FAIL <ticket> <reason>" line per landed row,
#         then a summary line.
# Exit:   0 every landed row passes (or there are none), 1 any row fails,
#         2 usage error or unreadable ledger.
#
# Read-only: it runs only git rev-parse, git diff and git patch-id. It writes no
# files, changes no git state and makes no GitHub calls.
set -eu

ledger=${1:-}
repo=${2:-.}
receipts=${3:-.}

if [ -z "$ledger" ] || [ ! -r "$ledger" ]; then
  echo "emr1-verify: usage: emr1-verify.sh <ledger.tsv> [repo] [receipts_dir]" >&2
  exit 2
fi

# The contract's key is `git patch-id --verbatim`. Older git (before 2.41) refuses that
# flag; fail loudly here rather than report every row as a key mismatch.
if ! git patch-id --verbatim </dev/null >/dev/null 2>&1; then
  echo "emr1-verify: git patch-id --verbatim unsupported by this git; EMR-1 cannot compute keys" >&2
  exit 2
fi

tab=$(printf '\t')

# Landed rows as: int_sha, key, review, ticket. Columns are found by header name.
rows=$(awk -F "$tab" '
  NR == 1 {
    for (i = 1; i <= NF; i++) col[$i] = i
    split("ticket key review int_sha disposition", need, " ")
    for (k in need) if (!(need[k] in col)) { print "missing column " need[k] > "/dev/stderr"; exit 2 }
    next
  }
  $(col["disposition"]) == "landed" {
    print $(col["int_sha"]) "\t" $(col["key"]) "\t" $(col["review"]) "\t" $(col["ticket"])
  }
' "$ledger") || { echo "emr1-verify: ledger has no usable header: $ledger" >&2; exit 2; }

verdicts=$(printf '%s\n' "$rows" | while IFS="$tab" read -r sha key review ticket; do
  [ -n "$sha" ] || continue

  if ! git -C "$repo" rev-parse --verify --quiet "$sha^{commit}" >/dev/null 2>&1; then
    echo "FAIL $ticket commit-missing"
    continue
  fi
  if ! parent=$(git -C "$repo" rev-parse --verify --quiet "$sha^" 2>/dev/null); then
    echo "FAIL $ticket no-parent"
    continue
  fi

  computed=$(git -C "$repo" diff -U0 --no-color --no-ext-diff --no-renames \
    --diff-algorithm=myers "$parent" "$sha" | git patch-id --verbatim | awk '{print $1}')
  if [ -z "$computed" ] || [ -z "$key" ] || [ "$computed" != "$key" ]; then
    echo "FAIL $ticket key-mismatch"
    continue
  fi

  case $review in
    /*) path=$review ;;
    *) path=$receipts/$review ;;
  esac
  if [ ! -f "$path" ]; then
    echo "FAIL $ticket receipt-missing"
  elif ! grep -qxF "EMR-Key: $computed" "$path"; then
    echo "FAIL $ticket receipt-key-mismatch"
  elif ! grep -qxF "Verdict: PASS" "$path"; then
    echo "FAIL $ticket receipt-not-passing"
  else
    echo "PASS $ticket"
  fi
done) || true

if [ -n "$verdicts" ]; then
  printf '%s\n' "$verdicts"
fi

total=$(printf '%s\n' "$verdicts" | grep -c -E '^(PASS|FAIL) ' || true)
fails=$(printf '%s\n' "$verdicts" | grep -c -E '^FAIL ' || true)
echo "emr1-verify: $total landed, $((total - fails)) pass, $fails fail"

[ "$fails" -eq 0 ] || exit 1
exit 0
