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

## 2026-09-15 — PMAT-1373, closed by #1376 (the 3.40.2 release)

#1376 merged at 467018793 and closed #1373, leaving the `PMAT-1373` row not completed — the same lifecycle gap as the entries above, now the fourth in one day (#1360 → PMAT-1359/1361, #1372 → PMAT-1371, #1376 → PMAT-1373). Because `ci.yml`'s required `gate` needs `traceability`, one uncompleted row makes every PR to master red.

Fixed with `pmat work sync --direction github-to-yaml`; the dry-run planned exactly one action, `close-item PMAT-1373 #1373 → Completed`. Diff: that row's `status` and `updated` lines (4 changed lines), plus this receipt section.

Also closed in the same sweep, without a roadmap row: **#1378**, filed by github-actions at 22:32Z ("3.40.2 is declared in Cargo.toml but not fully released") between #1376's merge and the tag. It was closed with the release evidence — `make release-check` on master now exits 0: 3.40.2 tagged, released and on crates.io.

After both, `pmat work sync --check-only` reads coherent.

## 2026-09-17 — PMAT-1381, an open issue with no row

Issue #1381 was opened at 2026-09-16T14:42:59Z and is still OPEN, but master (441d198e7) has no row for it. This is the other half of the bijection: the entries above complete rows whose issues closed, and this one registers a row for an issue that opened. CB-2115 reported `ORPHAN-GITHUB #1381`, and `pmat work sync --check-only` exited 1 with that as its only finding (114 open items, 115 open issues). Because `ci.yml`'s required `gate` needs `traceability`, every PR to master was red. The row landed inside PMAT-1365's PR #1368, and a review lane failed it there as outside that ticket's scope. So it lands here on its own.

Two writers were tried on scratch copies of the roadmap first:

- `pmat work sync --direction github-to-yaml` planned one action, `create-item #1381 → GH-1381`. It writes id `GH-1381`, sets `created`/`updated` to the wall clock with nanoseconds, and leaves `labels: []`.
- `pmat work add --github-issue 1381 -t kind:code "<issue title>"` mints `PMAT-1381` and binds `github_issue: 1381`. Its help calls this the collision-proof path and says to prefer it whenever an issue exists. The #1360 rows (PMAT-1356 … PMAT-1373) have the same `PMAT-<issue>` shape.

The second writer was used. Measured diff: `docs/roadmaps/roadmap.yaml` gains 17 lines, plus this receipt section.

It is not byte-identical to the row on the operator's unmerged branch `PMAT-1381-row` (242755717), which is the same row PMAT-1365's branch carries. Of the 17 lines, 15 match. The other two are `created` and `updated`: the writer stamps the time it ran, `2026-09-17T07:48:16Z`, while that row uses the issue's creation time, `2026-09-16T14:42:59Z`. Neither writer can produce the issue's time, and making the lines match by hand would be a hand edit. So when either branch merges after this one, those two lines will conflict, and the fix is to keep master's row. The deviation is put to the review quorum; its verdict is `docs/audits/quorum-PMAT-1336.json`.

After the write, `pmat work sync --check-only` reads coherent (115 matched, 0 findings), CB-2115 passes, and `pmat work validate --check-base origin/master` passes, with one warning that PMAT-1381 has no acceptance criteria. The row is only registered: #1381 stays open (keeps-open #1381).
