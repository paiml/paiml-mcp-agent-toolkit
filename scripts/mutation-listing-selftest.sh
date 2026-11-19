#!/usr/bin/env bash
# Offline self-test for scripts/mutation-listing.sh (GH-1846): empty stdout -> `[]`,
# non-empty stdout kept verbatim, a failing listing writes nothing, the diff copy is byte-equal.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
work="$(mktemp -d)"
trap 'rm -rf "${work:?}"' EXIT
fail=0
check() { if [ "$2" = "$3" ]; then echo "ok   $1"; else echo "FAIL $1: got '$2' want '$3'"; fail=1; fi; }

printf 'diff --git a/x b/x\n+changed' >"$work/in.diff" # no trailing newline: byte-equality matters

printf '#!/bin/sh\nexit 0\n' >"$work/empty"
printf '#!/bin/sh\necho "[{\\"name\\":\\"m\\"}]"\n' >"$work/some"
printf '#!/bin/sh\nexit 3\n' >"$work/broken"
chmod +x "$work/empty" "$work/some" "$work/broken"

CARGO_MUTANTS_BIN="$work/empty" "$here/mutation-listing.sh" "$work/in.diff" "$work/a"
check "empty stdout maps to []" "$(cat "$work/a/mutants-listing.json")" "[]"
check "diff copy is byte-equal" "$(cmp "$work/in.diff" "$work/a/mutants-listing.diff" && echo same)" "same"

CARGO_MUTANTS_BIN="$work/some" "$here/mutation-listing.sh" "$work/in.diff" "$work/b"
check "non-empty listing kept" "$(cat "$work/b/mutants-listing.json")" '[{"name":"m"}]'

mkdir "$work/c"
echo stale >"$work/c/mutants-listing.json"
rc=0
CARGO_MUTANTS_BIN="$work/broken" "$here/mutation-listing.sh" "$work/in.diff" "$work/c" 2>/dev/null || rc=$?
check "failing listing exits non-zero" "$rc" "1"
check "failing listing leaves stale file untouched, never []" "$(cat "$work/c/mutants-listing.json")" "stale"

exit "$fail"
