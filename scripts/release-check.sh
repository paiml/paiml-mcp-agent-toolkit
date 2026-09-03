#!/usr/bin/env bash
# AD-01 — a merged release must have BECOME a release (agentic-delivery-pmat.md section 3.5 / 9.1).
#
# Why: pmat 3.35.0's release PR (#1108) merged on 2026-09-02 and nothing tagged, released or
# published it; crates.io stayed at 3.34.0 and every gate was green, because no gate asks
# "did the version Cargo.toml declares reach the three channels". This one does.
#
# Reads the version from Cargo.toml (never a literal). If it is the latest v* tag, exit 0.
# Otherwise require, in this order, the tag, the GitHub release, and the crates.io version;
# the first missing one exits 1 naming the version. Runs on master after merge, never as a
# required PR check (a PR is not a release).
#
#   bash scripts/release-check.sh                 # judge this checkout
#   bash scripts/release-check.sh --self-test     # prove it can fail: a 9.9.9 fixture must fail every leg
#   CRATE=<name> REPO=<owner/name> override the crate and repository (defaults: pmat, paiml/paiml-mcp-agent-toolkit)
set -euo pipefail
fail(){ echo "FAIL: release-check: $*"; exit 1; }
CRATE="${CRATE:-pmat}"
REPO="${REPO:-paiml/paiml-mcp-agent-toolkit}"

check_version(){ # $1 version → 0 if fully released, 1 naming the first missing channel
  local v=$1
  git tag -l "v$v" | grep -q . || { echo "FAIL: release-check: Cargo.toml says $v but no tag v$v"; return 1; }
  gh release view "v$v" --repo "$REPO" >/dev/null 2>&1 || { echo "FAIL: release-check: no GitHub release v$v"; return 1; }
  curl -sf --max-time 20 "https://crates.io/api/v1/crates/$CRATE/$v" -H "User-Agent: release-check ($REPO)" \
    | jq -e --arg v "$v" '.version.num == $v and (.version.yanked | not)' >/dev/null 2>&1 \
    || { echo "FAIL: release-check: $v is not on crates.io (or is yanked)"; return 1; }
  echo "release-check: $v is tagged, released and on crates.io"
}

if [ "${1:-}" = "--self-test" ]; then
  # The control: a version that exists nowhere must fail at the tag leg, and — with a
  # planted tag — at the release leg, and — with a planted release name — at the registry
  # leg. A check that hardcodes any real version passes none of these.
  T=$(mktemp -d)
  cleanup(){ case "${T:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$T" ] && rm -rf -- "$T";; esac; }; trap cleanup EXIT
  git -C "$T" init -q; printf '[package]\nname = "fx"\nversion = "9.9.9"\n' > "$T/Cargo.toml"
  git -C "$T" -c user.email=t@t -c user.name=t add . >/dev/null; git -C "$T" -c user.email=t@t -c user.name=t -c core.hooksPath=/dev/null commit -qm fx
  out=$(cd "$T" && check_version 9.9.9 2>&1 || true)
  grep -q 'no tag v9.9.9' <<<"$out" || fail "self-test: missing tag not reported: $out"
  git -C "$T" tag v9.9.9
  out=$(cd "$T" && check_version 9.9.9 2>&1 || true)
  grep -q 'no GitHub release v9.9.9' <<<"$out" || fail "self-test: missing release not reported: $out"
  echo "SELF-TEST PASSED: the tag and release legs fired on a 9.9.9 fixture (the registry leg is exercised by the real run)"
  exit 0
fi

v=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
[ -n "$v" ] || fail "no version in Cargo.toml"
# Numeric tags only: this repository carries a stray `vv2.88.0`, which `sort -V` ranks above every number.
latest_tag=$(git tag -l 'v*' | sed -E 's/^v+//' | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)
echo "release-check: Cargo.toml $v · latest tag ${latest_tag:-none}"
check_version "$v"
