//! The literal bytes `pmat init` writes.
//!
//! Every template in here is either (a) `include_str!`d from a file this
//! repository already commits and already tests, or (b) derived from a schema
//! that was *measured*, not guessed. Nothing is invented: an artifact whose
//! format nobody has defined is refused by [`super::plan`] instead of being
//! filled in with something plausible-looking, on the precedent set by
//! `pmat agy sync` (#984).

/// The dual-client quality-feedback hook, embedded from the copy this repo
/// commits and exercises in `qa_mcp_sweep::agent_quality_hook_tests`.
///
/// `include_str!` rather than a string literal on purpose: it resolves at
/// COMPILE time, so if the script is deleted or renamed the build breaks
/// instead of `pmat init` silently emitting a stale fork of it. The bytes a
/// user gets are therefore the exact bytes this repo's own tests run.
pub const QUALITY_FEEDBACK_HOOK: &str =
    include_str!("../../../.agents/hooks/pmat-quality-feedback.sh");

/// Antigravity / `.agents` hook manifest.
///
/// Shape verified against `.agents/hooks.json` in this repo and against
/// PMAT-INIT-002 claim 1, which names the `PreToolUse` schema explicitly.
///
/// The command is relative (`./.agents/...`) because that is the only form
/// this repo's own manifest uses and no `$AGENTS_PROJECT_DIR`-style variable
/// is documented anywhere for this client. It therefore resolves only when the
/// client's cwd is the project root — and because BOTH clients treat a hook
/// that fails to launch as *allow*, a wrong cwd is a silent no-op rather than
/// an error. `AGENTS.md` says so in as many words; a bootstrap that hides that
/// would be selling a gate that is really a feedback loop.
pub const AGY_HOOKS_JSON: &str = r#"{
  "pmat-quality-feedback": {
    "PreToolUse": [
      {
        "matcher": "write_file|code_execution",
        "hooks": [
          {
            "type": "command",
            "command": "./.agents/hooks/pmat-quality-feedback.sh antigravity"
          }
        ]
      }
    ]
  }
}
"#;

/// Claude Code settings with the same entrypoint in `claude` mode.
///
/// `$CLAUDE_PROJECT_DIR` is used because Claude Code documents it and this
/// repo's own `.claude/settings.json` uses it — it removes the cwd hazard that
/// the `.agents` manifest above still carries.
pub const CLAUDE_SETTINGS_JSON: &str = r#"{
  "$schema": "https://json.schemastore.org/claude-code-settings.json",
  "hooks": {
    "PreToolUse": [
      {
        "matcher": "Write|Edit",
        "hooks": [
          {
            "type": "command",
            "command": "$CLAUDE_PROJECT_DIR/.agents/hooks/pmat-quality-feedback.sh claude"
          }
        ]
      }
    ]
  }
}
"#;

/// The MCP server registration, in the `{"mcpServers": {…}}` client-config
/// shape both Antigravity and Claude Code consume.
///
/// **This is the whole reason the ticket exists.** The template this repo
/// shipped named `cargo run --bin pmat -- serve --transport stdio`, which is
/// broken twice over and was never once executed:
///
/// * `stdio` is not an accepted `--transport` value — `pmat serve --transport
///   stdio` exits 2 at clap parse with `invalid value 'stdio'`, writing zero
///   bytes to stdout, so it cannot speak MCP even in principle; and
/// * `cargo run` requires a Cargo workspace in cwd, so in the only situation
///   that matters — a user's own repo — it exits 101 with "could not find
///   Cargo.toml" before pmat is even reached.
///
/// `pmat --mode mcp` is the invocation that works, measured end to end:
/// `initialize` and `tools/list` both return valid JSON-RPC, 16 tools,
/// `serverInfo` = paiml-mcp-agent-toolkit, protocol 2024-11-05, and zero
/// non-protocol bytes on stdout — from a cwd outside any Cargo project.
/// `mcp_config_names_a_command_that_actually_speaks_mcp` in `tests.rs` spawns
/// whatever this constant names and refuses to pass on anything less.
pub const MCP_CONFIG_JSON: &str = r#"{
  "mcpServers": {
    "pmat": {
      "command": "pmat",
      "args": ["--mode", "mcp"],
      "env": {}
    }
  }
}
"#;

/// A skill with the frontmatter keys the `.agents`/`.claude` skills in this
/// repo actually use: `effort`, `allowed-tools`, `description`.
///
/// `effort: xhigh` matches the pin the MACS spec's §4-F4 table gives
/// `pmat-quality` ("adversarial verification"), and `xhigh` is inside the
/// `{low, medium, high, xhigh}` set CB-1650 enforces — the session-only values
/// `max` and `ultracode` are rejected there by design.
///
/// The body deliberately contains no pmat-internal process. `.agents/rules/*`
/// in this repository is a byte-identical vendored copy of
/// `docs/agent-instructions/*` and is pmat-specific (its files are about
/// pmat's own coverage push and its work-command UX); emitting that into a
/// stranger's repository would be shipping our internal memos as their rules.
pub const SKILL_MD: &str = r#"---
effort: xhigh
allowed-tools: Bash(pmat:*), Bash(git:*), Read, Glob, Grep
description: Run pmat's quality gate on the current change and act on what it reports — complexity, SATD, dead code, coverage — before asking a human to review.
---

# pmat quality pass

Deterministic first. Every check below is a program, not a judgment call; spend
no model tokens deciding things a gate already decided.

## The gate

```bash
pmat verify --format json      # format + complexity + SATD + clippy + tests
```

`ok: true` means this change would survive the same checks CI runs. Anything
else is a stop: fix it, do not explain it.

## Narrowing a failure

```bash
pmat quality-gate --file <path> --format json   # one file, sub-second
pmat analyze complexity --path <path> --top-files 10
pmat analyze satd --path <path>
pmat analyze dead-code --path <path>
```

## Finding the code to change

Prefer `pmat query` over grep: it returns functions ranked by relevance with
grade, complexity and fault annotations attached.

```bash
pmat query "<what you are looking for>" --limit 10
pmat query "<name>" --include-source --limit 1
```

## Rules

- A failing gate is never "flaky until proven otherwise" — reproduce it.
- Do not delete or `#[ignore]` a test to make a gate pass.
- If a check is wrong, fix the check in the same change and say why.
"#;

/// The root rules file (`AGENTS.md`), the cross-client convention Claude Code,
/// Antigravity and others all read.
///
/// It states the hook's limits rather than advertising it as enforcement,
/// because that is what is true: both clients treat a crashed, missing or slow
/// hook as an approval, so with `pmat` off the PATH nothing is checked at all.
pub const AGENTS_MD: &str = r#"# Agent rules

Generated by `pmat init`. Edit freely — `pmat init` will not overwrite a file
you have changed unless you pass `--force`.

## Workspace compliance

This workspace was bootstrapped by `pmat init`. Check that the files it wrote
are still coherent — and that the ones you have edited since still parse:

```bash
pmat comply check
```

That covers the generated `.agents/` layout, the MCP registration, and the
skill frontmatter. Run it after you change anything under `.agents/`.

## Quality gate

Run this before you finish a change, and again before you ask for review:

```bash
pmat verify --format json
```

`ok: true` means the change passes the same format / complexity / SATD /
clippy / test checks a CI gate would run. Treat anything else as blocking.

Narrower, faster checks while you work:

```bash
pmat quality-gate --file <path> --format json
pmat analyze complexity --path <path> --top-files 10
pmat analyze satd --path <path>
pmat analyze dead-code --path <path>
```

## Searching this repository

Use `pmat query` instead of grep when you want to know *what exists*: it
returns functions ranked by relevance, annotated with grade, complexity and
fault patterns.

```bash
pmat query "error handling" --limit 10
pmat query "<function name>" --include-source --limit 1
pmat query --regex "fn\s+handle_\w+" --limit 10     # rg -e
pmat query --literal "unwrap()" --limit 10          # rg -F
```

## The pre-tool-use hook is FEEDBACK, not a gate

`.agents/hooks/pmat-quality-feedback.sh` runs `pmat quality-gate` on Rust files
before a write. Understand what it can and cannot do before you rely on it:

- If `pmat` is missing, slow, or errors, **both** Claude Code and Antigravity
  treat the hook as an approval. A hook that allows on failure is not
  enforcement.
- Claude Code blocks on exit 2 only; exit 1 blocks nothing.
- Antigravity blocks only on `{"decision": "deny"}`; unparseable output is an
  approval.
- The Antigravity manifest invokes the hook by a **relative** path, so it runs
  only when the client's working directory is the project root. Anywhere else
  the hook silently does not run. Claude Code's manifest uses
  `$CLAUDE_PROJECT_DIR` and does not have this problem.

The real gate is your CI job. This hook only shortens the feedback loop.

## MCP

pmat is registered as an MCP server so an agent can call its analyses directly
instead of shelling out and parsing text. The registration runs:

```bash
pmat --mode mcp
```

`pmat` must be on `PATH` (`cargo install pmat`). Do not change this to
`pmat serve --transport stdio`: `stdio` is not an accepted transport value and
that command exits at argument parsing without emitting a single byte of MCP.
"#;

/// A committed ultracode judgment workflow.
///
/// Structurally identical to `contracts/workflows/release-sweep.ultracode.mjs`
/// — the only committed ultracode convention in this repository — and held to
/// the same invariants its tests assert, re-asserted against *this* generated
/// text in `tests.rs` so the generator cannot drift away from the ground truth
/// it was derived from.
///
/// Note what this is NOT: nothing in pmat executes it, `globalThis
/// .spawnSubagent` is injected by the host orchestrator, and no upstream
/// document defines a file format called an "ultracode schema" — ultracode is
/// a session-only harness effort setting (MACS spec E1), not a config file.
/// See [`super::plan`] for what that means for #1032's first claim.
pub const ULTRACODE_WORKFLOW_MJS: &str = r#"// Committed ultracode judgment workflow. Generated by `pmat init --target ultracode`.
//
// Modelled on contracts/workflows/release-sweep.ultracode.mjs in the pmat
// repository, the one committed example of this convention.
// Sub-spec: docs/specifications/components/modern-agentic-coding-support.md 4-F5
//
// LAYER SPLIT, and it is the whole point of the file. The DETERMINISTIC layer
// is pmat: every mechanical check runs with schema-derived arguments, byte
// framing checked, N-way concurrency, at ZERO model cost, and writes a JSON
// artifact. THIS script is the JUDGMENT layer and only the judgment layer: it
// fans subagents out over the *anomalies* that artifact already contains, to
// skeptically re-verify each one before a human is asked to look. It never
// re-runs the deterministic layer -- doing so would make the cheap half
// stochastic and pay twice for the answer the split exists to get once.
//
// Durable state is .pmat-work receipts (spec E7). A workflow run is
// session-bound and evaporates on exit, so this committed file, not the run,
// is the versioned team-reproducible artifact. Every subagent stamps
// PMAT_AGENT_* so each judgment is attributable in the falsification ledger
// (MACS F1), and a refused turn is recorded with `pmat work event` rather than
// leaving a silent gap (MACS E5).
//
// SCOPE, stated so nobody over-reads it: this is plain ESM with no bundler and
// no runtime dependencies, so CI can validate it with `node --check` alone.
// globalThis.spawnSubagent is injected by the host orchestrator; pmat does not
// execute this file and makes no claim that it runs under any particular
// harness.

import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";

const WORKFLOW_ID = process.env.PMAT_AGENT_WORKFLOW_ID || "pmat-quality-sweep";
const SWEEP_ARTIFACT = "artifacts/qa/mcp-sweep.json";
const BATCH_SIZE = 8;

/** The single read site: the deterministic layer's artifact, and nothing else. */
function loadAnomalies() {
  const report = JSON.parse(readFileSync(SWEEP_ARTIFACT, "utf8"));
  return Array.isArray(report.anomalies) ? report.anomalies : [];
}

function chunk(items, size) {
  const out = [];
  for (let i = 0; i < items.length; i += size) out.push(items.slice(i, i + size));
  return out;
}

/**
 * Spawn one skeptic subagent to re-verify an anomaly. There is exactly ONE
 * spawn site in this file, which is what makes "every spawn stamps
 * provenance" a property of the text rather than of the runs someone watched.
 */
async function spawnSkeptic(anomaly) {
  const env = {
    ...process.env,
    PMAT_AGENT_HARNESS: "ultracode-workflow",
    PMAT_AGENT_WORKFLOW_ID: WORKFLOW_ID,
    PMAT_AGENT_MODEL: process.env.PMAT_AGENT_MODEL || "claude-fable-5",
  };
  const prompt =
    `Skeptically re-verify anomaly ${anomaly.id}: ${anomaly.detail}. ` +
    `Default to "still an anomaly" unless you can falsify it. Report a one-line verdict.`;
  try {
    return await globalThis.spawnSubagent({ prompt, env });
  } catch (err) {
    // A refused or failed turn is a recorded blocking state, never silent.
    execSync(
      `pmat work event --type refusal --note ${JSON.stringify(
        `${WORKFLOW_ID} skeptic refused on ${anomaly.id}: ${err.message}`,
      )}`,
      { stdio: "inherit" },
    );
    return { anomaly: anomaly.id, verdict: "refused", error: String(err) };
  }
}

async function main() {
  const anomalies = loadAnomalies();
  if (anomalies.length === 0) {
    console.log(`${WORKFLOW_ID}: 0 anomalies -- deterministic layer clean, no judgment needed`);
    return;
  }
  console.log(`${WORKFLOW_ID}: judging ${anomalies.length} anomaly(ies)`);
  const verdicts = [];
  for (const batch of chunk(anomalies, BATCH_SIZE)) {
    verdicts.push(...(await Promise.all(batch.map(spawnSkeptic))));
  }
  console.log(JSON.stringify({ workflow_id: WORKFLOW_ID, verdicts }, null, 2));
}

main().catch((err) => {
  console.error(`${WORKFLOW_ID} failed: ${err.message}`);
  process.exit(1);
});
"#;
