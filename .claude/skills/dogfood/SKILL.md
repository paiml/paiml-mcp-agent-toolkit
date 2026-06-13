---
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

1. **Summary table**: Gate 1–5 | PASS/FAIL/SKIP | evidence
2. **Command grid**: N commands | PASS / SKIP / FAIL counts
3. **Protocols**: P1–P6 | PASS/FAIL
4. **GO** iff Gate 1 (build), Gate 3 (`pmat verify` ok:true), and all protocols pass, and the command grid has zero FAIL.
5. **WARN** for soft issues only (SKIPs, cosmetic output, pre-existing latent findings) — no panics, no integrity violations.
6. **FAIL** on: build/install failure, `pmat verify` red, any command panic, JSON-impurity (P1), count/rows contradiction (P2), out-of-range score (P3), phantom-subcommand silent success (P5), or exit-code lies.

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
