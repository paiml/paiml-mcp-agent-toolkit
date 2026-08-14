# Design: Google Antigravity & Claude Code Ultracode Support for Agentic Coding

## 1. Five-Whys: Root Cause Analysis
**Problem**: Our platform currently lacks full, native orchestration support for Google Antigravity and Claude Code's Ultracode, resulting in manual scaffolding and unverified agent workflows.
- *Why?* PMAT currently tracks agent provenance but doesn't fully integrate with Antigravity conversation IDs (MACS-001) or Ultracode's `xhigh` execution harness (MACS-012).
- *Why?* The integration requires a dedicated orchestration bridge that handles subagent tracking and context isolation.
- *Why?* Ultracode and Antigravity each use distinct API models and environmental variables (`PMAT_AGENT_HARNESS`, `WORKFLOW_ID`).
- *Why?* We haven't implemented the `release-sweep.ultracode.mjs` committed judgment workflow and the Antigravity transcript parser.
- *Why?* There hasn't been a unifying Provable Contract (PV) specifying the falsification claims for cross-agent orchestration.

**Conclusion**: We must implement a PV-backed Hybrid Orchestration Bridge (MACS-019) that normalizes both Claude Code Ultracode and Google Antigravity workflows.

## 2. Proposed Solution & Provable Contracts (PV)
To solve this, we will implement:
1. **Google Antigravity Support**: Extend `AgentProvenance` to inject Subagent Task IDs and Conversation IDs. Add a transcript parser that bounds agent work for inclusion in `FalsificationReceipts`.
2. **Claude Code Ultracode Support**: Implement `contracts/workflows/release-sweep.ultracode.mjs` as a committed script for judgment-layer orchestration, verifying that `PMAT_AGENT_HARNESS` and `WORKFLOW_ID` are correctly set for subagents.

### Provable Contract Definition (pv)
```rust
// pv-contract: MACS-012-019-Orchestration
require(
    env::var("PMAT_AGENT_HARNESS").is_ok(),
    "Agent harness must be explicitly defined (ultracode/agy)"
);
require(
    provenance.has_conversation_id(),
    "Google Antigravity or Ultracode conversation ID must be captured in provenance"
);
```

## 3. Popperian Falsification Strategy
Per our falsification-based quality enforcement:
- **Claim 1 (Ultracode)**: Every subagent spawn in `release-sweep.ultracode.mjs` sets `PMAT_AGENT_HARNESS`.
  - *Falsification*: `node --check contracts/workflows/release-sweep.ultracode.mjs && grep -c 'PMAT_AGENT_' contracts/workflows/release-sweep.ultracode.mjs # >= 2`.
- **Claim 2 (Antigravity)**: The transcript parser successfully extracts Antigravity bounds and injects them into the receipts.
  - *Falsification*: Run a mock Antigravity transcript JSONL against `pmat falsify` and assert the receipt matches the mock conversation ID.
- **Claim 3 (No Raw Resume)**: Session-bound resume is never relied on for durable state.
  - *Falsification*: `grep -c 'resume' contracts/workflows/release-sweep.ultracode.mjs # == 0`.

## 4. Implementation Steps
1. Add `FalsificationMethod::ProvableContract` variant for agent contexts.
2. Build `contracts/workflows/release-sweep.ultracode.mjs`.
3. Implement `GoogleAntiGravity` parser in `AgentProvenance`.
4. Validate with `pmat verify` and `pmat work falsify`.
