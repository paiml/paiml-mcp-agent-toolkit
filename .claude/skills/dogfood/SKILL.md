---
effort: medium          # MACS F4: pinned for reproducible cost/behavior
allowed-tools: Bash(cargo:*), Bash(pmat:*), Bash(gh:*), Bash(git:*), Bash(make:*), Bash(find:*), Bash(head:*), Bash(tail:*), Bash(wc:*), Bash(grep:*), Bash(diff:*), Bash(timeout:*), Bash(jq:*), Bash(python3:*), Bash(echo:*), Bash(cat:*), Bash(ls:*), Bash(rm:*), Read, Glob, Grep, Agent
description: Dogfood pmat — rebuild, install, exercise every CLI command against pmat's own repo, check output integrity + self-quality, find next work. Read-only audit; files issues for bugs.
---

# PMAT Exhaustive Dogfood — Run pmat On Itself

Rebuild and install the local `pmat`, then exercise its **entire command surface
against this repository** (pmat analyzing pmat), check output integrity and
self-quality, and surface the next work. This is the canonical "does the binary
actually work end-to-end" audit — complementary to `pmat verify` (which runs the
CI gate) and `make dogfood` (which only touches a few analyze commands).

**Lineage**: modeled on `../aprender/.claude/skills/dogfood/SKILL.md` (apr-cli
exhaustive QA), adapted to pmat's command surface and bug history.

## Context

- pmat local version: !`grep '^version' Cargo.toml | head -1`
- Current git commit: !`git rev-parse --short HEAD` on !`git branch --show-current`
- Installed pmat version: !`pmat --version 2>/dev/null || echo "not installed"`
- Subcommand count: !`pmat --help 2>/dev/null | sed -n '/Commands:/,/Options:/p' | grep -cE '^  [a-z]'`
- Lib test count: !`grep -roE '#\[test\]|#\[tokio::test\]' src/ 2>/dev/null | wc -l` test attrs

## Arguments

$ARGUMENTS

If arguments name a target directory, dogfood against it. Otherwise dogfood
against this repo (the canonical pmat-on-pmat case).

## Your Task

Run ALL gates below. For each: run the check, report **PASS / FAIL / SKIP** with
one line of evidence. Run independent gates in parallel (use Agent for fan-out).
At the end, give a single **GO / WARN / FAIL** verdict.

**Do NOT modify source files.** This is a read-only audit. If bugs are found,
file GitHub issues (`gh issue create --repo paiml/paiml-mcp-agent-toolkit`).

### CRITICAL: exit-code capture (the gotcha that fakes bugs)

`$?` after a pipe reports the LAST stage's status, not your command's. NEVER do
`pmat foo | tail; echo $?` — `tail` always exits 0, so you'll report false
passes/fails. Always:

```bash
OUT=$(timeout 60 pmat <cmd> 2>&1); EC=$?
echo "$OUT" | tail -3; echo "exit=$EC"
```

This bug has been filed-then-retracted multiple times in this repo's history.
Every gate below uses the `OUT=$(...); EC=$?` pattern.

---

## Gate 1: Build & Install

```bash
cargo install --path . --force 2>&1 | tail -5
pmat --version
```

PASS if the installed `pmat --version` matches `Cargo.toml`'s `[package] version`.
(pmat does not embed the git SHA in `--version`, so match on the semver.)
FAIL on any build error or version mismatch.

## Gate 2: Full Command Grid

Exercise every subcommand category against this repo. `--help` for mutating/slow
commands; real invocation for read-only analysis. SKIP (not FAIL) a command that
needs an arg/network it doesn't have; FAIL only on panic / `thread 'main'
panicked` / non-zero exit where success is expected.

### 2a. Templates & scaffold
```bash
for c in "list" "search rust" "generate --help" "scaffold --help" "validate --help"; do
  OUT=$(timeout 30 pmat $c 2>&1); EC=$?; echo "pmat $c -> exit=$EC :: $(echo "$OUT"|head -1)"
done
```

### 2b. Context (AST analysis)
```bash
for f in markdown json llm-optimized; do
  OUT=$(timeout 150 pmat context --format $f -o /tmp/df-ctx.$f 2>&1); EC=$?
  echo "context --format $f -> exit=$EC :: $(wc -c </tmp/df-ctx.$f 2>/dev/null) bytes"
done
```

### 2c. Query & semantic search
```bash
OUT=$(timeout 150 pmat query "error handling" --limit 5 2>&1); EC=$?; echo "query semantic -> exit=$EC :: $(echo "$OUT"|grep -c .) lines"
OUT=$(timeout 150 pmat query --regex 'fn\s+handle_\w+' --limit 5 2>&1); EC=$?; echo "query regex -> exit=$EC"
OUT=$(timeout 150 pmat query --literal 'unwrap()' --limit 5 --exclude-tests 2>&1); EC=$?; echo "query literal -> exit=$EC"
OUT=$(timeout 150 pmat query --coverage-gaps --limit 10 --exclude-tests 2>&1); EC=$?; echo "coverage-gaps -> exit=$EC"
```

### 2d. Analyze family
```bash
# NOTE valid flags (drift killed make dogfood before): complexity/churn --format = summary|full|json|sarif / summary|json|markdown|csv (NOT table); dag uses --target-nodes (NOT --top-files).
OUT=$(timeout 150 pmat analyze complexity --top-files 5 --format json 2>&1); EC=$?; echo "complexity json -> exit=$EC :: $(echo "$OUT"|jq -e 'keys' >/dev/null 2>&1 && echo OK || echo BAD-JSON)"
OUT=$(timeout 150 pmat analyze complexity --top-files 5 --format full 2>&1); EC=$?; echo "complexity full(render) -> exit=$EC"
OUT=$(timeout 150 pmat analyze churn --days 30 --top-files 5 --format json 2>&1); EC=$?; echo "churn json -> exit=$EC"
OUT=$(timeout 150 pmat analyze dag --enhanced --target-nodes 15 -o /tmp/df-dag.mmd 2>&1); EC=$?; echo "dag mermaid -> exit=$EC :: $(grep -cE 'graph|flowchart' /tmp/df-dag.mmd 2>/dev/null) header, $(wc -l </tmp/df-dag.mmd 2>/dev/null) lines"
OUT=$(timeout 150 pmat analyze satd --format json 2>&1); EC=$?; echo "satd -> exit=$EC"
OUT=$(timeout 150 pmat analyze dead-code --top-files 5 --format json 2>&1); EC=$?; echo "dead-code -> exit=$EC"
```

### 2e. Scoring family (use fast/default modes; do NOT trigger mutation or rust-score --full)
```bash
for c in "score --format json" "tdg . --format json" "repo-score --format json" "quality-gate --format json"; do
  OUT=$(timeout 150 pmat $c 2>&1); EC=$?; echo "pmat $c -> exit=$EC"
done
```

### 2f. The rest (help-smoke — confirms dispatch, no panic)
```bash
for c in refactor qdd five-whys localize work qa-work falsify kaizen roadmap comply enforce spec ci-local maintain hooks memory cache telemetry config explain diagnose serve; do
  OUT=$(timeout 20 pmat $c --help 2>&1); EC=$?; echo "$c --help -> exit=$EC :: $([ $EC -eq 0 ] && echo OK || echo FAIL)"
done
```

FAIL the gate if any command panics or a real-invocation command exits non-zero unexpectedly.

## Gate 2g: Transports — CLI is only one of three surfaces

pmat ships **three** interfaces over the same analysis core: the CLI, an MCP
stdio server, and streamable HTTP (`serve --transport http`, in the default
build since 3.32.0). A dogfood that exercises only the CLI leaves two thirds of
the product untested, and the two have already disagreed in shipped releases —
#998 had the CLI and MCP return different SATD counts for the same path.

### 2g-i. MCP stdio: does it serve, and does it agree with the CLI?

```bash
# The server is `MCP_VERSION=1 pmat`, NOT `pmat mcp` (that is manifest management).
FRAMES='{"jsonrpc":"2.0","id":1,"method":"initialize","params":{"protocolVersion":"2024-11-05","capabilities":{},"clientInfo":{"name":"dogfood","version":"1"}}}
{"jsonrpc":"2.0","id":2,"method":"tools/list","params":{}}'
printf '%s\n' "$FRAMES" | timeout 60 env MCP_VERSION=1 pmat 2>/dev/null > /tmp/mcp.out
python3 - <<'EOF'
import json
tools=None; n=0
for line in open('/tmp/mcp.out'):
    line=line.strip()
    if not line: continue
    try: m=json.loads(line)
    except Exception:
        print("NON-JSON ON STDOUT (protocol corruption):", line[:80]); continue
    n+=1
    if m.get('id')==2 and 'result' in m: tools=[t['name'] for t in m['result']['tools']]
print(f"responses={n} tools={len(tools) if tools else 'NONE'}")
if tools: print("names:", sorted(tools))
EOF
```

**FAIL** if: any non-JSON line appears on stdout (stdio transport = stdout is the
protocol; pforge shipped five `println!` lines into its own stream this way), or
`tools/list` returns nothing, or the count disagrees with `mcp.json`'s
`tool_count`.

### 2g-ii. EOF handling — the truncation class

```bash
# Write both frames then close stdin immediately. A server that ties stdout's
# lifetime to stdin's EOF drops responses it already owes (pmcp #316).
for i in 1 2 3 4 5; do
  printf '%s\n' "$FRAMES" | timeout 40 env MCP_VERSION=1 pmat 2>/dev/null \
    | grep -c '"result"'
done
```

**FAIL** if any run returns fewer results than the others. Expect the same count
5/5 — this class was intermittent (10/40) before it was fixed.

### 2g-iii. CLI vs MCP must agree

```bash
CLI=$(pmat analyze satd -p src -f json --top-files 900 | jq .total_violations)
MCP=$(printf '%s\n{"jsonrpc":"2.0","id":3,"method":"tools/call","params":{"name":"analyze_satd","arguments":{"paths":["src"]}}}\n' "$FRAMES" \
  | timeout 300 env MCP_VERSION=1 pmat 2>/dev/null \
  | python3 -c "import sys,json
for l in sys.stdin:
    try: m=json.loads(l)
    except: continue
    if m.get('id')==3 and 'result' in m:
        d=json.loads(m['result']['content'][0]['text']); r=d.get('results',d)
        print(r.get('total_violations', r.get('total_satd')))")
echo "CLI=$CLI MCP=$MCP"
```

**FAIL** if they differ. One rule, two implementations, is pmat's most repeated
defect (#998 CLI vs MCP, #831 five-whys 808 vs 39, the three SATD counters).

### 2g-iv. HTTP transport (in the default build since 3.32.0)

```bash
# HTTP is in the DEFAULT build as of 3.32.0 — it needed `--features mcp-http`
# before, so do NOT rebuild for it and do NOT let "feature not built" excuse a skip.
# It must REFUSE to serve without a token rather than start open.
timeout 15 env -u PMAT_MCP_HTTP_TOKEN pmat serve --transport http --port 9977 2>&1 | head -3
# …and start with one.
PMAT_MCP_HTTP_TOKEN=0123456789abcdef0123 timeout 30 pmat serve --transport http --port 9977 &
# LIVENESS FIRST: without this the curls below report 000 (connection refused)
# and 000 reads as "rejected", so a server that never started PASSES the gate.
# That is exactly how the stale `--http` spelling hid here: clap exited 2, both
# curls printed 000, and the gate reported auth was holding.
for _ in $(seq 20); do curl -s -o /dev/null localhost:9977/ && break; sleep 0.5; done
# NOTE: curl prints 000 for connection-refused, and 000 IS three digits — a
# `grep -E '^[0-9]{3}$'` liveness check passes against a dead port. Test != 000.
LIVE=$(curl -s -o /dev/null -w "%{http_code}" localhost:9977/)
[ "$LIVE" != "000" ] && echo "LISTENING ($LIVE)" \
  || echo "2g-iv FAIL: server never bound :9977 — curl codes below are meaningless"
curl -s -o /dev/null -w "no-token -> HTTP %{http_code}\n" -X POST localhost:9977/ \
  -H 'Content-Type: application/json' -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
curl -s -o /dev/null -w "with-token -> HTTP %{http_code}\n" -X POST localhost:9977/ \
  -H 'Authorization: Bearer 0123456789abcdef0123' -H 'Content-Type: application/json' \
  -H 'Accept: application/json, text/event-stream' \
  -d '{"jsonrpc":"2.0","id":1,"method":"tools/list","params":{}}'
kill %1 2>/dev/null
```

**FAIL** if the no-token start does not refuse, if `LISTENING` is absent, if
`no-token` is not **401**, or if `with-token` is not **200**. All four matter:
the pair `000/000` means the server never started, not that auth held, and a
gate that cannot tell those apart is testing nothing. Note the `Accept` header:
content negotiation precedes auth, so omitting it yields 406 and tells you
nothing about authentication.

**Never report this gate as skipped for a missing feature.** HTTP ships in the
default build; if it does not answer, that is a FAIL, not "not exercised".

## Gate 3: Self-Quality (pmat on pmat) — the CI-faithful gate

```bash
OUT=$(pmat verify --format json 2>&1); EC=$?
echo "$OUT" | jq '{ok, stages: [.stages[]|{name,ok}]}' 2>/dev/null || echo "$OUT" | tail -5
echo "verify exit=$EC"
```

PASS iff `ok:true` (format, complexity, satd, clippy, tests all green).
This is the strongest single signal — green here ⇒ green in CI.

## Gate 4: Output Integrity Protocols (pmat's recurring bug classes)

### P1. JSON stdout purity (v3.18.2 regression class)
`--format json` stdout must be exactly ONE jq-parseable document; decoration on
stderr only.
```bash
OUT=$(pmat analyze complexity --top-files 3 --format json 2>/dev/null); echo "$OUT" | jq -e . >/dev/null 2>&1 && echo "P1 PASS" || echo "P1 FAIL (impure json stdout)"
```

### P2. Count/rows consistency (the "Found N results" ghost bug)
A printed count must match the rows actually rendered (semantic search printed
"Found 3 results" with 0 rows in v3.18.1).
```bash
OUT=$(pmat query "zzz_nonexistent_symbol_xyz" --limit 5 2>&1); echo "$OUT" | tail -3   # count must agree with rows (0 = 0)
```

### P3. Score bounds (the perfection-score 184% bug)
Every score must sit inside its declared range; no category exceeds its max.
```bash
S=$(pmat score --format json 2>/dev/null | jq -r '.score // .value // empty' 2>/dev/null); echo "score=$S (must be 0..=100)"
```

### P4. Flag validity / no drift
Spot-check that documented flags still exist (make dogfood rotted on removed
flags). `analyze dag --help` must list `--target-nodes`; `analyze complexity
--help` must NOT accept `table`.
```bash
pmat analyze dag --help 2>&1 | grep -q -- '--target-nodes' && echo "P4a PASS" || echo "P4a FAIL"
OUT=$(pmat analyze complexity --format table 2>&1); EC=$?; [ $EC -ne 0 ] && echo "P4b PASS (table correctly rejected)" || echo "P4b NOTE (table now accepted?)"
```

### P5. Phantom subcommand
Unknown subcommand → clean error + non-zero exit (no panic, no silent success).
```bash
OUT=$(pmat definitely-not-a-real-command 2>&1); EC=$?; echo "phantom exit=$EC (expect !=0) :: $(echo "$OUT"|head -1)"
```

### P6. Cross-format consistency
`--format json` and the human format must describe the SAME data (same file
count, same top entries).
```bash
J=$(pmat analyze complexity --top-files 3 --format json 2>/dev/null | jq '.files? // .summary?'); echo "json summary: $J"
```

### P7. Dead-code file-count sanity (v3.19.2 regression class)
`analyze dead-code`'s `total_files_analyzed` must roughly match the real source
file count (what `analyze complexity` reports) — NOT a multiple of it — and must
NOT include hidden/ignored trees like `.claude/worktrees/`. (A raw `walkdir` once
descended into git-worktree copies, inflating the count ~60×.) Clear the cache
first so this measures live behavior.
```bash
rm -f .pmat/dead-code-cache.json
DC=$(pmat analyze dead-code --format json 2>/dev/null | jq '.summary.total_files_analyzed // .total_files'); \
CX=$(pmat analyze complexity --format json 2>/dev/null | jq '.summary.total_files // .total_files_analyzed // empty'); \
echo "dead-code files=$DC vs complexity files=$CX (must be same order of magnitude; FAIL if DC ≫ CX or any result path contains .claude/worktrees)"
pmat analyze dead-code --top-files 5 --format json 2>/dev/null | jq -r '.files[].path' | grep -q '.claude/worktrees' && echo "P7 FAIL (worktree paths in results)" || echo "P7 PASS"
```

### P8. `--exclude-tests` actually excludes test code (v3.19.2 regression class)
With `--exclude-tests`, no result may come from a test file or test helper —
including `include!()`-ed `*_tests_basic.rs` fragments, `setup_test*`/`create_test*`
helpers, and `*fixtures*` support files — across semantic, `--literal`, and
`--coverage-gaps` modes.
```bash
for q in "pmat query --literal unwrap() --limit 12 --exclude-tests" "pmat query --coverage-gaps --limit 15 --exclude-tests"; do
  BAD=$($q 2>/dev/null | grep -ioE 'create_test[a-z_]*|setup_test[a-z_]*|_tests_[a-z]+\.rs|coverage_fixtures\.rs|/tests/' | sort -u)
  [ -z "$BAD" ] && echo "P8 PASS ($q)" || echo "P8 FAIL ($q leaked: $BAD)"
done
```
(Known limitation, not a P8 FAIL: functions inside `#[cfg(test)] mod` blocks in
otherwise-production files with non-test-prefixed names still leak — needs
AST-level attribute detection in the index.)

## Gate 6: Provable-contract coverage (policy)

**Every fix in this repo must carry a provable contract** —
`#[provable_contracts_macros::contract("pmat-core.yaml", equation = "...")]` on
the changed/added functions (`check_compliance` for the never-panics/valid-result
invariant; `path_exists`, `score_range`, `non_empty_index`, etc. where apter).
Spot-check that functions touched by the work-in-progress diff carry one:
```bash
# Functions changed on the branch that lack a contract attribute on the line above:
git diff --name-only origin/master... | grep '\.rs$' | while read f; do
  grep -nE '^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn ' "$f" 2>/dev/null
done | head   # review: each fix fn should have a #[...contract(...)] directly above
```
PASS if the functions implementing the fixes are annotated; the CI gate
(`pmat verify` / `pv`) validates the contracts themselves.

## Gate 5: Find Next Work

```bash
pmat query --coverage-gaps --rank-by impact --limit 15 --exclude-tests 2>&1 | head -20
gh issue list --repo paiml/paiml-mcp-agent-toolkit --state open --limit 20 2>/dev/null
```

Report the top coverage gaps and open issues as candidate next work (do not fix
them in this read-only audit).

---

## Verdict

After all gates, provide:

1. **Summary table**: Gate 1–6 | PASS/FAIL/SKIP | evidence
2. **Command grid**: N commands | PASS / SKIP / FAIL counts
3. **Protocols**: P1–P8 | PASS/FAIL
4. **GO** iff Gate 1 (build), Gate 3 (`pmat verify` ok:true), and all protocols pass, the command grid has zero FAIL, and fix functions carry provable contracts (Gate 6).
5. **WARN** for soft issues only (SKIPs, cosmetic output, pre-existing latent findings) — no panics, no integrity violations.
6. **FAIL** on: build/install failure, `pmat verify` red, any command panic, JSON-impurity (P1), count/rows contradiction (P2), out-of-range score (P3), phantom-subcommand silent success (P5), dead-code worktree inflation (P7), `--exclude-tests` leaking test files (P8), or exit-code lies.

If bugs found, file them:
```bash
gh issue create --repo paiml/paiml-mcp-agent-toolkit \
  --title "pmat <cmd>: <one-line defect>" \
  --body "Repro + expected vs actual + exit code (captured via OUT=\$(...); EC=\$?)."
```

## Cleanup

```bash
rm -f /tmp/df-ctx.* /tmp/df-dag.mmd
```
