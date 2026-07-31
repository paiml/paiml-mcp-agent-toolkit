#!/usr/bin/env bash
#
# Build the artifact users actually get, prove it came from this tree, then hand
# it to a caller for measurement.
#
# WHY THIS EXISTS
#
# v3.28.2 shipped a headline fix that did not work for anyone who installed it.
# The fix was real, but every pre-release measurement was taken against a binary
# that differed from the published artifact:
#
#   * a binary two hours stale, at a path `cargo metadata` reported but the build
#     did not write to      -> produced a false "improved to 8/12"
#   * a workspace build     -> 30/30, where users got 11/30
#   * a lockfile build      -> pmcp 2.11, where `cargo install` resolves 2.17
#
# The last one was the killer: `cargo install` ignores Cargo.lock, so users build
# against whatever the version requirements admit, which is not what CI tests.
#
# This script removes the possibility of measuring the wrong binary:
#
#   1. builds with `cargo install --path .` and deliberately WITHOUT `--locked`,
#      so dependency resolution matches a real user install;
#   2. refuses to continue unless the binary's embedded commit SHA equals HEAD
#      and its embedded worktree state matches the working tree (see
#      `emit_build_provenance` in build.rs);
#   3. prints the absolute path of the verified binary on stdout, so callers
#      cannot accidentally point at some other build.
#
# USAGE
#   scripts/verify-artifact.sh                 # build + verify, print path
#   scripts/verify-artifact.sh --allow-dirty   # permit a dirty worktree
#   BIN=$(scripts/verify-artifact.sh) && "$BIN" --version
#
set -euo pipefail

ALLOW_DIRTY=0
for arg in "$@"; do
  case "$arg" in
    --allow-dirty) ALLOW_DIRTY=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

REPO_ROOT="$(git rev-parse --show-toplevel)"
cd "$REPO_ROOT" || exit 1

HEAD_SHA="$(git rev-parse HEAD)"
if [ -z "$(git status --porcelain --untracked-files=no)" ]; then
  TREE_STATE="clean"
else
  TREE_STATE="dirty"
fi

if [ "$TREE_STATE" = "dirty" ] && [ "$ALLOW_DIRTY" -eq 0 ]; then
  cat >&2 <<EOF
error: working tree is dirty.

A measurement taken from a dirty tree cannot be reproduced from any commit, so
it is not evidence about a release. Commit first, or pass --allow-dirty if you
are deliberately measuring work in progress.
EOF
  exit 1
fi

ROOT="${VERIFY_ARTIFACT_ROOT:-$(mktemp -d -t pmat-artifact-XXXXXX)}"

# SEC010: only ever build into an absolute path with no traversal component, so a
# hostile VERIFY_ARTIFACT_ROOT cannot write outside where it claims to.
case "$ROOT" in
  /*) : ;;
  *) echo "error: VERIFY_ARTIFACT_ROOT must be an absolute path, got: $ROOT" >&2; exit 2 ;;
esac
case "$ROOT" in
  *..*) echo "error: VERIFY_ARTIFACT_ROOT must not contain '..', got: $ROOT" >&2; exit 2 ;;
esac

# REL002: clean up the temp root on failure only. Deliberately NOT an
# unconditional EXIT trap: on success the caller runs the binary we just
# verified, so removing it here would defeat the whole point of the script.
CLEANUP_ROOT="$ROOT"
VERIFIED=0
cleanup_on_failure() {
  [ "$VERIFIED" -eq 0 ] || return 0
  # Only remove a root this script created via mktemp.
  [ -z "${VERIFY_ARTIFACT_ROOT:-}" ] || return 0
  # SEC011: never let an empty, root, or non-absolute value reach `rm -rf`.
  case "${CLEANUP_ROOT:-}" in
    ""|"/"|"/home"|"/tmp") return 0 ;;
    /*/*) : ;;
    *) return 0 ;;
  esac
  case "$CLEANUP_ROOT" in
    *pmat-artifact-*) : ;;
    *) return 0 ;;
  esac
  if [ -n "$CLEANUP_ROOT" ] && [ "$CLEANUP_ROOT" != "/" ] && [ -d "$CLEANUP_ROOT" ]; then
    rm -rf -- "$CLEANUP_ROOT"
  fi
}
trap cleanup_on_failure EXIT INT TERM

mkdir -p "$ROOT"

{
  echo "building the install artifact (fresh dependency resolution, NOT --locked)"
  echo "  tree:  $HEAD_SHA ($TREE_STATE)"
  echo "  root:  $ROOT"
} >&2

# `env -u CARGO_REGISTRY_TOKEN`: a stale token in the environment overrides
# credentials.toml and breaks cargo's registry access.
env -u CARGO_REGISTRY_TOKEN cargo install --path . --root "$ROOT" --force >&2

BIN="$ROOT/bin/pmat"
[ -x "$BIN" ] || { echo "error: no binary produced at $BIN" >&2; exit 1; }

# --- provenance gate -------------------------------------------------------
LONG_VERSION="$("$BIN" --version 2>&1 || true)"
BIN_SHA="$(printf '%s\n' "$LONG_VERSION" | sed -n 's/^commit: //p' | head -1)"
BIN_TREE="$(printf '%s\n' "$LONG_VERSION" | sed -n 's/^worktree: //p' | head -1)"

if [ -z "$BIN_SHA" ]; then
  cat >&2 <<EOF
error: the binary reports no build provenance.

Expected '--version' to include a 'commit:' line, emitted by
emit_build_provenance() in build.rs. Without it there is no way to tell this
binary apart from a stale one, which is the failure this script exists to
prevent.

got:
$LONG_VERSION
EOF
  exit 1
fi

if [ "$BIN_SHA" = "unknown" ]; then
  echo "error: binary was built without git metadata; cannot confirm it matches this tree." >&2
  exit 1
fi

if [ "$BIN_SHA" != "$HEAD_SHA" ]; then
  cat >&2 <<EOF
error: STALE OR MISMATCHED BINARY -- refusing to measure it.

  binary was built from: $BIN_SHA
  this tree is at:       $HEAD_SHA

This is exactly the mistake that let v3.28.2 ship a fix that did not work.
EOF
  exit 1
fi

if [ "$BIN_TREE" != "$TREE_STATE" ]; then
  echo "error: binary reports worktree '$BIN_TREE' but the tree is '$TREE_STATE'." >&2
  exit 1
fi

VERIFIED=1
echo "provenance OK: $BIN_SHA ($BIN_TREE)" >&2
echo "$BIN"
