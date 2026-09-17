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

## 2026-09-17 — PMAT-1381, an open issue with no row (landed first by #1382)

Issue #1381 was opened at 2026-09-16T14:42:59Z, but master at 441d198e7 had no row for it. CB-2115 reported `ORPHAN-GITHUB #1381`, and it was the only finding of `pmat work sync --check-only` (114 open items against 115 open issues). Registering it is in PMAT-1336's scope because the ticket's first acceptance criterion asks for the whole bijection: "every roadmap item whose issue is closed is terminal and every non-terminal item's issue is open — pmat work sync --check-only reports a bijection with zero findings against live GitHub".

On this branch, d22870740 wrote the row with `pmat work add --github-issue 1381 -t kind:code "<issue title>"`, which mints `PMAT-1381`. That was chosen over `pmat work sync --direction github-to-yaml`, whose dry-run would have minted `GH-1381` with `labels: []`. The row did not match the one on the operator's branch `PMAT-1381-row` (242755717) byte for byte: `created` and `updated` held the writer's run time, 2026-09-17T07:48:16Z, instead of the issue's createdAt. Review round 1 on d22870740 did not agree; lane 1 failed it on scope and on a forward reference to the verdict file. 035941407 answered both findings, and round 2 on it passed 3/3.

Before this PR could merge, #1382 (PMAT-1366) merged at e89a827f7 with a PMAT-1381 row of its own: the issue's createdAt, one acceptance_criteria line, and `labels: []`. This branch's row was then redundant and conflicted with master. The merge commit cf76edd3c resolved the conflict by taking master's `docs/roadmaps/roadmap.yaml` unchanged, so this PR adds no PMAT-1381 bytes. The row master now carries is #1382's, and it matches neither the operator's branch nor d22870740 (different `acceptance_criteria` and `labels`). #1381 stays open (keeps-open #1381).

`pmat verify --format json` on d22870740 returned `ok: false`. Format, satd and clippy passed; complexity was not measured. The tests stage failed on one lib test out of 21,661: `services::tdg_baseline::tests::the_committed_baseline_is_the_measured_count` counts 1742 definitions below grade A against a baseline of 1688. That test counts over the local `.pmat/context.db` index. It is open issue #1266, which records it as unmeasurable in CI, and this branch did not cause it: the branch changes no `src/` file.

## 2026-09-17 — PMAT-1366, closed by #1382 (completed first by #1364)

#1382 merged at e89a827f7 and closed issue #1366 at 2026-09-17T08:55:11Z, leaving the `PMAT-1366` row `planned`. On this branch, 0533e9b1d completed it with `pmat work sync --direction github-to-yaml`; the dry-run planned exactly one `close-item PMAT-1366 #1366 → Completed`. Review round 3 on dcad45735 passed 3/3.

Before this PR could merge, #1364 (PMAT-1363) merged at 8915fe3e6 and set the PMAT-1366 row to `completed` itself. The merge commit 258c558ff resolved the conflict by taking master's `docs/roadmaps/roadmap.yaml` unchanged, so this PR adds no PMAT-1366 bytes.

## 2026-09-17 — PMAT-1385, an open issue with no row

Issue #1385 was opened at 2026-09-17T09:57:46Z with the label `kind:code`. Master 8915fe3e6 had no row for it, so `pmat work sync --check-only` reported `ORPHAN-GITHUB #1385`. Fixed in 475abd957 with `pmat work add --github-issue 1385 -t kind:code "<issue title>"`, the writer used for PMAT-1381 above, with the label copied from the issue. The diff is a 17-line row, `PMAT-1385` bound to `github_issue: 1385`. The issue stays open (keeps-open #1385).

## 2026-09-17 — PMAT-1363, closed by #1364 and left `planned`

#1364 merged at 8915fe3e6 and closed issue #1363 at 2026-09-17T09:51:56Z. It completed PMAT-1366 but left the `PMAT-1363` row `planned`, so `pmat work sync --check-only` reported `ORPHAN-ROADMAP PMAT-1363`. Fixed in b5b664ae5 with `pmat work sync --direction github-to-yaml`: once PMAT-1385 was registered, its plan was exactly one `close-item PMAT-1363 #1363 → Completed`. The diff is that row's `status` and `updated` lines.

After both, `pmat work sync --check-only` reads coherent.

## 2026-09-17 — PMAT-1386, an open issue with no row

Issue #1386 was opened at 2026-09-17T10:50:15Z with no labels. Master 7c2aa59b8 had no row for it. At `HEAD=7c2aa59b8 origin/master=7c2aa59b8 behind=0`, `pmat comply check --checks CB-2113,CB-2115` failed CB-2115 with exactly one finding, `ORPHAN-GITHUB #1386`, and `pmat work sync --check-only` read open items 114 against open issues 115. CB-2113 read `not_applicable` (HEAD is the default branch). Because the required `gate` needs `traceability`, that one finding made every open PR red. PR #1368 (PMAT-1365) does not carry the row: `gh pr diff 1368` adds no `PMAT-1386` or `github_issue: 1386` line, and neither do #1357, #1341 or #1224, the other open PRs that touch `docs/roadmaps/roadmap.yaml`.

Fixed in 373ace156 with `pmat work add --github-issue 1386 "<issue title>"`, the writer used for PMAT-1381 and PMAT-1385 above. `-t` was not passed, because the issue carries no label, so the row has `labels: []`. The diff is a 16-line row, `PMAT-1386` bound to `github_issue: 1386`. As with the PMAT-1381 row, `created` and `updated` hold the writer's run time, not the issue's createdAt. The issue stays open (keeps-open #1386).

After it, `pmat work sync --check-only` reads 115/115 coherent, and `pmat work validate --check-base origin/master` passes. It warns that PMAT-1385 and PMAT-1386 have no acceptance criteria, which does not fail validation.

Two things the skill's own gates reported, recorded here rather than worked around:
- `kind-gate.sh PMAT-1336` exits 2, `unknown kind 'lifecycle'`, and `model-gate.sh` exits 2 for the same reason. The rail knows only `code | triage | docs | measurement`. The branch keeps to the triage path allow-list (`docs/roadmaps/roadmap.yaml`, `docs/audits/**`), and no `src/` file changes.
- The quorum artifact lives at `docs/audits/quorum-PMAT-1336.json`, as it did for #1383, not under `.quorum/`, which does not exist in this repository.

## 2026-09-17 — PMAT-1305 and PMAT-708, closed around #1388 and left `planned`

#1388 (PMAT-1305) merged at ce945d81e at 2026-09-17T13:27:53Z. Its `Closes #1305` closed issue #1305 at 13:27:54Z. #1388's body names #1284 as a duplicate report of the same flaky test but does not close it: "whether this PR also closes it is left to the quorum". noahgift closed #1284 by hand at 13:29:02Z (stateReason COMPLETED), with the comment "duplicate of #1305 … fixed by #1388 (ce945d81e)". Neither row was touched, so PMAT-1305 (github_issue 1305) and PMAT-708 (github_issue 1284) both stayed `planned`.

At `HEAD=ce945d81e origin/master=ce945d81e behind=0`, `pmat comply check --checks CB-2113,CB-2115` failed CB-2115 with two findings: `ORPHAN-ROADMAP PMAT-1305: #1305 is closed` and `ORPHAN-ROADMAP PMAT-708: #1284 is closed`. `pmat work sync` read open items 115 against open issues 113. CB-2113 read `not_applicable` (HEAD is the default branch). The required `gate` needs `traceability`, so these two findings made every open PR red, including #1368, which is armed for auto-merge.

Fixed with `pmat work sync --direction github-to-yaml`. Its dry-run planned exactly `close-item PMAT-1305 #1305 → Completed` and `close-item PMAT-708 #1284 → Completed`. The writer applied both in one run. Its diff was split by hunk into one commit per finding, and the two commits together equal the writer's output line for line:
- da02fb505 completes PMAT-1305: the row's `status` and `updated` lines.
- ebbd9033e completes PMAT-708: the row's `status` and `updated` lines. The row keeps its `deferred:3.41.0` label.

After both, `pmat work sync --check-only` reads 113/113 coherent. `pmat comply check --checks CB-2113,CB-2115` passes both: CB-2113 finds the Pmat-Ticket trailer on both branch commits, and CB-2115 finds the bijection.

Corrections to the brief:
- The brief said #1388 might also have closed #1284. It did not. #1284 was closed by hand a minute later, not by a closing keyword. The row is PMAT-708, not a PMAT-1284 row.
- `.quorum/` does not exist in this repository. As for #1383 and #1387, the quorum artifact is `docs/audits/quorum-PMAT-1336.json`.

Gates of the skill itself: `kind-gate.sh PMAT-1336` and `model-gate.sh PMAT-1336` both still exit 2 on `kind:lifecycle`, as recorded for PMAT-1386. The branch keeps to `docs/roadmaps/roadmap.yaml` and `docs/audits/**`.

## 2026-09-17 — PMAT-1365, closed by #1368 and left `planned`

#1368 (PMAT-1365, `make gate`) merged as ef2a0b947 at 2026-09-17T14:57:09Z and its closing line closed issue #1365. A ticket cannot complete itself under CB-2113, so the `PMAT-1365` row stayed `planned`.

At `HEAD=ef2a0b947 origin/master=ef2a0b947 behind=0`, `pmat work sync --check-only` read open items 113 against open issues 112 with one finding, `ORPHAN-ROADMAP PMAT-1365: #1365 is closed`. The required `gate` needs `traceability`, so that finding reds every open PR (#1389 is in CI; two more tickets are being implemented).

Fixed with `pmat work sync --direction github-to-yaml`. Its dry-run planned exactly one action, `close-item PMAT-1365 #1365 → Completed`; the diff is that row's `status` and `updated` lines. After it, `pmat work sync --check-only` reads 112/112 coherent.

This section was written by the release-3.41.0 orchestrator session itself, not a ticket session: all three session slots were occupied and the change is the writer's own two-line output.

## 2026-09-17 — PMAT-1393 and PMAT-900001: two open issues with no row, each blocking the other's PR

At `HEAD=f25d7f1cc origin/master=f25d7f1cc behind=0`, `pmat work sync --check-only` read open items 112 against open issues 114 with two findings: `ORPHAN-GITHUB #1393` (opened 2026-09-17T15:27:32Z by a ticket session, no labels) and `ORPHAN-GITHUB #1395` (opened 16:48Z by the PMAT-900001 session, which binds its issue last; its row rides in PR #1391).

The two findings deadlock: #1391 is armed on a 3/3 verdict but its `traceability` is red on #1393, and a PR registering only #1393 would be red on #1395. So this PR registers both.

- PMAT-900001 (issue 1395): the row is PR #1391's own hunk — `git diff origin/master...origin/PMAT-900001-issue-closure-contract -- docs/roadmaps/roadmap.yaml`, last hunk only, applied with `git apply`. It was written on that branch by `pmat work add --id` and `pmat work sync --direction yaml-to-github`; it is transported here, not retyped, so #1391 ends up with ONE row for the issue, identical on both sides. Measured, and a correction to this section's first draft: a trial merge of this branch into #1391's head CONFLICTS in roadmap.yaml (both sides append at end of file: #1391 adds the PMAT-900001 row, this branch adds that row plus PMAT-1393; the reverse row order conflicts too). The resolution is mechanical — take master's side — and is #1391's to make when it takes master. That branch's other roadmap hunk, the `notes:` cross-reference on PMAT-1369, is NOT taken: it is PMAT-900001's work, not a registration.
- PMAT-1393 (issue 1393): `pmat work add --github-issue 1393 "<issue title>"`, a 16-line row, `labels: []`. keeps-open #1393.

After both, `pmat work sync --check-only` reads 114/114 coherent. Written by the release-3.41.0 orchestrator session; all three session slots were occupied.
