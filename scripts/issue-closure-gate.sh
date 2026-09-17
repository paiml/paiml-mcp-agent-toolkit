#!/usr/bin/env bash
# issue-closure-gate.sh — no pmat code path closes a GitHub issue (PMAT-900001).
# Contract: contracts/pmat-issue-closure-v1.yaml.
#
# An issue closes when a merged PR body carries its own `Closes #N` line, and in no other way
# pmat starts. This gate finds every place pmat could close one itself and FAILS on any call site.
#
# Two legs, because one tool cannot see the whole tree:
#
#   query  `pmat query` over Rust FUNCTION BODIES. It indexes functions only, so it cannot see a
#          const, a script, YAML or a Makefile. A hit inside a function annotated #[test] /
#          #[tokio::test] is a test and is skipped. A hit inside one of the named DEFINITIONS is
#          counted apart: the client API that could close an issue if called. Any other hit is
#          a CALL SITE, and fails the gate.
#   text   `git grep` over every tracked file except narrative docs (docs/**, *.md) and this gate's
#          own files: scripts, YAML, prompts, Makefiles, and Rust consts. In a .rs file a hit at or
#          after the file's first #[cfg(test)] line, or in a *test*.rs file, is a test and is skipped.
#
# Anti-vacuity: a leg that scanned 0 files FAILS, and every named definition must be FOUND by the
# query leg — a query that finds none of them is blind, not clean.
#
# Usage: scripts/issue-closure-gate.sh PMAT_BIN [--root DIR]
#        scripts/issue-closure-gate.sh PMAT_BIN --self-test   plant each defect in a scratch repo;
#                                                             every arm must go RED, the clean one GREEN
# PMAT_BIN comes first, as in every control script scripts/gate.sh runs (gate-control arm 12).
# Exit:  0 no call site · 1 a call site, a blind leg, or (self-test) an arm disagreed · 2 usage
set -uo pipefail

# The client API that can close an issue if called: <file>:<function>. Uncalled today (E1, D5).
DEFINITIONS=(
  "src/services/github_client.rs:update_issue"
  "src/services/github_client.rs:close_issue"
  "src/services/github_client.rs:reopen_issue"
  "src/services/github_issues_client.rs:update_issue"   # orphan file: not compiled (docs/status/orphan-files-ledger.md)
)

# Query leg: <mode> <pattern>, one per line. mode is literal|regex (`pmat query --literal|--regex`).
QUERY_PATTERNS='literal|.close_issue(
literal|.update_issue(
regex|[(,]\s*Some\("closed"\)
literal|closeIssue
regex|"issue"\s*,\s*"close"
regex|gh\s+issue\s+close
regex|state=closed
regex|\.state\((\w+::)*IssueState::Closed\)'

# Text leg: one ERE (git grep -E).
TEXT_ERE='gh[[:space:]]+issue[[:space:]]+close|closeIssue|state=closed|"issue",[[:space:]]*"close"|(-X|--method)[[:space:]]*PATCH.*"?state"?[[:space:]]*[:=][[:space:]]*"?closed|\.close_issue\('
TEXT_PATHSPEC=(':(exclude)docs/**' ':(exclude)*.md' ':(exclude)scripts/issue-closure-gate.sh'
               ':(exclude)contracts/pmat-issue-closure-v1.yaml')

usage() { sed -n '2,/^set -uo pipefail$/p' "$0" | sed '$d'; }

# is_test_fn FILE LINE — the function pmat reports at LINE carries a #[test]-like attribute. From
# LINE (which may be a doc comment) find the `fn` line, then walk up over attributes, comments and
# blank lines.
is_test_fn() {
  awk -v want="$2" '
    { line[NR] = $0 }
    END {
      fnl = 0
      for (i = want; i <= NR && i <= want + 40; i++) if (line[i] ~ /(^|[ \t])fn [A-Za-z_]/) { fnl = i; break }
      if (!fnl) exit 1
      for (i = fnl - 1; i >= 1; i--) {
        l = line[i]; sub(/^[ \t]+/, "", l)
        if (l ~ /^#\[(tokio::)?test/ || l ~ /^#\[rstest/) exit 0
        if (l !~ /^(#\[|\/\/|$)/) exit 1
      }
      exit 1
    }' "$1"
}

gate() {
  local pmat="$1" root="$2" fail=0
  cd "$root" || { echo "issue-closure-gate: --root $root is not a directory" >&2; return 2; }

  # ── query leg ──
  local rs_files; rs_files=$(git ls-files -- '*.rs' 2>/dev/null | wc -l)
  echo "LEG query  scanned_rs_files=$rs_files"
  if [ "$rs_files" -eq 0 ]; then echo "  FAIL  VACUOUS: 0 Rust files tracked — the query leg judged nothing"; fail=1; fi
  local hits="" mode pat out
  while IFS='|' read -r mode pat; do
    [ -n "$mode" ] || continue
    if ! out=$("$pmat" query "--$mode" "$pat" --format json --limit 1000 2>/dev/null); then
      echo "  FAIL  pmat query --$mode '$pat' did not run"; fail=1; continue
    fi
    hits+=$(printf '%s' "$out" | jq -r '.[]? | "\(.file_path)\t\(.function_name)\t\(.start_line)"' 2>/dev/null)$'\n'
  done <<< "$QUERY_PATTERNS"
  local defs_seen="" calls=0 tests=0 f fn ln
  while IFS=$'\t' read -r f fn ln; do
    [ -n "$f" ] || continue
    if printf '%s\n' "${DEFINITIONS[@]}" | grep -qxF "$f:$fn"; then
      case " $defs_seen " in *" $f:$fn "*) ;; *) defs_seen+=" $f:$fn" ;; esac
    elif [ -f "$f" ] && is_test_fn "$f" "$ln"; then
      tests=$((tests + 1))
    else
      echo "  FAIL  CALL SITE $f:$ln $fn — a pmat code path that can close a GitHub issue"; calls=$((calls + 1)); fail=1
    fi
  done < <(printf '%s' "$hits" | sort -u)
  local d found=0
  for d in "${DEFINITIONS[@]}"; do
    # Positive control: the definition must exist AND be seen. A pattern set that no longer
    # reaches a definition it names cannot be trusted to reach a call site either.
    if [ -f "${d%%:*}" ] && ! "$pmat" query --literal "fn ${d##*:}(" --format json --limit 1000 2>/dev/null \
         | jq -e --arg f "${d%%:*}" --arg n "${d##*:}" 'any(.[]?; .file_path == $f and .function_name == $n)' >/dev/null; then
      echo "  FAIL  BLIND: definition $d is on disk but pmat query does not return it"; fail=1
    elif [ ! -f "${d%%:*}" ]; then
      echo "  FAIL  DEFINITION GONE: $d — update DEFINITIONS and the contract in the same change"; fail=1
    else
      found=$((found + 1))
    fi
  done
  echo "  definitions=$found/${#DEFINITIONS[@]} (pattern-reached:${defs_seen:- none}) call_sites=$calls test_hits_skipped=$tests"

  # ── text leg ──
  local txt_files; txt_files=$(git ls-files -- "${TEXT_PATHSPEC[@]}" 2>/dev/null | wc -l)
  echo "LEG text   scanned_files=$txt_files"
  if [ "$txt_files" -eq 0 ]; then echo "  FAIL  VACUOUS: 0 files in the text leg's pathspec"; fail=1; fi
  local tcalls=0 tskip=0 line file lno first
  while IFS= read -r line; do
    [ -n "$line" ] || continue
    file=${line%%:*}; lno=${line#*:}; lno=${lno%%:*}
    if [[ "$file" == *.rs ]]; then
      first=$(grep -n '^[[:space:]]*#\[cfg(test)\]' "$file" | head -1 | cut -d: -f1)
      if [[ "$file" == *test*.rs ]] || { [ -n "$first" ] && [ "$lno" -ge "$first" ]; }; then
        tskip=$((tskip + 1)); continue
      fi
      # A hit inside a named definition's function is counted by the query leg.
    fi
    echo "  FAIL  TEXT HIT ${line:0:200}"; tcalls=$((tcalls + 1)); fail=1
  done < <(git grep -nIE "$TEXT_ERE" -- "${TEXT_PATHSPEC[@]}" 2>/dev/null)
  echo "  hits=$tcalls test_hits_skipped=$tskip"

  if [ "$fail" -eq 0 ]; then echo "issue-closure-gate: PASS"; else echo "issue-closure-gate: FAIL"; fi
  return "$fail"
}

# ── self-test ──
fixture() {
  local d="$1"
  mkdir -p "$d/src/services" "$d/scripts"
  cat > "$d/src/services/github_client.rs" <<'RS'
pub struct GitHubClient;
impl GitHubClient {
    pub async fn update_issue(&self, n: u64, state: Option<&str>) -> Result<(), ()> {
        let _ = (n, state == Some("closed"));
        Ok(())
    }
    pub async fn close_issue(&self, n: u64) -> Result<(), ()> {
        self.update_issue(n, Some("closed")).await
    }
    pub async fn reopen_issue(&self, n: u64) -> Result<(), ()> {
        self.update_issue(n, Some("open")).await
    }
}
RS
  cat > "$d/src/services/github_issues_client.rs" <<'RS'
pub async fn update_issue(n: u64) -> u64 {
    n
}
RS
  cat > "$d/src/lib.rs" <<'RS'
pub mod services;
pub fn unrelated() -> u32 {
    7
}
#[cfg(test)]
mod tests {
    #[tokio::test]
    async fn a_test_may_call_the_closer() {
        let c = crate::services::github_client::GitHubClient;
        let _ = c.close_issue(5).await;
    }
}
RS
  printf 'pub mod github_client;\n' > "$d/src/services/mod.rs"
  printf '#!/usr/bin/env bash\necho ok\n' > "$d/scripts/ok.sh"
  git -C "$d" init -q && git -C "$d" add -A
}

self_test() {
  local pmat="$1" base bad=0
  base=$(mktemp -d) || return 2
  arm() { # arm <name> <want exit> <reason the log must name> <setup commands, run in the fixture dir>
    local name="$1" want="$2" reason="$3" setup="$4" d got
    d="$base/$name"; mkdir -p "$d"
    if [ "$name" != vacuous ]; then fixture "$d"; else git -C "$d" init -q; fi
    (cd "$d" && eval "$setup") && git -C "$d" add -A
    (gate "$pmat" "$d") > "$d.log" 2>&1; got=$?
    if [ "$got" = "$want" ]; then
      echo "  ok    $name: exit $got"
      grep '^  FAIL' "$d.log" | sed 's/^  FAIL/          because/'
      grep -q "$reason" "$d.log" || { echo "  WRONG $name: the log does not name '$reason'"; bad=1; }
    else echo "  WRONG $name: exit $got, want $want"; sed 's/^/        /' "$d.log"; bad=1; fi
  }
  echo "issue-closure-gate --self-test"
  arm clean               0 'issue-closure-gate: PASS' ':'
  arm rust-call           1 'CALL SITE src/lib.rs:.* finish' "printf 'pub async fn finish(c: &crate::services::github_client::GitHubClient) {\n    let _ = c.close_issue(7).await;\n}\n' >> src/lib.rs"
  arm gh-args             1 'CALL SITE src/lib.rs:.* finish' "printf 'pub fn finish() {\n    let _ = std::process::Command::new(\"gh\").args([\"issue\", \"close\", \"7\"]).status();\n}\n' >> src/lib.rs"
  arm script              1 'TEXT HIT scripts/ok.sh' "printf 'gh issue close 7\n' >> scripts/ok.sh"
  arm rust-const          1 'TEXT HIT src/advice.rs' "printf 'const ADVICE: &str = \"gh issue close 7\";\n' > src/advice.rs"
  arm definition-renamed  1 'DEFINITION GONE\|BLIND: definition src/services/github_client.rs:close_issue' "sed -i 's/fn close_issue/fn shut_issue/' src/services/github_client.rs"
  arm vacuous             1 'VACUOUS: 0 Rust files' ':'
  rm -rf "$base"
  if [ "$bad" -eq 0 ]; then echo "issue-closure-gate --self-test: every arm as expected"; fi
  return "$bad"
}

command -v jq >/dev/null || { echo "issue-closure-gate: jq is required" >&2; exit 2; }
case "${1:-}" in
  -h|--help) usage; exit 0 ;;
  ""|-*) usage >&2; exit 2 ;;
esac
PMAT="$1"; shift
case "$PMAT" in /*) ;; *) PMAT="$(cd "$(dirname "$PMAT")" && pwd)/$(basename "$PMAT")" ;; esac
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
while [ $# -gt 0 ]; do
  case "$1" in
    --self-test) self_test "$PMAT"; exit $? ;;
    --root) ROOT="${2:-}"; shift 2 ;;
    *) echo "issue-closure-gate: unknown argument '$1'" >&2; exit 2 ;;
  esac
done
gate "$PMAT" "$ROOT"
