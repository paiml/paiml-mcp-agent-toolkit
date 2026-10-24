#!/usr/bin/env bash
# Self-test for emr1-verify.sh: four fixtures plus a mutation proof that the
# suite goes RED when the key comparison is removed. Never touches real ledger.
set -u
HERE="$(cd "$(dirname "$0")" && pwd)"
V="$HERE/emr1-verify.sh"
T="$(mktemp -d)"
[ -n "$T" ] && [ -d "$T" ] || exit 2
trap '[ -n "$T" ] && rm -rf "${T:?}"' EXIT
export GIT_AUTHOR_NAME=t GIT_AUTHOR_EMAIL=t@t GIT_COMMITTER_NAME=t GIT_COMMITTER_EMAIL=t@t

# No --stable fallback: git >= 2.39 is required ($EMR_GIT or PATH); older => exit 3.
GIT="${EMR_GIT:-git}"; export EMR_GIT="$GIT"
if ! "$GIT" patch-id --verbatim </dev/null >/dev/null 2>&1; then
  echo "selftest: $GIT lacks patch-id --verbatim (need >= 2.39): not measured"
  "$V" -r . -l /dev/null >/dev/null 2>&1; [ $? -eq 3 ] || { echo "FAIL old git must exit 3"; exit 1; }
  exit 3
fi
git() { "$GIT" "$@"; }

R="$T/repo"
git init -q "$R" && cd "$R" || exit 2
git config core.hooksPath /dev/null
seq 1 40 >f.txt && git add f.txt && git commit -q -m base
git checkout -q -b src
sed -i '20s/.*/twenty-changed/' f.txt && git commit -qam change
SRC=$(git rev-parse HEAD)
# main drifts next to the changed line (context change), then the change lands on it
git checkout -q master 2>/dev/null || git checkout -q main
sed -i '22s/.*/drift-neighbour/' f.txt && git commit -qam drift
git cherry-pick -x "$SRC" >/dev/null 2>&1 || { echo "selftest: cherry-pick failed"; exit 2; }
LANDED=$(git rev-parse HEAD)

KEY=$(git diff -U0 --no-color --no-ext-diff --no-renames --diff-algorithm=myers "$SRC^" "$SRC" | git patch-id --verbatim | cut -d' ' -f1)
BAD=0000000000000000000000000000000000000000
mkdir -p handoff/receipts/9 handoff/receipts/7 emr
printf '{"key":"%s","head_sha":"%s","verdict":"PASS"}\n' "$KEY" "$SRC" >"handoff/receipts/9/$KEY.json"
printf '{"key":"%s","head_sha":"%s","verdict":"PASS"}\n' "$BAD" "$SRC" >"handoff/receipts/9/$BAD.json"
H=$'pr\tticket\tclass\tkey\treview\tprescreen\tint_sha\tcheckpoint\tdisposition\treason\tts'
row() { printf '%s\tPMAT-1\tc\t%s\tr\tp\t%s\tck\tlanded\t-\t2026-10-24T00:00:00Z\n' "${2:-9}" "$1" "$LANDED"; }
mk() { { echo "$H"; cat; } >"emr/$1.tsv"; }
row "$KEY" | mk match            # also the drift fixture
row "$KEY" 8 | mk missing        # no receipt under handoff/receipts/8
row "$BAD" | mk mismatch         # ledger+receipt agree, but not the commit
printf '{"key":"%s","head_sha":"%s","verdict":"FAIL"}\n' "$KEY" "$SRC" >handoff/receipts/7/$KEY.json
row "$KEY" 7 | mk fail            # right key, verdict FAIL
echo "$H" | mk empty

fail=0
expect() { # name ledger want_rc script
  "$4" -r "$R" -l "emr/$2.tsv" >"$T/out" 2>&1; rc=$?
  if [ "$rc" -eq "$3" ]; then echo "PASS $1 (rc=$rc)"; else echo "FAIL $1: rc=$rc want $3"; sed 's/^/  | /' "$T/out"; fail=1; fi
}
# the landed commit's context differs from the source PR's (main drifted) yet the key survives:
[ "$(git diff -U3 "$SRC^" "$SRC" | tail -n +5)" != "$(git diff -U3 "$LANDED^" "$LANDED" | tail -n +5)" ] || { echo "FAIL drift fixture has no context change"; fail=1; }
expect "match + main drift (key survives context change)" match 0 "$V"
expect "missing receipt" missing 1 "$V"
expect "mismatched key" mismatch 1 "$V"
expect "receipt verdict FAIL" fail 1 "$V"
expect "zero rows refuses" empty 2 "$V"

# Mutation proof: remove the key comparison; the mismatch fixture must now pass (suite RED).
sed 's/^key_ok() .*# KEYCMP$/key_ok() { true; } # mutated/' "$V" >"$T/mut.sh" && chmod +x "$T/mut.sh"
grep -q 'mutated' "$T/mut.sh" || { echo "FAIL mutation did not apply"; fail=1; }
"$T/mut.sh" -r "$R" -l emr/mismatch.tsv >/dev/null 2>&1; mrc=$?
if [ "$mrc" -eq 0 ]; then echo "PROOF mutant (key comparison removed) passes the mismatch fixture: unmutated suite would go RED"
else echo "FAIL mutant still rejects mismatch (rc=$mrc): the key test does not discriminate"; fail=1; fi
[ "$fail" -eq 0 ] && echo "emr1-verify selftest: OK" || echo "emr1-verify selftest: FAILED"
exit "$fail"
