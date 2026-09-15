# impl receipt — PMAT-1336 (lifecycle: completing rows a ticket cannot complete itself)

## 2026-09-15 — PMAT-1359 and PMAT-1361, closed by #1360 and left `planned`

PR #1360 merged at 3f55dcb5a and closed #1359 and #1361 at 17:17Z, but it left both rows `status: planned`. A ticket cannot complete itself, which is this ticket's subject. CB-2115 then reported `ORPHAN-ROADMAP PMAT-1359` and `ORPHAN-ROADMAP PMAT-1361` on every PR and on master.

Done with `pmat work sync --direction github-to-yaml`. Its dry-run planned exactly two actions: `close-item PMAT-1359 #1359 → Completed` and `close-item PMAT-1361 #1361 → Completed`.

Measured diff:
- `docs/roadmaps/roadmap.yaml`: those two rows' `status` and `updated` lines, 8 changed lines.
- This receipt section.

After it, `pmat work sync --check-only` reads coherent, and `pmat work validate --check-base origin/master` passes. Same shape as master's `1cdffdcca chore(PMAT-1336): PMAT-1339 is completed — its issue closed when #1340 merged`. Split out of #1372 (PMAT-1371), whose quorum judged it separate work.
