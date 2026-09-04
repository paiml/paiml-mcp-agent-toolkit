#!/usr/bin/env bash
# AD-02 — dog-food the bytes crates.io serves (docs/specifications/agentic-delivery-pmat.md section 3.6 / 9.2).
#
# The pre-publish probe (crate-release-dogfood) reads the packaged tarball; the release gate
# (scripts/dogfood-use.sh) reads a locally built binary. Neither touches what a stranger
# obtains with `cargo install`. This does: it checks the registry entry, installs the given
# version from crates.io into a throwaway root, checks the binary reports that version and
# answers --help, runs the release gate against it, and writes a receipt.
#
#   bash scripts/dogfood-published.sh 3.36.0      # exit 0 = GO, receipt written; 1 = a leg failed; 2 = usage
#   bash scripts/dogfood-published.sh 9.9.9       # the control: a version the registry does not have fails at P7
#   RECEIPT=<path> overrides docs/audits/release-<v>-dogfood-published.md (the release-chain receipt release-<v>-dogfood.md links to it) · CRATE=<name> overrides pmat
#
# The receipt records what the INSTALLED binary says about itself. Since 3.34.0 a crates.io
# build reports `commit: unknown` (CRUX-21), so the artifact is pinned by the registry's
# size and created stamp, which a local build cannot forge either.
set -euo pipefail
fail(){ echo "FAIL: dogfood-published: $*"; exit 1; }
V="${1:-}"; [ -n "$V" ] || { echo "usage: $0 <version>"; exit 2; }
[[ "$V" =~ ^[0-9]+\.[0-9]+\.[0-9]+$ ]] || fail "'$V' is not a version"
CRATE="${CRATE:-pmat}"
RECEIPT="${RECEIPT:-docs/audits/release-$V-dogfood-published.md}"
ROOT=$(mktemp -d)
cleanup(){ case "${ROOT:-}" in "${TMPDIR:-/tmp}"/tmp.*|/tmp/tmp.*) [ -d "$ROOT" ] && rm -rf -- "$ROOT";; esac; }; trap cleanup EXIT

# P7 first: the registry must carry the version, un-yanked. A fabricated version stops here.
meta=$(curl -sf --max-time 20 "https://crates.io/api/v1/crates/$CRATE/$V" -H "User-Agent: dogfood-published ($CRATE)") \
  || fail "$CRATE $V is not on crates.io"
jq -e --arg v "$V" '.version.num == $v and (.version.yanked | not)' <<<"$meta" >/dev/null 2>&1 || fail "$CRATE $V is yanked or the registry answered for another version"
size=$(jq -r '.version.crate_size' <<<"$meta"); created=$(jq -r '.version.created_at' <<<"$meta")

# P5: install from the registry — the exact path a consumer takes.
echo "dogfood-published: cargo install $CRATE --version $V --locked --root $ROOT"
cargo install "$CRATE" --version "$V" --locked --root "$ROOT" >"$ROOT/install.log" 2>&1 \
  || { tail -20 "$ROOT/install.log"; fail "cargo install $CRATE $V failed"; }
BIN="$ROOT/bin/$CRATE"; [ -x "$BIN" ] || fail "install produced no executable at $BIN"
ver_out=$("$BIN" --version 2>&1 | head -3)
grep -q "^$CRATE $V" <<<"$ver_out" || fail "installed binary does not report $V: $ver_out"
"$BIN" --help >/dev/null 2>&1 || fail "installed binary does not answer --help"
# `|| true`: a crates.io build has no git metadata (CRUX-21) and this line is then absent.
commit=$(grep -oE 'commit: [0-9a-f]{7,40}' <<<"$ver_out" | head -1 || true)

# The release gate against the installed binary, not a local build.
BIN="$BIN" bash scripts/dogfood-use.sh > "$ROOT/dogfood-use.log" 2>&1 || { tail -20 "$ROOT/dogfood-use.log"; fail "dogfood-use.sh failed against the installed $V"; }
checks=$(sed 's/\x1b\[[0-9;]*m//g' "$ROOT/dogfood-use.log" | grep -oE '[0-9]+ checks, [0-9]+ failure\(s\)' | tail -1 || true)
[ -n "$checks" ] || fail "dogfood-use.sh printed no check summary"

mkdir -p "$(dirname "$RECEIPT")"
{
  echo "# Post-publish dog-food — $CRATE $V"
  echo
  echo "| leg | result |"
  echo "|---|---|"
  echo "| P7 registry | \`$CRATE $V\` on crates.io, not yanked; size $size bytes, created $created |"
  echo "| P5 install | \`cargo install $CRATE --version $V --locked\` into a throwaway root: executable present |"
  echo "| --version | \`$(head -1 <<<"$ver_out")\` — ${commit:-no commit line (CRUX-21: the crates.io build carries no git metadata)} |"
  echo "| --help | answered |"
  echo "| release gate | \`scripts/dogfood-use.sh\` against the INSTALLED binary: $checks |"
  echo
  echo "Written by \`scripts/dogfood-published.sh\` (AD-02, docs/specifications/agentic-delivery-pmat.md section 3.6). The registry size and stamp pin the artifact; a local build cannot forge them."
} > "$RECEIPT"
echo "GO: $CRATE $V — $checks; receipt $RECEIPT"
