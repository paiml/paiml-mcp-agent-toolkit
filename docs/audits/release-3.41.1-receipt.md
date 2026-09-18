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

**#1411 round 1's FAIL was correct and was not re-run away.** `quorum-review.sh:193` feeds the
lanes `docs/audits/impl-<ticket>-receipt.md`, and PMAT-1336 is a **standing** ticket that never
completes — so that file still described lifecycle-11 while the diff under review added
`GH-1410`. Lane 1 read the mismatch exactly as the refutation doctrine tells it to ("a receipt
claim is not backed by the diff"). Its proposed fix — remove the `GH-1410` row — is the one
thing that could not be done, since that row is what takes `make gate` from RED to green. The
receipt was made current instead, and the process finding recorded there: **a standing
ticket's receipt goes stale the moment its round lands, and the next round must make it
current before asking for a verdict.**

**This table covers #1409 and #1411 — the two merged PRs — and deliberately enumerates NO round
of this PR's own.** It cannot: every round that finds something rewrites this document, which
creates the next round, so any row count here is falsified the moment it is written. Three
successive revisions proved that empirically, each caught by lanes citing the row the previous fix
had just invalidated. The authority for **this** PR's rounds and its final verdict is the
committed artifact `docs/audits/quorum-PMAT-1408.json` — `agreed`, `partial`, `partial_reasons`,
`head`, and the three `model_measured` values — plus the per-round copies kept beside the run
logs. §4.5 narrates what those rounds *found*, which is stable; it states no count, which is not.

**#1411 round 3's `partial=true`** had one reason, byte-identical to what #1407 recorded:
`lane 1: non-empty .err (100 bytes, 1 line(s) beyond agy-lane's workspace narration)` — agy's
own shutdown narration (`root agent idle; waiting up to 5s for 1 background task(s)` /
`terminating 1 background task(s) on exit`), not review output. The brief permits arming on
that; the round was re-run anyway and round 4 came back clean, so **no caveat is claimed for
either merged PR.** The whitelist was never widened and no gate was edited.

### 4.5 What the rounds against THIS receipt found

**Round 1 (lane 1, `gemini-3.1-pro-high`) — two findings, both valid.** (a) The PMAT-1408 row in
`impl-estimates.jsonl` recorded `mode: "direct"`, which undercounts a session that dispatched an
agy delegate at width 3 and ran repeated `quorum-review.sh` rounds. The lane's own proposed value
(`"orchestrator"`) is wrong for this ledger — `mode` records the routing mix, and 10 of its rows
are bare `direct` — but the field was inaccurate and now names the mix it used. (This receipt
deliberately does **not** quote that field's value: an earlier revision did, the value was
corrected once afterwards, and the quotation went stale — which rounds 3's lanes 1 and 2 both
caught, independently and correctly. `impl-estimates.jsonl` is the authority for its own row.)
The finding was valid; its fix was not, and saying so is the point. (b) §3's P3b row read "Four rounds; two
produced no verdict", contradicting §4.4 two screens later: only **round 2** on #1411 produced no
verdict; round 1 produced a real FAIL. Corrected. (Its citation `:163` was off by fifty lines —
the sentence is at `:112` — but the substance stood.)

**Round 2 (lane 2, `gemini-3.8-flash-high`) — two findings, and the first is the worst defect this
session produced.** (a) §4.4 **pre-recorded round 2 of #1404 as `PASS | PASS | PASS`,
`agreed=true partial=false -> armed`, and the ledger's `mode` forward-counted that round — while
it was still executing and nothing had been armed.** That is a receipt asserting an
unmeasured outcome: precisely the failure the whole rail exists to prevent, committed by the
orchestrator into the document whose job is to prevent it. The lane quoted the line. It is
removed, the table now enumerates none of this PR's rounds at all, the preface above says why,
and the ledger's `mode` no longer forward-counts a round. (b) §6 still said "four
`quorum-review.sh` rounds" after §4.4 had grown past four — a stale total. Both fixed.

**Round 3 (lanes 1 and 2) — one finding upheld, one refuted by measurement.** Upheld, and both
lanes found it independently: §4.5 still quoted the *previous* `mode` value after the ledger had
been corrected, so the receipt and the ledger disagreed. Fixed above by removing the quotation
rather than re-synchronising it, because a prose copy of a mutable field will go stale again.
Refuted: lane 1 also claimed PMAT-1408's `"basis":"…:L27-L38"` is self-referential because "the
PMAT-1408 row itself is inserted at line 38". Measured — `grep -n 'PMAT-1408'
docs/audits/impl-estimates.jsonl` puts that row on **line 40**, and `sed -n '27,38p'` shows the
range ends at PMAT-1403 on line 38. The basis excludes the row it explains, which is what it
should do, and it was computed by `estimate.sh` before the row existed. A cited grounding is not
a correct citation, and this one is recorded as refuted rather than silently accommodated.

**Round 4 (3/3 FAIL) — one defect, found by all three lanes independently, and it was mine
twice over.** The fix for round 3 had patched §4.5's copy of the round-1 narrative while leaving a
**stale duplicate of the same paragraph in §4.4's prose**, still reproducing the superseded ledger
value. So the correction announced in §4.5 was contradicted, verbatim, twenty lines above it. Two
lanes additionally noted that §4.4's table had gained no row for round 3 and that §4.5's heading
still said "Two rounds". All three findings upheld. The duplicate paragraph is deleted, §4.4 now
enumerates none of this PR's rounds and says why, the heading counts nothing, and no copy of that
ledger field survives anywhere in this document — verified by `grep`, not by reading.

That is the third consecutive round whose finding was created by the previous round's fix, and the
pattern, not the individual defects, is the lesson: **a receipt that narrates its own review will
generate a finding per revision for as long as it keeps a count or a quotation that the next
revision can falsify.** The structure above removes both. What §4.5 states from here on is only
what a round *found*, which no later round can invalidate; whether a later round exists, and what
it concluded, is answered by `docs/audits/quorum-PMAT-1408.json` at the merged head — which is
where a reader should look, and the only place that can be right.

**Round 5 (lane 1) — one finding upheld, one already disclosed, one refuted.**

*Upheld, and it is the sharpest finding of the session:* **PMAT-1408's own acceptance criteria
assert that `make release-check` exiting 0 "is what closes #1401" — and #1401 is OPEN.** That
criterion is unmet as written and cannot be met, because the premise behind it is false: nothing
closes a `release-check` issue (correction 7.2, measured). I wrote that criterion myself when
filing the ticket, taking the brief's premise on trust instead of measuring it first. So the
ticket's definition of done contains a claim this work disproved. **The criterion is what is
wrong, not the outcome**, and it is named here rather than left for a reader to trip over;
PMAT-1408's `notes` carry the same correction, and amending the criterion is the orchestrator's,
since a ticket cannot complete itself. This receipt does **not** claim that criterion satisfied.

*Already disclosed:* the lane objected that re-running the clean room for
`Host key verification failed` extends the brief's allowance, which only names
runner-lost-communication. It does, and §4.2 says so in those words — the extension was declared
before the re-run, not excused after it. The lane is right that it is an extension; the
disclosure is the answer, and the re-run then produced a real 58-minute verdict rather than none.
Recorded, unchanged.

*Refuted:* the lane called `#1404` "an unrelated PR … a typo leftover". #1404 **is this pull
request** — `gh pr view 1404` returns this branch, `PMAT-1399-release-receipt`, and this receipt
is its diff. Grounded `cited` and wrong, like round 3's line-38 claim.

**No FAIL was re-run away, and none is quietly dropped.** A receipt that hides the rounds which
criticised it is worth nothing. Every lane FAIL against this receipt found a real defect in this
session's own artefacts; every upheld one was fixed at the source, and the single finding that
measurement refuted is recorded above as refuted rather than accommodated. The rounds themselves
live in `docs/audits/quorum-PMAT-1408.json` and the kept per-round copies. No total is stated
here, for the same reason §4.4 enumerates no row for this PR and §6 quotes no total either.

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
whole checkout. Every `quorum-review.sh` round tabulated in §4.4 consists of agy lanes launched by
that script, not Claude subagents, so they are accounted for there and in each PR's committed
`docs/audits/quorum-*.json`, rather than in the transcript gate — which sees Claude subagents
only. No total is quoted here on purpose: §4.4 is the count, and it grew twice while this
document was being written.

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
- **PMAT-1408's acceptance criteria are wrong where they say `make release-check` green "is what
  closes #1401".** Nothing closes that issue (7.2). The criterion was written on the brief's
  unmeasured premise and is disproved by this session's own measurement; it needs amending, which
  a ticket cannot do to itself. `pmat work edit --notes` records the correction on the ticket.
  Every other criterion in PMAT-1408 is met and evidenced above.
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
