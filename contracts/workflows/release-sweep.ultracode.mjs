// MACS-012 (Component 32): committed ultracode judgment workflow.
//
// Sub-spec: docs/specifications/components/modern-agentic-coding-support.md §4-F5
// Contract:  contracts/macs-sweep-v1.yaml
//
// The DETERMINISTIC layer (pmat qa-work mcp-sweep, MACS-011) does the
// mechanical work — every tool called with schema-derived args, byte-framing
// checked, N-way concurrency — at ZERO LLM cost. This script is the JUDGMENT
// layer and ONLY the judgment layer: it fans out subagents over the
// *anomalies* that the deterministic sweep emitted, to skeptically re-verify
// each one before a human is asked to look. It never re-does the sweep.
//
// Durable state lives in .pmat-work/ receipts (spec E7): workflow runs are
// session-bound and evaporate on exit, so this committed script — not the
// ephemeral run — is the versioned, team-reproducible artifact. Every
// subagent stamps PMAT_AGENT_* so each judgment it makes is attributable in
// the falsification ledger (MACS F1); a refusal is recorded via
// `pmat work event`, never a silent gap (MACS E5).
//
// Run:  make release-sweep      (regenerates artifacts/qa/mcp-sweep.json first)
// This file is plain ESM so it can be `node --check`ed in CI without a runtime.

import { readFileSync } from "node:fs";
import { execSync } from "node:child_process";

const WORKFLOW_ID = process.env.PMAT_AGENT_WORKFLOW_ID || "release-sweep";
const SWEEP_ARTIFACT = "artifacts/qa/mcp-sweep.json";
const BATCH_SIZE = 8;

/** Read the deterministic sweep artifact produced by MACS-011. */
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
 * Spawn one skeptic subagent to re-verify an anomaly. Every spawn stamps
 * PMAT_AGENT_* so the judgment is attributable (MACS F1). A refusal is
 * recorded as a work event (MACS E5) rather than swallowed.
 */
async function spawnSkeptic(anomaly) {
  const env = {
    ...process.env,
    PMAT_AGENT_HARNESS: "ultracode-workflow",
    PMAT_AGENT_WORKFLOW_ID: WORKFLOW_ID,
    PMAT_AGENT_MODEL: process.env.PMAT_AGENT_MODEL || "claude-fable-5",
  };
  const prompt =
    `Skeptically re-verify MCP sweep anomaly ${anomaly.id}: ${anomaly.detail}. ` +
    `Default to "still an anomaly" unless you can falsify it. Report a one-line verdict.`;
  try {
    // The orchestrator injects the real spawnSubagent; under `node --check`
    // and in CI this indirection keeps the script side-effect-free.
    return await globalThis.spawnSubagent({ prompt, env });
  } catch (err) {
    // A refused/flagged turn is a recorded, blocking state — never silent.
    execSync(
      `pmat work event --type refusal --note ${JSON.stringify(
        `release-sweep skeptic refused on ${anomaly.id}: ${err.message}`,
      )}`,
      { stdio: "inherit" },
    );
    return { anomaly: anomaly.id, verdict: "refused", error: String(err) };
  }
}

async function main() {
  const anomalies = loadAnomalies();
  if (anomalies.length === 0) {
    console.log("release-sweep: 0 anomalies — deterministic sweep clean, no judgment needed");
    return;
  }
  console.log(`release-sweep: judging ${anomalies.length} anomaly(ies) via ${WORKFLOW_ID}`);
  const verdicts = [];
  for (const batch of chunk(anomalies, BATCH_SIZE)) {
    const results = await Promise.all(batch.map(spawnSkeptic));
    verdicts.push(...results);
  }
  console.log(JSON.stringify({ workflow_id: WORKFLOW_ID, verdicts }, null, 2));
}

// Session-bound continuation is never relied upon (spec E7): durable state is
// the .pmat-work receipts, this script is the deterministic orchestration.
main().catch((err) => {
  console.error(`release-sweep failed: ${err.message}`);
  process.exit(1);
});
