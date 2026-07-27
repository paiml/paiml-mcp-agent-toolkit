#!/usr/bin/env bash
# dogfood-use: use the freshly-built pmat release binary on real data.
#
# Contract (see ~/.claude/skills/dogfood):
#   $BIN  - the built release binary
#   $WORK - a scratch directory
# Exit non-zero if the tool misbehaves on real input.
#
# This is deliberately NOT a re-run of the test suite. It drives the shipped
# binary against this repo's own source tree and roadmap, which is the largest
# realistic input we have on hand.
set -uo pipefail

BIN="${BIN:?BIN must point at the built pmat binary}"
WORK="${WORK:?WORK must be a scratch directory}"
REPO="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "$WORK" in
  */..*) printf 'refusing WORK containing "..": %s\n' "$WORK" >&2; exit 2 ;;
  /*)    : ;;
  *)     printf 'WORK must be an absolute path: %s\n' "$WORK" >&2; exit 2 ;;
esac

fails=0
step() { printf '\n\033[1m▸ %s\033[0m\n' "$1"; }
ok()   { printf '  \033[0;32m✓\033[0m %s\n' "$1"; }
bad()  { printf '  \033[0;31m✗\033[0m %s\n' "$1"; fails=$((fails + 1)); }

mkdir -p "$WORK"

# --- 1. the binary identifies itself -----------------------------------------
step "version"
ver="$("$BIN" --version 2>/dev/null)" || bad "--version exited non-zero"
[[ "$ver" == *pmat* ]] && ok "reports: $ver" || bad "unexpected --version output: $ver"

# --- 2. complexity analysis over this repo's real source ----------------------
step "analyze complexity (real source tree)"
if out="$("$BIN" analyze complexity --path "$REPO" --format summary 2>&1)"; then
  if grep -qiE "median|cyclomatic" <<<"$out"; then
    ok "produced complexity metrics over $(find "$REPO/src" -name '*.rs' | wc -l) files"
  else
    bad "no recognisable metrics in output"
  fi
else
  bad "analyze complexity exited non-zero"
fi

# --- 3. roadmap validation: the capability this release changed ---------------
step "work validate (valid roadmap)"
mkdir -p "$WORK/good/docs/roadmaps"
cat >"$WORK/good/docs/roadmaps/roadmap.yaml" <<'YAML'
roadmap_version: "1.0"
github_enabled: false
roadmap:
  - id: "DOGFOOD-1"
    title: Exercise the shipped binary
    status: completed
  - id: "DOGFOOD-2"
    title: "Quoted: because it needs it"
    status: inprogress
YAML
if (cd "$WORK/good" && "$BIN" work validate >"$WORK/good.out" 2>&1); then
  ok "accepted a valid roadmap"
else
  bad "rejected a valid roadmap:"; sed 's/^/      /' "$WORK/good.out"
fi

# A valid roadmap must need no migration AND must not be rewritten.
step "work migrate (must not touch a clean roadmap)"
before="$(md5sum "$WORK/good/docs/roadmaps/roadmap.yaml" | cut -d' ' -f1)"
(cd "$WORK/good" && "$BIN" work migrate >"$WORK/migrate.out" 2>&1)
after="$(md5sum "$WORK/good/docs/roadmaps/roadmap.yaml" | cut -d' ' -f1)"
if [[ "$before" == "$after" ]]; then
  ok "left a clean roadmap byte-identical"
else
  bad "rewrote a roadmap that needed no migration"
fi
if compgen -G "$WORK/good/docs/roadmaps/*.bak" >/dev/null; then
  bad "created a backup for a roadmap needing no migration"
else
  ok "created no spurious backup"
fi

# --- 4. multi-error reporting (the #628 scenario) -----------------------------
step "work validate (reports every broken row in one pass)"
mkdir -p "$WORK/bad/docs/roadmaps"
cat >"$WORK/bad/docs/roadmaps/roadmap.yaml" <<'YAML'
roadmap_version: "1.0"
roadmap:
  - id: OK-1
    title: fine
    status: planned
  - id: BAD-TYPE
    title: t
    item_type: verification
    status: planned
  - id: BAD-STATUS
    title: t
    status: obsolete
  - id: NO-STATUS
    title: t
YAML
(cd "$WORK/bad" && NO_COLOR=1 "$BIN" work validate >"$WORK/bad.out" 2>&1)
if grep -q "BAD-TYPE" "$WORK/bad.out" &&
   grep -q "BAD-STATUS" "$WORK/bad.out" &&
   grep -q "NO-STATUS" "$WORK/bad.out"; then
  ok "reported all three violation classes in one run"
else
  bad "did not report every broken row:"; sed 's/^/      /' "$WORK/bad.out"
fi

# Every doc path and command the diagnostics cite must actually resolve.
# pmat does not honour NO_COLOR, so its output carries ANSI escapes; matching
# the path shape with awk avoids both the escapes and the trailing sentence
# period. The docs_checked guard is deliberate: an earlier version used a tr
# set whose '-' formed an invalid range, tr aborted, and this loop passed
# without examining a single path.
step "diagnostics cite only things that exist"
docs_checked=0
while read -r doc; do
  docs_checked=$((docs_checked + 1))
  if [[ -f "$REPO/$doc" ]]; then ok "doc exists: $doc"; else bad "cites missing doc: $doc"; fi
done < <(awk 'match($0, /docs\/[A-Za-z0-9_.\/-]+[.]md/) { print substr($0, RSTART, RLENGTH) }' "$WORK/bad.out" | sort -u)
if [[ "$docs_checked" -eq 0 ]]; then
  bad "extracted no doc paths from the diagnostics - the check would pass vacuously"
fi

while read -r cmd; do
  if "$BIN" $cmd --help >/dev/null 2>&1; then ok "command exists: pmat $cmd"
  else bad "cites non-existent command: pmat $cmd"; fi
done < <(grep -oE "pmat work [a-z]+" "$WORK/bad.out" | sed 's/^pmat //' | sort -u)

# --- 5. status vocabulary is self-consistent ---------------------------------
step "work list-statuses"
if statuses="$("$BIN" work list-statuses 2>&1)"; then
  missing=0
  for s in planned inprogress blocked review completed cancelled; do
    grep -q "$s" <<<"$statuses" || { bad "vocabulary omits '$s'"; missing=1; }
  done
  [[ $missing -eq 0 ]] && ok "all six canonical statuses documented"
else
  bad "work list-statuses exited non-zero"
fi

# --- 6. context generation over real source ----------------------------------
step "context (real Rust project)"
mkdir -p "$WORK/proj/src"
printf 'fn main() { println!("hi"); }\n' >"$WORK/proj/src/main.rs"
printf 'pub fn add(a: i32, b: i32) -> i32 { a + b }\n' >"$WORK/proj/src/lib.rs"
{ printf '%s\n' '[package]'; printf '%s\n' 'name = "ctxprobe"' 'version = "0.1.0"' 'edition = "2021"'; } >"$WORK/proj/Cargo.toml"
if (cd "$WORK/proj" && timeout 300 "$BIN" context --output "$WORK/context.md" >/dev/null 2>&1); then
  sz=$(wc -c <"$WORK/context.md" 2>/dev/null || echo 0)
  if [[ "$sz" -gt 100 ]]; then ok "generated context ($sz bytes)"
  else bad "context suspiciously small ($sz bytes)"; fi
else
  bad "context generation failed or timed out"
fi

printf '\n'
if [[ $fails -eq 0 ]]; then
  printf '\033[0;32m✅ dogfood-use PASSED\033[0m - the shipped binary behaves on real input\n'
  exit 0
fi
printf '\033[0;31m❌ dogfood-use FAILED\033[0m - %d check(s) failed\n' "$fails"
exit 1
