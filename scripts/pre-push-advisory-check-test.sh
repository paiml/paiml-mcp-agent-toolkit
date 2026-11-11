#!/usr/bin/env bash
# Falsifier for OUT-324 (1): planted advisory failure refuses; untouched skips; missing tool = 2.
set -u
chk="$(cd "$(dirname "$0")" && pwd)/pre-push-advisory-check.sh"
T=$(mktemp -d); trap 'rm -rf -- "${T:?}"' EXIT
cd "$T" && git init -q . && git config user.email t@t && git config user.name t
echo a > a; echo l1 > Cargo.lock; git add -A; git commit -qm base; base=$(git rev-parse HEAD)
echo b > a; git commit -qam noop; noop=$(git rev-parse HEAD)
echo l2 > Cargo.lock; git commit -qam lock; lock=$(git rev-parse HEAD)
run() { printf 'r %s r %s\n' "$2" "$1" | PMAT_DENY_CMD="$3" "$chk" >/dev/null; echo $?; }
[ "$(run "$base" "$noop" false)" = 0 ] || { echo "FAIL untouched must skip"; exit 1; }
[ "$(run "$base" "$lock" false)" = 1 ] || { echo "FAIL planted advisory failure must refuse"; exit 1; }
[ "$(run "$base" "$lock" true)" = 0 ] || { echo "FAIL clean deny must pass"; exit 1; }
[ "$(run "$base" "$lock" nosuch-tool-xyz)" = 2 ] || { echo "FAIL missing tool must be 2"; exit 1; }
echo "PASS pre-push-advisory-check (4 cases; planted failure refused)"
