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

## 2026-09-17 — PMAT-1385, closed by #1389 and left `planned`

#1389 (PMAT-1385, `pmat work migrate` under the repository lock) merged as b58addab8 at 2026-09-17T18:20:40Z and closed issue 1385. A ticket cannot complete itself under CB-2113, so its row stayed `planned`.

At `HEAD=b58addab8 origin/master=b58addab8 behind=0`, `pmat work sync --check-only` read open items 114 against open issues 113 with one finding, `ORPHAN-ROADMAP PMAT-1385: #1385 is closed`. It reds `traceability` on every open PR; #1391 is armed and waits on it.

Fixed with `pmat work sync --direction github-to-yaml`: the dry-run planned exactly `close-item PMAT-1385 #1385 → Completed`; the diff is that row's `status` and `updated` lines. After it: 113/113 coherent. Written by the release-3.41.0 orchestrator session.

## 2026-09-17 — PMAT-636 (closed by #1394) and PMAT-900001 (landed by #1391, issue closed here)

At `HEAD=b3df4a402 origin/master=b3df4a402 behind=0`, `pmat work sync --check-only` read open items 113 against open issues 112 with one finding, `ORPHAN-ROADMAP PMAT-636: #1266 is closed` — #1394 (CB-200 back under its baseline) merged as b3df4a402 and closed issue 1266.

PMAT-900001 is the second row. #1391 merged as 739d70269 without a closing line on purpose ("the row and the issue are completed together by a later lifecycle PR"), so issue 1395 stayed open and its row `planned`: coherent, but finished work. The issue was closed by the orchestrator through `mutate.sh close --issue 1395 --cite docs/audits/impl-PMAT-900001-receipt.md --quorum docs/audits/quorum-PMAT-900001.json` (agreed=true, 3/3 PASS; the close gate's two requirements), read back CLOSED.

Then `pmat work sync --direction github-to-yaml`: the dry-run planned exactly `close-item PMAT-636 #1266 → Completed` and `close-item PMAT-900001 #1395 → Completed`; the diff is those two rows' `status` and `updated` lines. After it: 111/111 coherent. Written by the release-3.41.0 orchestrator session.

## 2026-09-18 — after the 3.41.0 cut: PMAT-1399 completed; #1401 and #1403 registered

At `HEAD=ecd97c6bc origin/master=ecd97c6bc behind=0`, `pmat work sync --check-only` read open items 112 against open issues 113 with three findings: `ORPHAN-ROADMAP PMAT-1399` (#1400, the release cut, merged as ecd97c6bc and closed issue 1399), `ORPHAN-GITHUB #1401` (auto-filed by github-actions at 23:20:36Z: "release-check: 3.41.0 is declared in Cargo.toml but not fully released" — true, and it stays true until the clean room is green and the crate is published), `ORPHAN-GITHUB #1403` (00:24:33Z: the clean-room GATE B2 defect; the PMAT-1403 fix session's branch carries its own copy of this row and takes master's bytes when it merges).

Fixes, all through the sanctioned writers: `pmat work add --github-issue 1401` and `--github-issue 1403` (two 16-line rows, `labels: []`); `pmat work sync --direction github-to-yaml`, whose dry-run planned exactly `close-item PMAT-1399 #1399 → Completed`. After them: 113/113 coherent. Written by the release-3.41.0 orchestrator session. keeps-open #1401, keeps-open #1403.

## 2026-09-18 — PMAT-1403 completed after #1406 merged

At `HEAD=117ce5171 origin/master=117ce5171 behind=0`, `pmat work sync --check-only` read open items 113 against open issues 112 with one finding, `ORPHAN-ROADMAP PMAT-1403: #1403 is closed` — #1406 (`LockfileGuard`: dead-code analysis no longer rewrites the analysed project's `Cargo.lock` under an ambient `[patch]`) merged as 117ce5171 at 2026-09-18T07:55Z and closed issue 1403. A ticket cannot complete itself under CB-2113, so its row stayed `planned` in that PR.

Fixed with `pmat work sync --direction github-to-yaml`: the dry-run planned exactly `close-item PMAT-1403 #1403 → Completed`; the diff is that row's `status` and `updated` lines, and nothing else (`git diff --stat`: 1 file, 2 insertions, 2 deletions). After it: 112/112 coherent.

`docs/roadmaps/entries/` does not exist in this repo, so `RoadmapService::save()` takes its whole-file branch and writes `docs/roadmaps/roadmap.yaml` directly — PMAT-1363's fragment path is conditional on that directory existing and is not exercised here.

Observation, not a defect of this change: the writer stamps `updated` as `2026-09-18T07:58:27.422072847+00:00` while 204 of the file's rows carry the `…Z` form and 185 carry `+00:00`. The split predates this PR (counted on `117ce5171` before the edit) and is left alone rather than mass-rewritten inside a lifecycle PR.

#1401 ("3.41.0 is declared in Cargo.toml but not fully released") is still open and still true: the tag's clean room was red, nothing was published, and crates.io max_stable_version is 3.40.2. keeps-open #1401. Written by the release-3.41.0 orchestrator session.

## 2026-09-18 — #1410 registered: 3.41.1 declared, not yet published (lifecycle-12)

At `HEAD=94286c23d origin/master=94286c23d behind=0`, `pmat work sync --check-only` read open
items 113 against open issues 114 with one finding, `ORPHAN-GITHUB #1410` — auto-filed by the
release-check automation at 2026-09-18T11:21:32Z, "release-check: 3.41.1 is declared in
Cargo.toml but not fully released". #1409 (the 3.41.1 release commit, PMAT-1408) had merged as
`94286c23d` minutes earlier and master began declaring a version crates.io does not carry;
crates.io `max_stable_version` is `3.40.2`. The claim is exactly true and it closes itself
when `make release-check` reads green after the publish. #1401 is the same issue for 3.41.0
and is still open and still rowed.

**Why this round is in front of the tag rather than after it.** Until #1410 is rowed it is an
`ORPHAN-GITHUB`, so CB-2115 fails, so the `cb-2113-cb-2115` leg of `make gate` fails, so
`make gate` is **RED** on `94286c23d` — the very commit the release brief requires green
before `v3.41.1` goes on it. Measured on that commit: 32 legs ran, **31 PASS and 1 FAIL**, the
one FAIL being `cb-2113-cb-2115` with `✗ CB-2115: 1 finding(s) — ORPHAN-GITHUB #1410`, while
`✓ CB-2113` read `not_applicable: HEAD is the default branch; 22 of 22 non-merge commit(s)
since v3.41.0 carry a Pmat-Ticket trailer`.

Fixed with the sanctioned writer, `pmat work sync --direction github-to-yaml`: the dry-run
planned exactly one action, `create-item #1410 "release-check: 3.41.1 is declared in
Cargo.toml but not fully released" → GH-1410`, and the diff is that row and nothing else
(`git diff --stat`: 1 file, 16 insertions, 0 deletions). Row status `planned`, which is
correct — the thing it describes has not happened yet. After it: **114/114 coherent**, and
`pmat comply check --checks CB-2113,CB-2115` reads `✓` on both. keeps-open #1410, keeps-open
#1401.

**Recorded, not fixed here.** The release-check filer and CB-2115 are in tension by
construction: every release reds `make gate` from the version-bump merge until the publish,
and the only way through is a lifecycle row like this one. Nothing in this PR changes that,
and nothing should — it wants its own ticket after the release rather than a change made
inside one.

**A second process finding, from this round's own quorum.** Round 1 on #1411 returned lane 1
`FAIL`, lane 2 `NO-VERDICT`, lane 3 `PASS`. Lane 1's FAIL was correct on its own terms and is
the reason this section exists: `quorum-review.sh:193` feeds the lanes
`docs/audits/impl-PMAT-1336-receipt.md`, and because PMAT-1336 is a **standing** ticket that
never completes, that file still described the *previous* round (PMAT-1403 completed, #1401
and #1403 registered) while the diff under review added `GH-1410`. Lane 1 read the mismatch
exactly as the refutation doctrine tells it to — "a receipt claim is not backed by the diff" —
and failed the PR. The fix is this section, not a narrower lane brief: a standing ticket's
receipt goes stale the moment its round lands, and the round after it must make the receipt
current *before* it asks for a verdict. Written by the release-3.41.1 orchestrator session.

## lifecycle-14 — PMAT-1370 completed (#1443 merged)

*(An earlier round.)*

PR #1443 merged as `c234a75cb` and GitHub closed #1370. CB-2113 forbids a ticket completing
itself, so #1443 left the PMAT-1370 row at `inprogress`. That left one CB-2115 ORPHAN-ROADMAP
finding on master.

Measured at HEAD=`c234a75cb`, origin/master=`c234a75cb`, behind=0: `pmat work sync --direction github-to-yaml --dry-run` reported
129 open items against 128 open issues, one finding (`ORPHAN-ROADMAP PMAT-1370: #1370 is closed`), and a plan of exactly
one action, `close-item PMAT-1370 #1370 → Completed`. Applying it changed 1 file with 2
insertions and 2 deletions: the row's `status` (`inprogress` → `completed`) and `updated`, and nothing else. After it,
`pmat work sync --check-only` reads **128/128 coherent**.

## lifecycle-15 — PMAT-1428 and GH-1417 completed (#1438 merged)

*(An earlier round.)*

PR #1438 merged as `26cef2be3`. It folded #1432 (PMAT-1428, the glibc 2.35 floor for linux-gnu assets) and the dependabot patch-updates group #1417 (GH-1417, tracked by issue #1445). The PR body said "Refs" rather than using a closing keyword, and CB-2113 forbids a ticket completing itself, so both rows stayed open. After the merge, #1428 and #1445 were closed with a comment naming #1438.

Measured at HEAD=`26cef2be3`, origin/master=`26cef2be3`, behind=0:
- `pmat work sync --direction github-to-yaml --dry-run` reported 129 open items against 127 open issues.
- It found two findings: `ORPHAN-ROADMAP GH-1417: #1445 is closed` and `ORPHAN-ROADMAP PMAT-1428: #1428 is closed`.
- Its plan was exactly those two `close-item` actions.

Applying the plan changed 1 file, with 4 insertions and 4 deletions: each row's `status` (`inprogress`/`planned` → `completed`) and `updated`, and nothing else. After it, `pmat work sync --check-only` reads **coherent**.

## lifecycle-16 — GH-1448 completed (#1449 merged, nightly republished)

**Current round.** This section describes the diff under review. Everything above it describes earlier rounds.

#1449 merged as `350f1adf4`. It pinned cargo-zigbuild to 0.23.4 so that aarch64-gnu links, and it filed the GH-1448 row.

Nightly run 36231434580 on `350f1adf4` then published fresh assets:
- both gnu assets pass `sha256sum -c`;
- the highest GLIBC symbol in each is GLIBC_2.34;
- both run `pmat --version` on a glibc 2.35 host (aarch64 under qemu).

Master's `traceability` check was green before #1448 was closed as fixed by #1438 and #1449.

Measured at HEAD = origin/master = `350f1adf4`, behind=0:
- `pmat work sync --direction github-to-yaml --dry-run` reported 128 open items against 127 open issues.
- It found one finding: `ORPHAN-ROADMAP GH-1448: #1448 is closed`.
- Its plan was one `close-item` action.

Applying the plan changed 1 file, with 2 insertions and 2 deletions: the row's `status` (`planned` → `completed`) and `updated`. After it, `pmat work sync --check-only` reads **coherent**.

**Addendum: GH-1451 filed.** CI on the first head (`a7fcdc7a7`) went red on CB-2115 with `ORPHAN-GITHUB #1451`, an external issue opened at 09:39Z, after the sync above. Re-measured at origin/master `350f1adf4`, behind=0: 127 open items against 128 open issues, one finding, and a one-action plan (`create-item #1451 → GH-1451`). Applying it adds 16 lines, which is a new `planned` row with no other change. `--check-only` reads **coherent**. The row only files the issue. Triaging it (keep or close as not planned) is left to the operator.
