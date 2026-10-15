#!/usr/bin/env bash
# publish-from-tag.sh — publish pmat to crates.io from a detached worktree of a
# release tag (PMAT-678, #1274). Never from the working tree, never with
# `--allow-dirty`, never with a registry token from the environment.
#
# WHY: at 3.38.0 `cargo publish --dry-run --locked` refused the working tree
# because an untracked `.claude/agent-memory/` made it dirty, and the file had
# to be moved aside by hand. release.yml has named this target as the publish
# path since PMAT-678, but the target itself was never added.
#
# Usage:
#   make publish-from-tag TAG=vX.Y.Z
#   DRY_RUN=1 make publish-from-tag TAG=vX.Y.Z   # dry-run only, publish nothing
#
# Refusals, exit 2, before any cargo call:
#   * no TAG, or TAG not shaped like vX.Y.Z
#   * TAG does not exist
#   * TAG's commit is not on origin/master
#   * Cargo.toml's [package] version at TAG differs from the tag
# A worktree that is not clean exits 3.
#
# The credential is cargo's own credentials file (~/.cargo/credentials.toml):
# every cargo call runs under `env -u CARGO_REGISTRY_TOKEN`, so a stale token
# in the environment can neither publish nor 403 the release.
#
# MUTATION GUARD: the guard is NOT `--detach` (`git worktree add` at a tag
# detaches with or without the flag, so dropping it changes nothing a test can
# see). The guard is that cargo runs inside a WORKTREE OF THIS REPOSITORY, and
# that the worktree has no untracked or ignored file when cargo first sees it.
# scripts/publish-from-tag-audit.sh pins both with a shim cargo.
set -euo pipefail

die() {
  printf '%s\n' "$*" >&2
  exit 2
}

run() {
  printf '+ %s\n' "$*" >&2
  "$@"
}

TAG="${1:-}"
[[ -n "$TAG" ]] || die "usage: make publish-from-tag TAG=vX.Y.Z"
[[ "$TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "refusing: '$TAG' is not shaped like vX.Y.Z"
git rev-parse -q --verify "refs/tags/$TAG" >/dev/null || die "refusing: tag '$TAG' does not exist"

BASE="${PUBLISH_BASE:-origin/master}"
if [[ "$BASE" == origin/* ]]; then
  run git fetch -q origin "${BASE#origin/}"
fi
git merge-base --is-ancestor "$TAG" "$BASE" || die "refusing: '$TAG' is not on $BASE"

# The version at the TAG, never the working tree's: master may have moved on.
manifest="$(git show "$TAG:Cargo.toml")"
version="$(printf '%s\n' "$manifest" | sed -n '/^\[package\]/,/^\[/{s/^version[[:space:]]*=[[:space:]]*"\([^"]*\)".*/\1/p}')"
version="${version%%$'\n'*}"
[[ "$version" = "${TAG#v}" ]] || die "refusing: Cargo.toml version '$version' at $TAG does not match $TAG"

WT="$(mktemp -d "${TMPDIR:-/tmp}/pmat-publish-XXXXXX")"
cleanup() {
  git worktree remove --force "$WT" >/dev/null 2>&1 || true
  rm -rf "${WT:?}"
}
trap cleanup EXIT

run git worktree add --detach "$WT" "$TAG"

# A fresh checkout of a tag holds nothing untracked and nothing ignored; any
# entry at all is a defect in this script or in the tag.
dirty="$(git -C "$WT" status --porcelain --ignored)"
[[ -z "$dirty" ]] || { printf 'refusing: the tag worktree is not pristine:\n%s\n' "$dirty" >&2; exit 3; }

cd "$WT"
run env -u CARGO_REGISTRY_TOKEN cargo publish --dry-run --locked
if [[ "${DRY_RUN:-}" = 1 ]]; then
  printf 'dry run: pmat %s packaged clean from %s; nothing published\n' "$version" "$TAG" >&2
  exit 0
fi
# The dry-run builds target/ (gitignored); anything NOT ignored is new litter.
dirty="$(git -C "$WT" status --porcelain)"
[[ -z "$dirty" ]] || { printf 'refusing: the worktree is dirty before publishing:\n%s\n' "$dirty" >&2; exit 3; }
run env -u CARGO_REGISTRY_TOKEN cargo publish --locked
printf 'published: pmat %s from %s (%s)\n' "$version" "$TAG" "$(git rev-parse HEAD)"
