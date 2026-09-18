# release receipt — pmat 3.41.1

**Verdict: `DONE`.** pmat **3.41.1** is published. crates.io `max_stable_version` reads
`3.41.1` back from the registry API, the GitHub release object exists and is promoted, and
the CI clean room — the gate that stopped the 3.41.0 attempt — read **green on the tag's own
sha** before anything was uploaded.

**3.41.0 was not re-cut. It is burned, unpublished, and its tag is left where it points**, on
a unanimous 3/3 quorum verdict against moving it. That decision, and its basis, is the first
section below.

**Orchestrator:** Claude Opus 5 (`model-gate.sh`: `model=opus-5 class=opus decision=admit
basis=transcript`). **Ticket:** PMAT-1408 / issue #1408, `kind:code`, filed issue-first by
this session after the quorum decided a new version was needed. **Clone:**
`~/src/paiml-mcp-agent-toolkit.wt/REL-3.41.0`, standalone. Session 1's receipt for the
stopped 3.41.0 attempt is `docs/audits/release-3.41.0-receipt.md`, in this same PR.

---

## 1. The decision: burn 3.41.0, do not move its tag

The brief offered two options and recommended (a): delete the remote tag `v3.41.0` and
re-create it on the fixed master head. A width-3 `grillme` quorum was given both options,
every measurement below, and an explicit instruction to attack the recommendation rather than
defer to it.

### The three facts, re-measured by this session before anything was decided

| fact | command | result |
|---|---|---|
| no GitHub release object | `gh release view v3.41.0` | **exit 1**, stderr `release not found`; `gh release list` newest were `v3.40.2`, `v3.40.1` (Latest) |
| never on crates.io | registry API `curl -A '<ua>' https://crates.io/api/v1/crates/pmat` | `max_version 3.40.2 max_stable 3.40.2 newest 3.40.2`, 304 versions, newest five `3.40.2 3.40.1 3.40.0 3.39.0 3.38.0` |
| never on crates.io, directly | `curl .../api/v1/crates/pmat/3.41.0` | **HTTP 404** |
| the tag object | `gh api .../git/ref/tags/v3.41.0` then `.../git/tags/42b4b7192` | annotated tag `42b4b719237a1890732da60ae8677373cac8ad01` -> target `ecd97c6bc318f09552364dfc052c013cd6fbaa6d`, tagger `Noah Gift` `2026-09-17T23:20:55Z` |
| master only moved forward | `git merge-base --is-ancestor ecd97c6bc 18e5ddb87` | true |

### Verdict: 3/3 FAIL, unanimous, against option (a)

| lane | model (measured) | verdict |
|---|---|---|
| 1 | `gemini-3.1-pro-high` | **FAIL** |
| 2 | `gemini-3.8-flash-high` | **FAIL** |
| 3 | `gemini-3.7-flash-high` | **FAIL** |

Author `claude-opus-5` / family `claude`; no lane in the author's family. All three
`grounding_check: parity`, no `verdict_coerced`, `partial=false`, `partial_reasons=[]`. All
three lanes ran in sandboxed self-contained clones, all tree-witness verified, all removed
byte-identical; **no `KEPT`, no exit 3, no exit 4 (`LANE BLIND`)**. No `--concurrent-scope`
was declared, so every lane asserted the whole checkout. agy conversations
`9364f056…`, `a6d8ad4e…`, `1ff9a725…`; `child_conversations=3`.

**The basis, which every lane reached independently:** the brief's premise — "never published,
therefore no consumer can be holding it" — is false for git-native consumers. A pushed tag
propagates immediately, and `git fetch` **refuses to clobber an existing tag** without
`--force`. Any clone, CI cache, Docker build, vendoring tool or `cargo install --git --tag`
from the ~12-hour window would go on resolving `v3.41.0` to the red commit `ecd97c6bc`,
silently, for as long as that clone lives. An empty crates.io and a missing release object say
nothing about them. Lanes 2 and 3 added that amending the `[3.41.0]` CHANGELOG section to name
#1406 would **conceal** the gate failure rather than record it, and that committed receipts
already bind `v3.41.0` to `ecd97c6bc`.

Per the brief — "if any lane FAILs (a) on that ground, take (b) — do not argue it" — option
(b) was taken without argument.

### The third option, recorded and NOT taken

All three lanes independently proposed the same option, in neither brief: delete the
`v3.41.0` tag without re-creating it, **and** cut 3.41.1. It was not taken, for three reasons
stated here rather than buried:

1. Deleting a pushed tag is the same class of outward, hard-to-undo action the lanes had just
   refused. It removes the ref for anyone who has not fetched it and changes nothing for
   anyone who has — the asymmetry they objected to, in the other direction.
2. The operator is away and authorised no deletion. The brief authorised option (b) as
   written, and option (b) as written does not delete anything.
3. Lanes 2 and 3 argue in the same breath that the tag should be **preserved** as the audit
   record binding `v3.41.0` to `ecd97c6bc` ("Preserve the audit record of v3.41.0 as a failed
   pre-publish cut"). The two halves of their own recommendation are in tension; preserving is
   the half that is not destructive.

It is the operator's call. `v3.41.0` still points at `ecd97c6bc` and the `[3.41.0]` CHANGELOG
section is byte-for-byte as it was written, with a note above it recording what became of that
cut.

---

## 2. What shipped

`[3.41.1] - 2026-09-18` names **#1406** (`LockfileGuard`, PMAT-1403): a read-only
`pmat analyze dead-code` can no longer rewrite the analysed project's `Cargo.lock`. The
`### Note on 3.41.0` section beneath it records the cut that failed, with the run id, so the
CHANGELOG does not quietly claim 3.41.0 shipped.

**Version carriers are two, and only two** — `Cargo.toml:4` and `Cargo.lock:4989` (session 1's
correction, re-confirmed here: `grep -rn '3\.41\.0' Cargo.toml Cargo.lock` after the bump
returns nothing).

---

## 3. The rail, in order, with what each step measured

Every measurement opened with a tree line. **None ran with `behind≠0`.**

| # | step | result |
|---|---|---|
| P0 | discovery | `kind=code` · `model=opus-5 class=opus decision=admit basis=transcript` · `slots=3 gh_calls_per_min=30 bank=3` · `discover.json` sha256 `300f875ec89064baf0629fc683193d1325d62bd6859a58f4b71f87c5741188a5` · `gate_cmd=make gate`, **`gate_cmd_fallback=false`** · `target-guard` PASS |
| P1 | the decision quorum | 3/3 FAIL on option (a) — section 1 |
| P1b | ticket, issue-first | issue **#1408** filed, then `pmat work add --github-issue 1408` -> **PMAT-1408** (the collision-proof path, #1240). Bijection 112/112 -> 113/113 coherent |
| P2 | release commit `b00c5ae77` | `Cargo.toml` + `Cargo.lock` 3.41.0->3.41.1, `CHANGELOG.md [3.41.1]`, PMAT-1408's row. **4 files, nothing else.** `cargo metadata --locked` exit 0 |
| P2b | PR **#1409** | quorum round 1: 2 PASS + 1 **NO-VERDICT** (lane failure) -> re-run. Round 2: **3/3 PASS, `agreed=true`, `partial=false`, `partial_reasons=[]`, `dissent=[]`**, all three `.err` 0 bytes. CI 47 SUCCESS / 5 SKIPPED / 0 FAILURE, `mergeStateStatus: CLEAN`. Armed only via `pmat-merge`. Merged **`94286c23d`** |
| P3 | `make gate` on `94286c23d` | **RED** — 31 PASS, 1 FAIL (`cb-2113-cb-2115`). Root-caused, not worked around — section 4 |
| P3b | lifecycle-12, PR **#1411** | rows `#1410`. Four rounds; **one** (round 2) produced no verdict at all, and round 1 produced a real FAIL — section 4. Final: **3/3 PASS, `partial=false`**. CI 47 SUCCESS / 0 FAILURE. Merged **`516305ef0`** |
| P4 | `make gate` on `516305ef0` | **GREEN — 32 PASS, 0 FAIL.** 16 CI-only rows printed by name, not counted |
| P4b | package size | `pmat-3.41.1.crate` **9,359,632 B = 8.9260 MiB compressed**; cargo prints `8.9MiB`. Gate fails at `>= 9.0` -> **PASS**, 77,552 B headroom, 99.18% of the repo's own 9.0 MiB budget (`feature-matrix.yml:507`), 89.26% of crates.io's 10 MiB. sha256 `efb7c2af9f937ee0b359d7019d19612b5caf57c70dfac407066060d7cb86b28f` |
| P5 | tag | annotated **`v3.41.1`** -> `516305ef07f7197b9eb9a773c609f834dbdf4c5b`, pushed |
| P6 | **the clean room** | `release.yml` run **35351362363**, job `gate / cpu-gates` — **SUCCESS**, 13:41:06Z -> 14:39:14Z (**58 min**). `gate / gate` success, `gate / lint-gate` success, `gate / gpu-gates` skipped, `verify` success. First attempt failed in 16 s on infrastructure — section 4 |
| P7 | publish | `cargo publish --dry-run --locked` **exit 0**; `cargo publish --locked` **exit 0**, `Published pmat v3.41.1 at registry crates-io`. Both from a **detached** checkout of the tag (`git describe --exact-match` -> `v3.41.1`, `git status --porcelain` empty), `env -u CARGO_REGISTRY_TOKEN`, never `--allow-dirty` |
| P7b | read back | registry API: `max_version 3.41.1 · max_stable 3.41.1 · newest 3.41.1`; top five `3.41.1 3.40.2 3.40.1 3.40.0 3.39.0` — **3.41.0 absent**, as it always was. `/api/v1/crates/pmat/3.41.1` **HTTP 200** |
| P8 | promote | `gh release edit v3.41.1 --prerelease=false` with run **35351362363** in the notes -> `isPrerelease: false, isDraft: false` |
| P8b | `make release-check` | **exit 0** — `Cargo.toml 3.41.1 · latest tag 3.41.1` / `3.41.1 is tagged, released and on crates.io` |
| P9 | dogfood the INSTALLED binary | section 5 |

---

## 4. Three reds, each root-caused rather than retried

### 4.1 `make gate` RED on the release merge commit — the release's own alarm

`cb-2113-cb-2115` failed on `94286c23d` with `✗ CB-2115: 1 finding(s) — ORPHAN-GITHUB #1410`.
(`✓ CB-2113` read `not_applicable: HEAD is the default branch`.)

**Five whys.** (1) `make gate` is red because CB-2115 found ORPHAN-GITHUB #1410. (2) #1410 is
an orphan because it was auto-filed at **2026-09-18T11:21:32Z** and no roadmap row existed for
it. (3) It was filed because master's `Cargo.toml` declared 3.41.1 while crates.io's
`max_stable` was 3.40.2 — **exactly true**, and precisely the alarm's purpose. (4) That reds a
gate on the release itself because a release is necessarily in that state between the
version-bump merge and the publish; the alarm is designed to fire in that window and CB-2115
requires bijection at all times. (5) **Root cause: the release-check filer and CB-2115 are in
tension by construction.** Every release reds `make gate` from the version-bump merge until
the publish. #1401 is the identical artefact for 3.41.0.

**Owner: the repo's own gates, not this release.** Fixed the sanctioned way — a lifecycle row
through `pmat work sync --direction github-to-yaml`, whose plan was exactly one action
(`create-item #1410 -> GH-1410`) and whose diff is 16 lines in one file. 113/114 -> **114/114
coherent**. It deserves its own ticket after the release; it was not changed inside one.

**Consequence for the tag, stated plainly.** `v3.41.1` is on `516305ef0`, the lifecycle-12
merge commit, **not** on `94286c23d`, the release PR's merge commit. `make gate` is RED on the
latter by construction and GREEN on the former, and `git diff --name-only 94286c23d 516305ef0`
is **docs-only** (`docs/audits/impl-PMAT-1336-receipt.md`, `docs/audits/quorum-PMAT-1336.json`,
`docs/roadmaps/roadmap.yaml`) — no code. The brief said "tag on the merge commit"; this is the
merge commit whose gate is green and whose package was measured.

### 4.2 The clean room's first attempt: 16 seconds, no verdict

Run 35351362363's first `gate / cpu-gates` failed at 13:38:39Z, **16 seconds** after starting,
on runner `gx10-pool1` (the 3.41.0 attempt ran on `intel-clean-room-5`). The failure:

```
Cloning into '/home/runner/src/infra'...
Host key verification failed.
fatal: Could not read from remote repository.
##[error]Process completed with exit code 128.
```

The runner could not clone `paiml/infra`, so **no gate ran and no verdict was produced**. This
is not a red verdict on the release; it is an unprovisioned runner.

**The brief allows one re-run for a runner-lost-communication failure, named.** This is not
literally that, and the extension is declared rather than smuggled: it is an infrastructure
failure of the same character — the job never executed a gate — and re-running is the only way
to obtain a verdict at all. It was re-run **once**, with `gh run rerun --failed`. The re-run
landed on a provisioned runner and ran the full 58 minutes to **success**. Had it failed
again, this receipt would read `STOPPED`.

### 4.3 `quorum-review.sh` edited underneath a running instance

Round 2 on #1411 died with `line 310: lint.sh: command not found` and wrote no lanes.
`quorum-review.sh` was modified at **14:08:07** by another process on this host while that
instance was executing; bash re-reads a script by byte offset, so the edit shifted the offsets
and the run executed garbage. Two other sessions were live on this box.

Rounds 3 and 4 were run from a **byte-identical snapshot** of that script — sha256
`36ce57ae0fbcc8b4e0b542e0251a9abc2e8eb80847fe9bac53056c2c412ed500`, verified equal to the live
file before the copy and again afterwards — so a concurrent edit could not corrupt them.
**The gate was not modified: the snapshot is the same bytes.**

### 4.4 The quorum rounds in full, so the count is auditable

| PR | round | lane 1 | lane 2 | lane 3 | outcome |
|---|---|---|---|---|---|
| #1409 | 1 | PASS | PASS | **NO-VERDICT** | lane failure -> re-run |
| #1409 | 2 | PASS | PASS | PASS | `agreed=true partial=false` -> **armed** |
| #1411 | 1 | **FAIL** | NO-VERDICT | PASS | a real finding -> fixed, see below |
| #1411 | 2 | — | — | — | no verdict: script edited mid-run (4.3) |
| #1411 | 3 | PASS | PASS | PASS | `agreed=true` but `partial=true` -> re-run |
| #1411 | 4 | PASS | PASS | PASS | `agreed=true partial=false` -> **armed** |
| #1404 | 1 | **FAIL** | PASS | PASS | two real findings -> fixed, see below |
| #1404 | 2 | PASS | PASS | PASS | `agreed=true partial=false` -> **armed** |

**#1411 round 1's FAIL was correct and was not re-run away.** `quorum-review.sh:193` feeds the
lanes `docs/audits/impl-<ticket>-receipt.md`, and PMAT-1336 is a **standing** ticket that never
completes — so that file still described lifecycle-11 while the diff under review added
`GH-1410`. Lane 1 read the mismatch exactly as the refutation doctrine tells it to ("a receipt
claim is not backed by the diff"). Its proposed fix — remove the `GH-1410` row — is the one
thing that could not be done, since that row is what takes `make gate` from RED to green. The
receipt was made current instead, and the process finding recorded there: **a standing
ticket's receipt goes stale the moment its round lands, and the next round must make it
current before asking for a verdict.**

**#1404 round 1's FAIL was also correct, and was also fixed rather than re-run.** Two
findings, both cited against the diff. (a) The PMAT-1408 row in `impl-estimates.jsonl` recorded
`mode: "direct"`, which undercounts a session that dispatched an agy delegate at width 3 and ran
six `quorum-review.sh` rounds; the lane's own proposed value (`"orchestrator"`) is wrong for this
ledger — `mode` records the routing mix, and 10 of its rows are bare `direct` — but the field was
inaccurate and is now `direct + agy-delegate(grillme x3) + quorum-review.sh(width 3; 2 rounds
#1409, 4 rounds #1411, 2 rounds #1404)`. (b) §3's P3b row said "Four rounds; two produced no
verdict", which contradicts §4.4: only **round 2** produced no verdict; round 1 produced a FAIL.
The lane was right and the sentence was wrong; it is corrected above. (Its line citation for (b),
`:163`, is off — the sentence is at `:112` — but the substance stands.)

**#1411 round 3's `partial=true`** had one reason, byte-identical to what #1407 recorded:
`lane 1: non-empty .err (100 bytes, 1 line(s) beyond agy-lane's workspace narration)` — agy's
own shutdown narration (`root agent idle; waiting up to 5s for 1 background task(s)` /
`terminating 1 background task(s) on exit`), not review output. The brief permits arming on
that; the round was re-run anyway and round 4 came back clean, so **no caveat is claimed for
either merged PR.** The whitelist was never widened and no gate was edited.

---

## 5. Dogfood: the INSTALLED binary, not the tree

`cargo install pmat --version 3.41.1 --locked --root /mnt/nvme-raid0/targets/pmat-install-3411`
— from crates.io, i.e. the bytes a stranger receives.

| probe | result |
|---|---|
| `pmat --version` | `pmat 3.41.1` · `commit: unavailable (source archive: a .crate tarball carries no git metadata)` — exit 0 |
| `pmat roadmap aggregate --help` | exit 0, describes the fragment aggregator (#1364) |
| `pmat work estimate check` | exit 0, prints its exclusions (`range-phase`, `unit-not-turn`) — #1382 |
| `pmat work edit --help \| grep notes` | `--notes <NOTES>  New notes (markdown; replaces existing) …` — #1391 |
| `scripts/issue-closure-gate.sh <installed pmat> --self-test` | **exit 0**, `every arm as expected` |
| `scripts/issue-closure-gate.sh <installed pmat>` | **exit 0**, `PASS` — `scanned_rs_files=4500`, `definitions=4/4`, `call_sites=0`; text leg `scanned_files=5149`, `hits=0` |

### The #1403 reproduction, with its discrimination control

A throwaway crate `lockfix` with a two-line `Cargo.lock`, and a throwaway `CARGO_HOME` holding
one `[patch.crates-io] pmat = { path = … }` entry.

**Control first — the fixture CAN be dirtied.** A bare `cargo check` under that ambient patch
rewrites the lockfile:

```diff
  name = "lockfix"
  version = "0.1.0"
+
+ [[patch.unused]]
+ name = "pmat"
+ version = "3.41.1"
```

**Then the installed 3.41.1**, same directory, same `CARGO_HOME`:

- `pmat analyze dead-code` exit **0**, and it performed a **full scan** — `Dead functions: 1`,
  `src/main.rs - 100.0% dead`. This matters: the reverted `--locked` fix silently disabled the
  compiler scan (80 dead functions became 0), so a lockfile left clean by a scan that never ran
  would prove nothing.
- Lockfile sha256 **`6d2d47290a8bcf76e6bd26cf752066172d7b5e1eb4fe1d27ffd513f0c7517e53` before and
  after** — byte-identical. `diff` empty.

Both halves are required and both were measured: the fixture is dirtyable, and 3.41.1 does not
dirty it while still doing the work.

---

## 6. Slots, dispatch, denials

| # | mode | agent | model | width | turns | maxTurns? | resumed? | agy conversations |
|---|---|---|---|---|---|---|---|---|
| 1 | delegate | `paiml-agy-delegate` `a871a3c5ca0c3b25d`, lane `quorum`, mode `grillme`, `writes=false` | opus (delegate); lanes `gemini-3.1-pro-high`, `gemini-3.8-flash-high`, `gemini-3.7-flash-high`, all `model_source=measured` | 3 | 30 tool uses, 775 s | no | no | `9364f056…`, `a6d8ad4e…`, `1ff9a725…`; `child_conversations=3` |

**I-3 line:** `PASS transcript-gate: attempted=1 denied=0 stalled=0 running_peak=1 slots=3
segments=30 files=1 (agent_calls=1 resumes=0 workflow_started=0)`.

**slots** 3, live peak **1** — never more than one Claude subagent existed at a time.
**denials 0. stalls 0.** No `writes=true` agy lane ran, so the R-4 one-writer rule was
trivially satisfied; no `--concurrent-scope` was declared by anyone, so every lane asserted the
whole checkout. The four `quorum-review.sh` rounds are agy lanes launched by that script, not
Claude subagents, and are accounted for here rather than in the transcript gate.

---

## 7. Corrections to the brief

1. **CB-2113 did NOT refuse the first commit, and the PMAT-1336 vehicle was not needed for the
   release PR.** The brief anticipated a refusal because PMAT-1399 is terminal. Taking option
   (b) created a fresh **open** ticket, PMAT-1408, so `pmat comply check --checks CB-2113` read
   `✓` on `b00c5ae77` directly. PMAT-1336 was needed later, for the lifecycle row (4.1), and for
   the receipt commit in this PR — where CB-2113 *did* refuse `Pmat-Ticket: PMAT-1399` with
   "is completed — work belongs to an open item", exactly as predicted; that commit's trailer
   now names PMAT-1408.
2. **"Issue #1401 closes itself only if `make release-check` reads green" is false — nothing
   closes it.** `make release-check` exits 0 and #1401 and #1410 are both still OPEN.
   `quality-gate.yml:310-341` only **opens or refreshes** the issue, on failure; it has no close
   step. The single `issue close` string in `.github/workflows/` is inside `ci.yml`'s *control*
   proving that **no pmat code path can close a GitHub issue** (PMAT-900001,
   `contracts/pmat-issue-closure-v1.yaml`) — the opposite of a closer. Per that contract an
   issue closes only via a merged PR body's own `Closes` line. Neither was closed by hand here,
   and neither is closed by this PR: closing them turns two rowed issues into ORPHAN-ROADMAP
   findings and reds CB-2115 on the next PR, so the close and its lifecycle row belong together,
   in the orchestrator's hands.
3. **The PR title must be a conventional-commit type; `release:` is not one.** `PR Title Check`
   (`pr-checks.yml:28`, `amannn/action-semantic-pull-request@v6`) failed #1409 with `Unknown
   release type "release"`. Retitled `chore(PMAT-1408): …`. It re-runs on `edited` and passed;
   `gh pr view` keeps listing the superseded FAILURE alongside the two later SUCCESS runs on the
   same sha, and it is **not** in `required_status_checks` (`["ci / gate","feature-gate","docs
   build (docs.rs environment)","pmat score","provable ladder"]`). The *commit* subject
   `release: pmat 3.41.1 …` was accepted by the commit-msg hook — the two lint different things.
4. **Package size moved, as the brief suspected.** Session 1 measured 9,345,290 B; this release
   is 9,359,632 B — **+14,342 B** from #1406. Still under the 9.0 MiB gate, with 77,552 B left.
5. **`release.yml` does not publish to crates.io.** No `cargo publish`, no
   `CARGO_REGISTRY_TOKEN` anywhere in it; its `prerelease` job only creates the release object.
   The manual publish in P7 is the only publish, so there was no double-publish hazard.
6. **The `prerelease` job failed AFTER creating the release, and the failure is pre-existing.**
   Its "Dispatch the release listeners" step exited 1 on `could not create workflow dispatch
   event: HTTP 403: Resource not accessible by integration` — the workflow's default
   `GITHUB_TOKEN` has no `actions: write`, so `binary-release.yml` and `post-release.yml` were
   never dispatched by it and the release had **no attached assets**. Both were dispatched by
   hand afterwards (runs 35358864629 and 35358868718). This is a workflow-permissions defect,
   not a property of 3.41.1, and it would have hit any release. Named here rather than filed,
   because an un-rowed issue reds CB-2115 on every open PR (4.1) and this PR is open.
7. **The runner pool is not uniform.** 3.41.0's clean room ran on `intel-clean-room-5`;
   3.41.1's first attempt drew `gx10-pool1`, which cannot clone `paiml/infra` (4.2). A green
   clean room is therefore partly a function of which runner the job draws.
8. **`quorum-review.sh` is not safe against concurrent edits** (4.3). On a shared host it must
   be snapshotted before a long round, or a round can die with an error that looks like a
   quorum failure and is not.

---

## 8. Gaps

- **#1401 and #1410 are open and stale.** Both now assert something false. Neither is closable
  by this session without creating the ORPHAN-ROADMAP that correction 2 describes. Left to the
  orchestrator, together with their rows.
- **PMAT-1408's row is `planned` with issue #1408 open.** A ticket cannot complete itself under
  CB-2113; row completion is the orchestrator's, in the next lifecycle round.
- ~~The release has no attached binaries.~~ **Resolved and measured.** `binary-release.yml` run
  **35358864629** and `post-release.yml` run **35358868718**, both dispatched by hand after the
  403 in correction 6, completed **success**. `gh release view v3.41.1 --json assets` lists **12
  assets** — a `.tar.gz` and a `.sha256` for each of `aarch64-apple-darwin`,
  `aarch64-unknown-linux-gnu`, `aarch64-unknown-linux-musl`, `x86_64-apple-darwin`,
  `x86_64-unknown-linux-gnu`, `x86_64-unknown-linux-musl`.
- **`make validate-book` was NOT run this session.** Session 1 ran it and it passed on the same
  content; 3.41.1 adds one CHANGELOG entry and the lockfile guard, neither of which touches the
  book's chapters. This is a gap, not a pass, and is named as one.
- **The local clean room was not re-run.** Session 1's correction stands and is not re-litigated:
  `~/src/infra` on this host is 49 commits behind `origin/main` and lacks the infra#653 overlay,
  so a local clean-room green is **not** equivalent evidence to the CI one. The CI clean room on
  the tag's own sha is the evidence this release rests on, and it is green.
- **The release-check / CB-2115 tension (4.1) is recorded, not fixed.** It will red `make gate`
  on the next release too.

## Host

`uptime` before the first heavy job `15:12:05 up 21:40, load 6.11`; peak observed load **35.5**
on 48 cores, driven mostly by *other* sessions on this box (a ~20-core `apr qa` model eval, an
ffmpeg transcode, a peer session's agy lanes, a self-hosted runner). Memory never fell below
**96 GB available** of 125 GB. **No crash, no reboot.** Only one heavy job of this session's own
ran at a time. Disk: `/mnt/nvme-raid0` 2.2 T free before and after; `/` 286 G.
