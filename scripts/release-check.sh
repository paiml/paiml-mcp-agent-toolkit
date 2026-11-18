#!/usr/bin/env bash
# AD-01 — a merged release must have BECOME a release (agentic-delivery-pmat.md section 3.5 / 9.1).
#
# Why: pmat 3.35.0's release PR (#1108) merged on 2026-09-02 and nothing tagged, released or
# published it; crates.io stayed at 3.34.0 and every gate was green, because no gate asks
# "did the version Cargo.toml declares reach the three channels". This one does.
#
# Reads the version from Cargo.toml (never a literal). If it is the latest v* tag, exit 0.
# Otherwise require, in this order, the tag, the GitHub release, and the crates.io version;
# the first missing one exits 1 naming the version; a failed read of the GitHub release or the
# registry (reason printed) exits 3 = not measured, never "missing". Runs on master after merge, never as a
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
  gh_release_leg "v$v" || return $?
  crates_leg "$v" || return $?
  echo "release-check: $v is tagged, released and on crates.io"
}

# Fetch the GitHub release for tag $1 into $2; print the read's exit status. Output (stdout+stderr)
# lands in $2. GH_RELEASE_FIXTURE=<dir> replaces the network (self-test): <dir>/out is the
# output, <dir>/rc the simulated gh exit status (default 0).
gh_fetch(){
  local rc=0
  if [ -n "${GH_RELEASE_FIXTURE:-}" ]; then
    if [ -f "$GH_RELEASE_FIXTURE/out" ]; then cp "$GH_RELEASE_FIXTURE/out" "$2"; else : > "$2"; fi
    [ -f "$GH_RELEASE_FIXTURE/rc" ] && rc=$(cat "$GH_RELEASE_FIXTURE/rc")
    echo "$rc"; return 0
  fi
  gh release view "$1" --repo "$REPO" --json tagName --jq .tagName >"$2" 2>&1 || rc=$?
  echo "$rc"
}

# The GitHub-release leg. Exit 0 present (a successful read that names the tag); 1 a real miss
# (gh exited non-zero AND said "release not found", its answer for an HTTP 404 on the tag);
# 3 anything else after retries (network error, 401/403, rate limit, empty or mismatched output):
# NOT MEASURED, never reported as "no GitHub release".
gh_release_leg(){ # $1 tag (v<version>)
  local t=$1 try rc=0 out why="" txt
  out=$(mktemp)
  for try in 1 2 3; do
    : > "$out"; rc=$(gh_fetch "$t" "$out")
    txt=$(head -c 200 "$out" | tr '\n' ' ')
    if [ "$rc" = 0 ]; then
      if [ "$(tr -d '[:space:]' < "$out")" = "$t" ]; then rm -f "${out:?}"; return 0; fi
      why="gh exit 0, unexpected output: ${txt:-empty}"
    elif grep -qi 'release not found' "$out"; then
      rm -f "${out:?}"; echo "FAIL: release-check: no GitHub release $t (gh: release not found)"; return 1
    else
      why="gh exit $rc: ${txt:-no output}"
    fi
    [ "$try" -lt 3 ] && sleep "${RELEASE_CHECK_RETRY_PAUSE:-5}"
  done
  rm -f "${out:?}"
  echo "FAIL: release-check: could not read the GitHub release $t ($why): not measured"
  return 3
}

# Sparse-index path for a crate name: 1 -> 1/, 2 -> 2/, 3 -> 3/<c>/, 4+ -> <cc>/<cc>/.
index_path(){
  local n=${1,,}
  case ${#n} in
    1) echo "1/$n";; 2) echo "2/$n";; 3) echo "3/${n:0:1}/$n";; *) echo "${n:0:2}/${n:2:2}/$n";;
  esac
}

# Fetch the sparse-index file for $CRATE into $1; print "<http code> <curl exit status>"
# (code 000 = no answer). CRATES_INDEX_FIXTURE=<dir> replaces the network (self-test):
# <dir>/$CRATE is the body, <dir>/$CRATE.status the code (default 200; absent body = 404),
# <dir>/$CRATE.curl the simulated curl exit status (default 0).
index_fetch(){
  local code crc=0
  if [ -n "${CRATES_INDEX_FIXTURE:-}" ]; then
    local f="$CRATES_INDEX_FIXTURE/$CRATE" s="$CRATES_INDEX_FIXTURE/$CRATE.status" c="$CRATES_INDEX_FIXTURE/$CRATE.curl"
    [ -f "$c" ] && crc=$(cat "$c")
    if [ -f "$s" ]; then code=$(cat "$s"); elif [ -f "$f" ]; then cp "$f" "$1"; code=200; else code=404; fi
    echo "$code $crc"; return 0
  fi
  code=$(curl -s --max-time 20 -o "$1" -w '%{http_code}' -H "User-Agent: release-check ($REPO)" \
    "${CRATES_INDEX_URL:-https://index.crates.io}/$(index_path "$CRATE")" 2>/dev/null) || crc=$?
  echo "${code:-000} $crc"
}

# The registry leg. Exit 0 published; 1 a real miss (a complete index read: curl exit 0, http 200,
# non-empty parsable body; version absent or yanked); 3 the read itself failed after retries
# (curl error, non-200, empty or unparsable body): NOT MEASURED, never reported as "not published".
crates_leg(){ # $1 version
  local v=$1 try code="000" crc=0 body verdict why="" res
  body=$(mktemp)
  for try in 1 2 3; do
    : > "$body"; res=$(index_fetch "$body"); code=${res% *}; crc=${res#* }
    if [ "$crc" != 0 ]; then why="http $code, curl $crc"
    elif [ "$code" != 200 ]; then why="http $code"
    elif [ ! -s "$body" ]; then why="http 200, empty body"
    elif ! verdict=$(jq -rs --arg v "$v" \
        'map(select(.vers == $v)) | if length == 0 then "absent" elif any(.[]; .yanked | not) then "ok" else "yanked" end' "$body" 2>/dev/null); then
      why="http 200, unparsable body"
    else
      rm -f "${body:?}"
      case $verdict in
        ok) return 0;;
        yanked) echo "FAIL: release-check: $v is yanked on crates.io (index http 200)"; return 1;;
        *) echo "FAIL: release-check: $v is not on crates.io (index http 200)"; return 1;;
      esac
    fi
    [ "$try" -lt 3 ] && sleep "${RELEASE_CHECK_RETRY_PAUSE:-5}"
  done
  rm -f "${body:?}"
  echo "FAIL: release-check: could not read the crates.io index for $CRATE ($why): not measured"
  return 3
}

# The fixture hooks are for --self-test only; a real run must never read them.
for hook in CRATES_INDEX_FIXTURE GH_RELEASE_FIXTURE; do
  if [ -n "${!hook:-}" ] && [ "${1:-}" != "--self-test" ]; then
    echo "FAIL: release-check: $hook is set outside --self-test: not measured"; exit 3
  fi
done

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
  # GitHub-release leg against a fixture gh (no network): absent, refused / rate-limited / empty, present.
  GX="$T/gh"; mkdir -p "$GX"; export GH_RELEASE_FIXTURE="$GX" RELEASE_CHECK_RETRY_PAUSE=0
  echo 1 > "$GX/rc"; echo 'release not found' > "$GX/out"
  rc=0; out=$(cd "$T" && check_version 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 1 ] && grep -q 'no GitHub release v9.9.9' <<<"$out"; } || fail "self-test: missing release rc=$rc: $out"
  echo 1 > "$GX/rc"; echo 'HTTP 401: Bad credentials (https://api.github.com/repos/x/y/releases/tags/v9.9.9)' > "$GX/out"
  rc=0; out=$(gh_release_leg v9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'HTTP 401.*not measured' <<<"$out" && ! grep -q 'no GitHub release' <<<"$out"; } || fail "self-test: unauthenticated gh rc=$rc: $out"
  echo 1 > "$GX/rc"; echo 'HTTP 403: API rate limit exceeded for installation. (https://api.github.com/repos/x/y/releases/tags/v9.9.9)' > "$GX/out"
  rc=0; out=$(gh_release_leg v9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'rate limit.*not measured' <<<"$out" && ! grep -q 'no GitHub release' <<<"$out"; } || fail "self-test: rate-limited gh rc=$rc: $out"
  echo 1 > "$GX/rc"; : > "$GX/out"
  rc=0; out=$(gh_release_leg v9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'not measured' <<<"$out" && ! grep -q 'no GitHub release' <<<"$out"; } || fail "self-test: empty failed read rc=$rc: $out"
  echo 0 > "$GX/rc"; : > "$GX/out"
  rc=0; out=$(gh_release_leg v9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'not measured' <<<"$out"; } || fail "self-test: empty successful read rc=$rc: $out"
  echo 0 > "$GX/rc"; echo v9.9.9 > "$GX/out"
  gh_release_leg v9.9.9 >/dev/null || fail "self-test: present release was not accepted"
  # Registry leg against a fixture index: present, absent, yanked, unreadable (no network).
  FX="$T/index"; mkdir -p "$FX"; export CRATES_INDEX_FIXTURE="$FX" RELEASE_CHECK_RETRY_PAUSE=0 CRATE=fx
  printf '{"name":"fx","vers":"9.9.8","yanked":false}\n{"name":"fx","vers":"9.9.9","yanked":false}\n' > "$FX/fx"
  crates_leg 9.9.9 >/dev/null || fail "self-test: present version was not accepted"
  rc=0; out=$(crates_leg 9.9.7 2>&1) || rc=$?
  { [ "$rc" = 1 ] && grep -q 'not on crates.io (index http 200)' <<<"$out"; } || fail "self-test: absent version rc=$rc: $out"
  printf '{"name":"fx","vers":"9.9.9","yanked":true}\n' > "$FX/fx"
  rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 1 ] && grep -q 'is yanked' <<<"$out"; } || fail "self-test: yanked version rc=$rc: $out"
  printf '{"name":"fx","vers":"9.9.9","yanked":false}\n' > "$FX/fx"; echo 403 > "$FX/fx.status"
  rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'http 403): not measured' <<<"$out" && ! grep -q 'not on crates.io' <<<"$out"; } || fail "self-test: refused read rc=$rc: $out"
  rm -f "$FX/fx.status"; printf 'not json\n' > "$FX/fx"
  rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'not measured' <<<"$out"; } || fail "self-test: unparsable index rc=$rc: $out"
  : > "$FX/fx"; rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'http 200, empty body): not measured' <<<"$out" && ! grep -q 'not on crates.io' <<<"$out"; } || fail "self-test: empty 200 body rc=$rc: $out"
  printf '{"name":"fx","vers":"9.9.8","yanked":false}\n' > "$FX/fx"; echo 18 > "$FX/fx.curl"
  rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'http 200, curl 18): not measured' <<<"$out" && ! grep -q 'not on crates.io' <<<"$out"; } || fail "self-test: truncated transfer rc=$rc: $out"
  rm -f "${FX:?}/fx.curl"
  rm -f "$FX/fx"; rc=0; out=$(crates_leg 9.9.9 2>&1) || rc=$?
  { [ "$rc" = 3 ] && grep -q 'http 404): not measured' <<<"$out"; } || fail "self-test: missing index file rc=$rc: $out"
  echo "SELF-TEST PASSED: tag leg fired on a 9.9.9 fixture; release leg: present ok, absent rc=1, refused/rate-limited/empty read rc=3 (not measured); registry leg: present ok, absent rc=1, yanked rc=1, refused/unparsable/404/empty/truncated read rc=3 (not measured)"
  exit 0
fi

v=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
[ -n "$v" ] || fail "no version in Cargo.toml"
# Numeric tags only: this repository carries a stray `vv2.88.0`, which `sort -V` ranks above every number.
latest_tag=$(git tag -l 'v*' | sed -E 's/^v+//' | grep -E '^[0-9]+\.[0-9]+\.[0-9]+$' | sort -V | tail -1)
echo "release-check: Cargo.toml $v · latest tag ${latest_tag:-none}"
check_version "$v"
