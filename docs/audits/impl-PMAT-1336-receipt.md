# impl receipt — PMAT-1336 (lifecycle: completing rows a ticket cannot complete itself)

## 2026-09-15 — PMAT-1359 and PMAT-1361, closed by #1360 and left `planned`

PR #1360 merged at 3f55dcb5a and closed #1359 and #1361 at 17:17Z, but it left both rows `status: planned`. A ticket cannot complete itself, which is this ticket's subject. CB-2115 then reported `ORPHAN-ROADMAP PMAT-1359` and `ORPHAN-ROADMAP PMAT-1361` on every PR and on master.

Done with `pmat work sync --direction github-to-yaml`. Its dry-run planned exactly two actions: `close-item PMAT-1359 #1359 → Completed` and `close-item PMAT-1361 #1361 → Completed`.

Measured diff:
- `docs/roadmaps/roadmap.yaml`: those two rows' `status` and `updated` lines, 8 changed lines.
- This receipt section.

After it, `pmat work sync --check-only` reads coherent, and `pmat work validate --check-base origin/master` passes. Same shape as master's `1cdffdcca chore(PMAT-1336): PMAT-1339 is completed — its issue closed when #1340 merged`. Split out of #1372 (PMAT-1371), whose quorum judged it separate work.

## 2026-09-15 — PMAT-1371, closed by #1372 and left open

PR #1372 merged at 2604c1e78 and closed #1371, leaving the `PMAT-1371` row not completed. It is the same lifecycle gap as #1360's two rows above: a ticket cannot complete itself. CB-2115 then reported `ORPHAN-ROADMAP PMAT-1371`. Because `ci.yml`'s required `gate` needs `traceability`, that one row blocked every PR to master, including the 3.40.2 release (#1376).

Fixed with `pmat work sync --direction github-to-yaml`. The dry-run planned exactly one action, `close-item PMAT-1371 #1371 → Completed`.

Measured diff:
- `docs/roadmaps/roadmap.yaml`: that row's `status` and `updated` lines, 4 changed lines.
- This receipt section.

After it, `pmat work sync --check-only` reads coherent.
