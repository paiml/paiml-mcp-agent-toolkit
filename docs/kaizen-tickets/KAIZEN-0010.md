# KAIZEN-0010: Commit-intent index with semantic drift detection

**Source paper:** arxiv:2511.19875 — "CodeFuse-CommitEval: Towards Benchmarking LLM's Power on Commit Message and Code Change Inconsistency Detection" + arxiv:2603.15566 — "Lore: Repurposing Git Commit Messages as a Structured Knowledge Protocol for AI Coding Agents"
**Category:** provable-contracts
**Priority:** medium
**Effort:** M

## Problem
pmat has a `CommitEmbedder` (TF-IDF, 128-dim) for `-G` git-history search. But we do not *validate* that a commit message actually describes the diff. CodeFuse-CommitEval shows many commit messages are semantically inconsistent with their diffs — which means `-G` sometimes surfaces misleading results (commit promises X, diff does Y). Lore argues commit messages are a *structured knowledge protocol*; that only works if they're accurate.

## Proposed improvement
Add `pmat git-verify --range master..HEAD` that:
1. For each commit, computes (embedding(message), embedding(diff_summary)).
2. Flags commits where cosine similarity < threshold as *inconsistent*.
3. Optionally runs in pre-push hook (behind `PMAT_VERIFY_COMMITS=1`) to block drift.

Additionally surface the consistency score as a field on `pmat query -G` results so agents can down-weight suspicious history.

## Impact
- Paper shows LLMs struggle to detect inconsistency (baseline ~60% accuracy); a dedicated signal is useful context for agents.
- Improves the `-G` search precision by penalizing mis-labeled history.

## Implementation sketch
1. Extend `git_history_index` with `diff_embedding` column (SQLite schema migration).
2. Add `pmat git-verify` CLI.
3. Expose `commit_consistency_score` on `-G` search results.
4. Document in book: pmat gives agents *calibrated* git signals.

## Acceptance criteria
- `pmat git-verify` outputs per-commit score + overall drift report.
- Schema migration is backward compatible (v2.1 bumps).
- `pmat query -G "fix memory leak"` returns results with `commit_consistency_score` field.
- Pre-push hook opt-in documented.
- **No MCP schema break** — additive field on existing git-history results.
