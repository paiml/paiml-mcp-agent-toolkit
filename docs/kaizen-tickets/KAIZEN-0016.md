# KAIZEN-0016: Asynchronous isolated delegation (CAID) for pmat work pipelines

**Source paper:** arxiv:2603.21489 — "Effective Strategies for Asynchronous Software Engineering Agents" (Mar 2026)
**Category:** sub-agent-behavior
**Priority:** low
**Effort:** L

## Problem
CAID (Centralized Asynchronous Isolated Delegation) is the paper's SOTA coordination paradigm: a central manager constructs dependency-aware task plans, dispatches subtasks to *concurrent isolated workspaces*, and consolidates via test-based verification. pmat's `pmat work` is sequential and single-branch; concurrent contracts trample each other in a shared checkout.

## Proposed improvement
Add `pmat work parallel --contracts CB-001,CB-002,CB-003` that:
1. Creates a git worktree per contract in `.pmat-work/<id>/worktree/`.
2. Dispatches each to an isolated agent (or manual agent via a slot).
3. On completion, runs cross-contract verifier: merges to staging branch, runs `cargo test` across the union.
4. Only unblocks `work close` if the union passes.

## Impact
- Paper reports substantial throughput lift on multi-task batches without loss of quality.
- Enables pmat to orchestrate *fleets* of sub-agents against a backlog.

## Implementation sketch
1. Worktree-per-contract in `work.rs`.
2. New subcommand `pmat work parallel`.
3. Staging-branch merge + cross-verify step.
4. Compatibility: reject non-independent contracts (file-overlap check) up-front.

## Acceptance criteria
- `pmat work parallel CB-a,CB-b` creates two worktrees and surfaces status.
- File-overlap detection warns when contracts touch same files.
- On success, a staging branch contains a merged result verified by test suite.
- **No MCP tool changes** — pmat_work_* tools remain; parallel is an orchestration layer around them.
