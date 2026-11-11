#!/usr/bin/env bash
# OUT-324 (1): a push that changes Cargo.lock or deny.toml must pass
# `cargo deny check advisories`. Reads git's pre-push stdin
# (<lref> <lsha> <rref> <rsha>). Exit 0 ok / nothing to check, 1 advisory
# failure, 2 not measured (cargo-deny missing): not measured blocks the push.
# PMAT_DENY_CMD overrides the command (used by the self-test).
set -u
zero=0000000000000000000000000000000000000000
cmd="${PMAT_DENY_CMD:-cargo deny check advisories}"
touched=0
while read -r _lref lsha _rref rsha; do
  [ -n "${lsha:-}" ] || continue
  [ "$lsha" = "$zero" ] && continue   # branch delete
  if [ "$rsha" = "$zero" ]; then
    base=$(git merge-base "$lsha" origin/master 2>/dev/null || true)
    [ -n "$base" ] || { echo "pre-push-advisory-check: no merge-base for new branch (not measured)"; exit 2; }
  else
    base="$rsha"
  fi
  if git diff --name-only "$base" "$lsha" 2>/dev/null | grep -qE '^(Cargo\.lock|deny\.toml)$'; then
    touched=1
  fi
done
[ "$touched" = 1 ] || { echo "pre-push-advisory-check: Cargo.lock/deny.toml untouched, skip"; exit 0; }
# shellcheck disable=SC2086  # intentional word splitting: PMAT_DENY_CMD is a command line
set -- $cmd
command -v "$1" >/dev/null 2>&1 || { echo "pre-push-advisory-check: '$1' not installed (not measured, push blocked)"; exit 2; }
echo "pre-push-advisory-check: Cargo.lock/deny.toml changed -> $cmd"
$cmd || { echo "pre-push-advisory-check: ADVISORY FAILURE, push refused"; exit 1; }
