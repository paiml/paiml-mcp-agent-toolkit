#!/usr/bin/env bash
# emr1-verify.sh - EMR-001 contract EMR-1, read-only.
# Every ledger row with disposition=landed must have: int_sha whose -U0 patch-id
# equals key, and a receipt at handoff/receipts/<pr>/<key>.json whose key field
# equals key and whose verdict is PASS (PASS|FAIL). Never writes the ledger or receipts.
# Usage: emr1-verify.sh [-r REPO] [-l LEDGER]
# Exit: 0 all rows ok | 1 violations | 2 zero landed rows / bad input | 3 not measured
set -u

# --- receipt field names (one block, change here) ---
RCPT_KEY_FIELD='key'
RCPT_VERDICT_FIELD='verdict'
RCPT_VERDICT_PASS='PASS'
# --- ledger columns (1-based): pr ticket class key review prescreen int_sha checkpoint disposition reason ts ---
COL_PR=1 COL_KEY=4 COL_INTSHA=7 COL_DISP=9
DISP_LANDED='landed'
RCPT_DIR='handoff/receipts'

REPO="."
LEDGER=""
while getopts "r:l:" opt; do
  case "$opt" in
    r) REPO="$OPTARG" ;;
    l) LEDGER="$OPTARG" ;;
    *) echo "usage: emr1-verify.sh [-r REPO] [-l LEDGER]" >&2; exit 2 ;;
  esac
done
[ -n "$LEDGER" ] || LEDGER="$REPO/emr/ledger.tsv"
[ -r "$LEDGER" ] || { echo "emr1-verify: ledger unreadable: $LEDGER" >&2; exit 2; }
command -v jq >/dev/null 2>&1 || { echo "emr1-verify: jq missing (not measured)" >&2; exit 3; }

# git patch-id --verbatim needs git >= 2.39; there is NO --stable fallback: a different
# key is not a measurement. The git binary is $EMR_GIT (default: git on PATH).
GIT="${EMR_GIT:-git}"
if ! "$GIT" patch-id --verbatim </dev/null >/dev/null 2>&1; then
  echo "emr1-verify: $GIT lacks 'patch-id --verbatim' (need >= 2.39): not measured" >&2
  exit 3
fi

key_of() {
  "$GIT" -C "$REPO" diff -U0 --no-color --no-ext-diff --no-renames --diff-algorithm=myers "$1^" "$1" |
    "$GIT" patch-id --verbatim | cut -d' ' -f1
}

key_ok() { [ "$1" = "$2" ]; } # KEYCMP

checked=0
viol=0
report() { viol=$((viol + 1)); printf 'VIOLATION row %s (%s): %s\n' "$1" "$2" "$3"; }

lineno=0
while IFS=$'\t' read -r -a f || [ -n "${f[*]:-}" ]; do
  lineno=$((lineno + 1))
  [ "$lineno" -eq 1 ] && continue # header
  [ "${f[COL_DISP - 1]:-}" = "$DISP_LANDED" ] || continue
  checked=$((checked + 1))
  key="${f[COL_KEY - 1]:-}"
  sha="${f[COL_INTSHA - 1]:-}"
  pr="${f[COL_PR - 1]:-}"
  id="line$lineno"
  if [ -z "$key" ] || [ -z "$sha" ] || [ -z "$pr" ]; then
    report "$id" "${sha:-?}" "empty key, int_sha or pr"
    continue
  fi
  got=$(key_of "$sha" 2>/dev/null) || got=""
  if [ -z "$got" ]; then
    report "$id" "$sha" "cannot compute patch-id (sha missing or root commit)"
  elif ! key_ok "$got" "$key"; then
    report "$id" "$sha" "int_sha patch-id $got != key $key"
  fi
  rcpt="$RCPT_DIR/$pr/$key.json"
  rpath="$REPO/$rcpt"
  if [ ! -f "$rpath" ]; then
    report "$id" "$sha" "receipt missing: $rcpt"
    continue
  fi
  rkey=$(jq -r --arg f "$RCPT_KEY_FIELD" '.[$f] // empty' "$rpath" 2>/dev/null) || rkey=""
  rver=$(jq -r --arg f "$RCPT_VERDICT_FIELD" '.[$f] // empty' "$rpath" 2>/dev/null) || rver=""
  key_ok "$rkey" "$key" || report "$id" "$sha" "receipt key '$rkey' != key $key"
  [ "$rver" = "$RCPT_VERDICT_PASS" ] || report "$id" "$sha" "receipt verdict '$rver' != $RCPT_VERDICT_PASS"
done <"$LEDGER"

echo "emr1-verify: rows checked=$checked violations=$viol ledger=$LEDGER"
[ "$checked" -gt 0 ] || { echo "emr1-verify: zero landed rows: refusing to pass" >&2; exit 2; }
[ "$viol" -eq 0 ]
