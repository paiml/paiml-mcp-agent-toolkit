#!/usr/bin/env bash
# publish-from-tag-audit — PMAT-678 (#1274) falsifier for scripts/publish-from-tag.sh,
# run against a throwaway repository with a shim `cargo` first on PATH. The shim never
# reaches crates.io; it logs its argv, its cwd's git common dir, the worktree's status
# and whether CARGO_REGISTRY_TOKEN survived into its environment.
#   bash scripts/publish-from-tag-audit.sh               # every arm
#   bash scripts/publish-from-tag-audit.sh --self-test   # must be RED against a no-op script
# Exit 0 = every arm green; 1 = an arm red; 2 = harness failure.
set -uo pipefail
HERE=$(cd "$(dirname "$0")" && pwd)
if [ "${1:-}" = "--self-test" ]; then
  S=$(mktemp -d); printf '#!/bin/sh\nexit 0\n' > "$S/noop.sh"
  if PUBLISH_SCRIPT="$S/noop.sh" bash "$0" >/dev/null 2>&1; then echo "self-test FAILED: the audit passed against a no-op publish script"; rm -rf "${S:?}"; exit 1; fi
  rm -rf "${S:?}"; echo "self-test OK: the audit is RED against a no-op publish script"; exit 0
fi
SCRIPT=${PUBLISH_SCRIPT:-$HERE/publish-from-tag.sh}
[ -f "$SCRIPT" ] || { echo "no script at $SCRIPT" >&2; exit 2; }
red=0; arm(){ if [ "$2" = 0 ]; then echo "  ✓ $1"; else echo "  ✗ $1"; red=1; fi; }
T=$(mktemp -d); [[ "$T" == /* && "$T" != "/" ]] || exit 2
trap 'rm -rf "${T:?}"' EXIT
unset GIT_DIR GIT_WORK_TREE

# ── fixture: a repo at version 1.2.3, tagged, pushed to a bare origin ─────────
R="$T/repo"; mkdir -p "$R/src" "$T/bin"
g() { git -C "$R" -c user.email=a@b -c user.name=a -c core.hooksPath=/dev/null "$@" >/dev/null 2>&1; }
printf '[package]\nname = "fixture"\nversion = "1.2.3"\nedition = "2021"\n\n[dependencies]\nx = { version = "9.9.9" }\n' > "$R/Cargo.toml"
printf 'target/\n' > "$R/.gitignore"; printf 'fn main() {}\n' > "$R/src/main.rs"
git init -q --bare -b master "$T/origin.git" >/dev/null 2>&1
{ g init -q -b master && g add . && g commit -qm v1.2.3 && g tag v1.2.3 \
  && g remote add origin "$T/origin.git" && g push -q origin master \
  && g checkout -qb side && g commit -q --allow-empty -m off-master && g tag v9.9.9 \
  && g checkout -q master && g tag v1.2.4 v1.2.3; } || { echo "fixture failed" >&2; exit 2; }
# v9.9.9 is off master; v1.2.4 is on master but Cargo.toml there says 1.2.3.
# The untracked file that stopped 3.38.0's dry run, in the main tree:
mkdir -p "$R/.claude/agent-memory"; echo x > "$R/.claude/agent-memory/notes.md"

LOG="$T/cargo.log"
cat > "$T/bin/cargo" <<EOF
#!/bin/sh
{ printf 'argv=%s\n' "\$*"
  printf 'common=%s\n' "\$(cd "\$(git rev-parse --git-common-dir)" && pwd)"
  printf 'top=%s\n' "\$(git rev-parse --show-toplevel)"
  printf 'status=%s\n' "\$(git status --porcelain --ignored | tr '\n' ';')"
  printf 'token=%s\n' "\${CARGO_REGISTRY_TOKEN-UNSET}"; } >> "$LOG"
mkdir -p target
EOF
chmod +x "$T/bin/cargo"
pub() { # <tag> [VAR=val…] → rc in $rc, stderr in $T/err
  : > "$LOG"; local tag=$1; shift
  (cd "$R" && env PATH="$T/bin:$PATH" TMPDIR="$T" CARGO_REGISTRY_TOKEN=stale "$@" bash "$SCRIPT" "$tag") >/dev/null 2>"$T/err"; rc=$?
}
no_cargo() { [ ! -s "$LOG" ]; }

pub ""
[ "$rc" = 2 ] && no_cargo; arm "1  no TAG exits 2 before any cargo call (exit $rc)" $?
pub 1.2.3
[ "$rc" = 2 ] && no_cargo; arm "2  a TAG not shaped vX.Y.Z exits 2 before any cargo call (exit $rc)" $?
pub v7.7.7
[ "$rc" = 2 ] && no_cargo && grep -q "does not exist" "$T/err"; arm "3  a non-existent tag exits 2 before any cargo call (exit $rc)" $?
pub v9.9.9
[ "$rc" = 2 ] && no_cargo && grep -q "not on origin/master" "$T/err"; arm "4  a tag off origin/master exits 2 before any cargo call (exit $rc)" $?
pub v1.2.4
[ "$rc" = 2 ] && no_cargo && grep -q "does not match" "$T/err"; arm "5  Cargo.toml version != tag exits 2 before any cargo call (exit $rc)" $?

common=$(cd "$R/.git" && pwd)
pub v1.2.3 DRY_RUN=1
[ "$rc" = 0 ] && [ "$(grep -c '^argv=' "$LOG")" = 1 ] && grep -qx 'argv=publish --dry-run --locked' "$LOG"
arm "6  DRY_RUN=1 runs exactly 'cargo publish --dry-run --locked' and exits 0, untracked file in the main tree notwithstanding (exit $rc)" $?
grep -qx "common=$common" "$LOG" && ! grep -qx "top=$R" "$LOG"
arm "7  cargo runs in a worktree of THIS repository, not in the main tree and not in a copy" $?
grep -qx 'status=' "$LOG"
arm "8  the worktree cargo sees has no untracked or ignored file" $?
grep -qx 'token=UNSET' "$LOG" && ! grep -q 'token=stale' "$LOG"
arm "9  CARGO_REGISTRY_TOKEN from the environment never reaches cargo" $?

pub v1.2.3
[ "$rc" = 0 ] && [ "$(grep '^argv=' "$LOG" | tr '\n' ';')" = 'argv=publish --dry-run --locked;argv=publish --locked;' ]
arm "10 a publish dry-runs, then publishes with --locked, never --allow-dirty (exit $rc)" $?
[ "$(git -C "$R" worktree list | wc -l)" = 1 ] && [ -z "$(find "$T" -maxdepth 1 -name 'pmat-publish-*')" ]
arm "11 the worktree is removed afterwards" $?

[ "$red" = 0 ] && echo "publish-from-tag-audit: all arms green" || echo "publish-from-tag-audit: RED"
exit "$red"
