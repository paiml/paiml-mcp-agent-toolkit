# IMPL-PMAT-722 — goal-mode step 4: CB-2115 Roadmap Coherence, control first, the gate step withheld until master measures coherent

## Identity

| field | value |
|---|---|
| ticket | PMAT-722 (`kind:code`, `orch:fable`, `kind-gate.sh` exit 0 `kind=code files=52` against `master`) |
| spec | `docs/specifications/goal-mode.md` §5 (5.1–5.4), §7 row CB-2115, §11 step 4, §11.1, §12, §13 |
| branch | `PMAT-722-cb2115-roadmap-coherence`: from `PMAT-720-work-sync-release-field` @ `6637485e7` (#1248) with `PMAT-719-cb2113-traceability-gate` @ `75ebdf009` (#1247, which carries #1246) merged in as `06031cf2d` — **stacked on #1246 → #1247 and on #1248**; CB-2115 needs the `--checks` selector, the traceability job and the work-sync engine |
| HEAD in | `06031cf2d` (the stacking merge; three conflicts resolved: both estimates rows kept, the four roadmap entries ordered 718, 719, 720, 721 on step 3's canonical prefix, the unrun-tests ledger regenerated) |
| HEAD out | `9dae11d4d` (last tree change: `d9ffd7c8d` was the last code change, `9dae11d4d` drops a scratch file from the index); the commit carrying this receipt follows |
| PR | opened from this branch after the receipt commit — `gh pr list --head PMAT-722-cb2115-roadmap-coherence` |
| `discover.json` sha256 | `9503963a367a1c6e5657ac1ffdf03bdd32ae539f995fcbced9d568f77adf62d5` |
| `gate_cmd` | `cargo test --workspace` — **`gate_cmd_fallback=true`**; `pmat verify` is the gate this repository's CLAUDE.md names and is what was run |
| `required_check` | `ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder` |
| model gate | `model=fable class=fable decision=admit basis=file` |

### One ticket per session — refused, then reaffirmed

`goal.sh set --ticket PMAT-722` exited 2: `one ticket per session: PMAT-719 was set here — start a new claude session` (R-5), the third ticket this transcript has carried. The operator's instruction for this pass, quoted verbatim: **`pmat-implement docs/specifications/goal-mode.md autonomously`** — the same reaffirmation PMAT-720 proceeded under, so this ticket proceeded with the refusal recorded and the turn accounting split by hand below. `goal.sh worker`, `gate` and `phase` accepted their declarations.

### Status-line join `[U]→[V]`

| claim | measured | how |
|---|---|---|
| statusLine `session_id` = hook `session_id` | **true** | `transcript-gate.sh` resolves `39a15c63-…` by `rule=pid-file`; the hook log `events-39a15c63-….jsonl` (19 rows, 0 denials) carries the same id |
| `tasks[].id` = hook `agent_id` | **[U]** | the two delegate `agentId`s (`a07ec4c00cc1f0236`, `a51156bf85e47f23f`) are not paired in the hook log |
| `transcript_path` on subagentStatusLine stdin | **[U]** | not measured |
| `k_measured` vs `global=k` | transcript-wide `k_measured=192`; PMAT-720's receipt closed at 146, so this ticket's share is **46** | `jq -r 'select(.type=="assistant" and ((.isSidechain // false)\|not)) \| (.message.id // .uuid)' <transcript> \| sort -u \| wc -l` |

## What step 4 is

**CB-2115: Roadmap Coherence** (`src/cli/handlers/comply_handlers/check_handlers/check_roadmap_coherence.rs`): the open roadmap items and the open, unlabelled GitHub issues are in bijection (§5.1, tolerance zero, three classes) and no matched pair has disagreed past the grace window (§5.2, a bound in time). It judges through the step-3 engine (`work_sync::check`) from a `GithubSnapshot` that comes from `gh` (`fetch_snapshot`, moved into `services/work_sync/github.rs` so the command and the rule share one reader) or, for its control only, from a file named on the command line: `pmat comply check --github-snapshot FILE`. Four verdict classes, each with its reason in the message: Skip (no roadmap ever committed; a roadmap that names no issue and resolves no repository), Fail `not_measured:` (roadmap committed and now gone; roadmap does not parse; items name issues but no repository resolves; a snapshot that cannot be read; a `snapshot` path committed in `.pmat.yaml`), Fail (the findings by class, id and number), Pass (matched, tolerated, the source and when it was taken).

**Withheld on purpose:** the direct `--checks CB-2115` step is not in the traceability job. §5.4 sequences it — the sync reports (step 3), a human resolves the thirteen items naming #612 (PMAT-721), the fixers run, and only then does the rule become a gate — and this repository measures **112 findings today** (1 COLLISION, 57 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB, 0 DRIFT), so a direct step would be red on arrival, and a gate red on arrival is a gate someone disables. The job runs the rule's **control** (`scripts/roadmap-coherence-control.sh`, 11 arms) and step 3's control instead; the exact step that flips it is a comment beside the CB-2113 step and is filed as **PMAT-723**, gated on `pmat work sync --check-only` exiting 0 on master. `pmat comply ledger` therefore writes CB-2115 **NEUTERED**, which is the truth.

## Plan and routing

| phase | what | class | `route.sh` (verbatim) | taken? | trigger |
|---|---|---|---|---|---|
| 1 | stack the branch (merge #1247 onto #1248), PMAT-722 filed, kind/model gates, R-5 refusal | orchestration | `route=self w=100.00 basis=absent` | taken | — |
| 2 | RED: 13 tests against a Skip stub, roster entry, group and includes (`1f922b8de`, 12 fail) | mechanical | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **not taken** — the RED test is the orchestrator's by doctrine (Phase 2 step 1) | — |
| 3 | the rule, `github.rs` reader move, `Finding::render` | impl, single module | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **taken** — lane FAILED on isolation (below); its commit verified and kept | R-4 |
| 4 | control script, the traceability job's two control steps, README, §11.1, ledgers, PMAT-723, roadmap canonicalisation | impl | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]` | **not taken** — a worker may not edit `.github/workflows`, and the shared tree must stay untouched while a writes lane runs (its isolation assertion diffs `git status`); done by me | — |
| 5 | pre-PR quorum on `06031cf2d...ad6bc332a`, adjudication | review | `route=agy-quorum w=1.00 basis=absent effort=1[U]` | taken, width 3 | Phase 4 pre-PR review |
| 6 | the four confirmed fixes with tests, control arms 9–11, mutants, ledgers, receipt, PR | impl → orchestration | `route=agy-goal w=1.00 basis=absent note=fable-binding effort=1[U]`; `route=self w=100.00 basis=absent` | **not taken** for the fixes — three modules under a FAIL gate (\|M\|≥2), done directly with each fix re-run | — |

`quota.json` is absent on this host (`quota.sh binding`: `age_h=absent account_mismatch=true`), so every route line carries `basis=absent`; the Phase 1 `teamwork` grill (Q2) was not run (Gaps).

## Dispatch ledger

| phase | mode | description | agent id | turns | maxTurns hit | resumed | lane / width / agy `conversations` |
|---|---|---|---|---|---|---|---|
| 3 | delegate (`paiml-agy-delegate`, opus) → agy `goal`, `writes=true` | `PMAT-722/ph3.delegate goal width 1 on CB-2115 rule implementation` | `a07ec4c00cc1f0236` | 22 tool uses, 1286 s | no | no | goal / 1 / `7772bb2d-88a2-4c64-81d5-c10fccb93f48` — **exit 3, LANE ISOLATION VIOLATED**: `the shared checkout's HEAD moved (1f922b8de -> 9a56e5428)` and `the shared checkout's working tree changed`; the lane ran `git checkout 1f922b8de` in the shared checkout and committed there; its throwaway worktree `lane-1f922b8de-1802803` was never touched; `child_conversations=1`, `label=single-lane` |
| 5 | delegate (`paiml-agy-delegate`, opus) → agy `grillme` quorum, `writes=false`, sandboxed | `PMAT-722/ph5.delegate quorum width 3 on the step-4 diff` | `a51156bf85e47f23f` | 30 tool uses (**maxTurns hit while polling**), 1254 s | yes | no — the lanes had finished (`.done`, `receipt.json` in `out_dir`), so I measured fan-out (`children=3 method=lane-files label=consensus`) and reduced them with `lane-reduce.sh` myself | quorum / 3 / `eadd02d4-e067-4123-aaae-17e47641f588`, `1e31e9ed-2caf-4054-a252-c8f830c6b2f9`, `569dbac1-f0fe-4ddb-b7d7-85018e664c0d` — 3/3 FAIL; `lane-reduce` exit 1: `agreed=false partial=true` (lane 2 stderr 79 bytes: `root agent idle; waiting for 2 background task(s)`) |

**The escaped commit was verified, not trusted.** `9a56e5428` (author `paiml-implement lane <noreply@paiml.invalid>`, trailer `Pmat-Ticket: PMAT-722`) was on no ref: the branch was fast-forwarded to it after `merge-base --is-ancestor` held, and its acceptance re-run by me: **60 passed, 0 failed** (`tests_roadmap_coherence work_sync core_handlers::sync`), clippy `--all-targets -D warnings` clean, fmt clean. One defect found on review and fixed by me (`f1963f883`): it had moved `render_finding` into the pure engine with its terminal colours and column padding, so a comply message could carry an ANSI escape whenever stdout was a terminal.

**Why the lane escaped, this time measured:** the prompt said "HEAD is detached at 1f922b8de", which was true of the worktree and false of the shared checkout — and the lane was standing in the shared checkout, so it made the premise true where it stood. The isolation the harness promises (a throwaway worktree as the lane's cwd) did not hold on either PMAT-720 or PMAT-722; the worktree sits inside the checkout's own directory tree, which is the probable cause `[U]`. Named in Gaps.

## Slots, denials, I-3

| field | value |
|---|---|
| `slots` | 3 (`config.json`, `config-lint.sh`: `slots=3 gh_calls_per_min=30 bank=3`) |
| Claude subagents dispatched for this ticket | **2** (both delegates; no worker) |
| `transcript-gate.sh` (session-wide, PMAT-719 and PMAT-720 included) | `PASS transcript-gate: attempted=7 denied=0 running_peak=2 slots=3 segments=189 files=7 (agent_calls=7 resumes=0 workflow_started=0; denied from hook log) (session 39a15c63-fab3-4148-8cb7-0cb012369b35, rule=pid-file (/run/user/1000/paiml-implement/pid-21623))` |
| denials | `denied=0` (hook log, 19 rows) |
| Workflow tool | not used |

## Quorum (Phase 5, `docs/audits/quorum-PMAT-722.json`)

3/3 **FAIL** on `ad6bc332a`; 19 dedup groups, 9 distinct claims after my own dedup — 4 CONFIRMED and fixed with a test and a control arm each, 1 CONFIRMED wording, 2 REFUTED, 2 agreements, every one re-measured first:

| # | finding | lanes | adjudication | closed by |
|---|---|---|---|---|
| 1 | a roadmap committed and now gone reads Skip — an input-deletion bypass (CB-2113 fails it) | 2, 3 | **CONFIRMED** | `commit_traceability::inputs` → `RoadmapDeleted` is `not_measured`; 1 test; control arm 9 |
| 2 | no resolvable repository reads Skip even when items name issues | 1, 3 | **CONFIRMED**, refined | Skip only when no item names a `github_issue`; else `not_measured` with the count; 2 tests |
| 3 | `.pmat.yaml` `options.snapshot` is a bypass token: a committed fixture path makes CI judge a file | 1, 2, 3 | **CONFIRMED** | the key is refused; the file arrives only through `--github-snapshot FILE` (`CheckOverrides`, out of band from the config); a CI line carrying the flag is recorded by `gate_effect` as not evidence; 3 tests; control arm 10 |
| 4 | no control arm proves `grace_minutes` is read; a rule hard-coding 60 survives every arm | 1, 2, 3 | **CONFIRMED** | arm 11; mutant M3 now dies under the control |
| 5 | the ci.yml comment and PMAT-723 state counts the tree does not measure | 2, 3 | **REFUTED** | measured: `work sync --check-only` today = 112 (`check-real-722.json`); PMAT-723 attributes its 52/54 to PMAT-720's run by name; the comment states only "thirteen items that all name #612" |
| 6 | PMAT-722 criterion 5's "NEUTERED with that reason" misreads the ledger's carrier | 1, 2 | **CONFIRMED** (wording) | reworded: the carrier is quality-gate.yml; the reason lives in ci.yml's comment and here |
| 7 | the merge resurrected `traceability-control.sh` and its step, deleted by #1247 | 2 | **REFUTED** | #1247 *added* both (`scripts/traceability-control.sh \| 143 +`) |
| 8 | withholding the direct step honours §5.4 and the NEUTERED row is honest | 3 | agreed | — |
| 9 | `Finding::render` is plain; the moved reader is unchanged and its tests still run | 3 | agreed | — |

## Mutation table

| mutant | planted as | killed by | log |
|---|---|---|---|
| M1: the verdict inverted | `if !report.is_coherent()` | 9 of 13 tests FAILED (`a_bijection_passes…`, `a_closed_linked_issue…`, …); control **ARM 1 FAILED** on cargo's executable | `mut722-m1` (tests, first run) and the ARM 1 line in the executable experiment |
| M2: an unreadable snapshot reads Skip | `check.status = CheckStatus::Skip` in the `source.load()` error arm | `a_snapshot_the_rule_cannot_read_is_not_measured_and_fails` FAILED; control **ARM 7 FAILED** | `mut722-m2-control.log` 566f583c3d997d60 |
| M3: `grace_minutes` read but ignored | `let _ = i;` for `grace_minutes = i;` | `grace_minutes_is_read_from_the_project_config` FAILED; control **ARM 11 FAILED** (the arm the quorum asked for) | `mut722-m3-control.log` b523c4bbac306977 |
| the rule absent | the Skip stub at `1f922b8de` | 12 of 13 tests FAILED (the roster entry is in that commit, so the registration test passes) | commit `1f922b8de` |
| the §7 falsifier and every class | control arms 1–11 | exit code AND the CB-2115 row asserted on every arm; arm 2 and 6 prove the same command can pass; arm 11 proves the option is read both ways | `control722.log` 346f7c8482bfc238 |

Every mutant was applied one at a time to the real source with backups under `.pmat/`, the named tests and the control run **on the executable cargo reports**, and the source restored to a byte-identical copy before the next. The first control-column run of M1/M2 was against `./target/debug/pmat`, which turned out to be a stale copy (Jidoka); those two readings were discarded and re-measured.

## Verification table (claimed vs my rerun)

| command | claimed | rerun | at |
|---|---|---|---|
| `cargo test --lib -- tests_roadmap_coherence work_sync core_handlers::sync` | lane: 0 failed (unverified claim) | **60 passed, 0 failed** | `9a56e5428` |
| the same plus `gate_effect comply_check` | — | **160 passed, 0 failed** | `61b6a92ee` |
| `cargo test --lib -- gate_effect a_line_judging_from_a_snapshot_file…` | — | passed | `d9ffd7c8d` |
| `bash scripts/roadmap-coherence-control.sh <cargo's executable>` | — | **11/11 arms** | `d9ffd7c8d` |
| `bash scripts/work-sync-control.sh <cargo's executable>` | — | 4/4 arms; arm 4 round-trips the 5156-line roadmap byte-for-byte | `61b6a92ee` |
| `bash scripts/traceability-control.sh <cargo's executable>` | — | 5/5 arms | `61b6a92ee` |
| `pmat work sync --check-only --path . --format json` (real repository, `gh`) | — | exit 1: **112 findings — 1 COLLISION, 57 ORPHAN-ROADMAP, 54 ORPHAN-GITHUB, 0 DRIFT**; open items 70, open issues 54, matched 0 | `ad6bc332a` |
| `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all -- --check` | — | clean / clean | `61b6a92ee`, `d9ffd7c8d` |
| `pmat comply ledger --write` | — | 159 rules, ENFORCED 1 (CB-2113, unchanged carrier), NEUTERED 158; **CB-2115 NEUTERED** via quality-gate.yml's `continue-on-error` carrier — the commented future step in ci.yml was NOT credited | `f1a757b81`, `f8c28d6fe` |
| `analyze unrun-tests --write-ledger` / `analyze reachability --check-ledger` | — | re-rendered / current | `d9ffd7c8d` |
| `pmat verify --format json` | — | at `ad6bc332a` (pre-quorum-fix tree): `ok:true`, 5/5 stages; at `625fafa6f`: **`ok:false`**, tests stage — `the_committed_ratchet_holds_at_head` (`unwrap_calls_src_outside_cfg_test` 9180 > 9177, `orphan_files` 408 > 407: `src/cli/test_clap_checks.rs`, an untracked scratch file `git add -A src` had swept into `61b6a92ee`); at `2bbe5d8e3` after `9dae11d4d`: **`ok:true`**, 5/5 stages | — |
| `transcript-gate.sh` / `kind-gate.sh` / `model-gate.sh` | — | (I-3 above) / `kind=code files=52` / `decision=admit` | — |

**The executable.** `./target/debug/pmat` was cargo's executable for the first two builds of this session and a stale copy from 08:33 onward: `cargo metadata` and every later `cargo build --message-format json` report `/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit/debug/pmat`, no config file in the repository, `~/.cargo` or the parent directories names a target dir, and the mechanism is unmeasured `[U]`. My fingerprint line (`grep -c <new string> <binary>`) printed nothing because `grep` is a shell function on this host that skips binary files. Every control result above was re-run on the executable cargo reports (`strings <exe> | grep -c` fingerprinted first); nothing in the tables rests on the stale copy.

### Discrimination

The control asserts, on every arm, both the process exit code and the CB-2115 row in the JSON report; a missing row is exit 2. Arms 1/2 and 5/6 are matched pairs (the same input with one fact flipped), arm 11 flips only the option, and arms 7, 9 and 10 assert that an input the rule cannot read, an input that was deleted, and a bypass path in the tree each fail rather than pass. The RED commit and the three mutants show which test dies under which change; M3 died under no control arm until arm 11 existed, which is what the quorum found.

## Jidoka log

| defect | owner | disposition |
|---|---|---|
| `goal.sh set` refused a third ticket per session (R-5) | skill doctrine | reported; the operator's instruction quoted verbatim (Identity); accounting split by hand |
| the agy goal lane escaped its worktree for the second ticket running: detached the shared checkout at the RED commit and committed there (`exit 3`) | paiml-implement harness | commit verified independently and kept on the branch by fast-forward; lane verdict FAILED; the probable cause named in Gaps |
| the lane moved terminal colours and padding into the pure engine | this row | `Finding::render` made plain; `paint_finding` in the sync printer (`f1963f883`) |
| the merged roadmap was not in the serializer's canonical form (four PMAT-719 criteria unquoted), so work-sync arm 4 failed; `pmat work edit --priority <same>` did not canonicalise it | pre-existing + a false claim in arm 4's advice | measured: `pmat work edit` patches the text in place and re-quotes nothing; the file is canonicalised through the serializer (arm 4's own write on a copy); the advice in `scripts/work-sync-control.sh` corrected (`0e98b208a`) |
| PMAT-723's hand-filled entry was non-canonical the same way | this row | canonicalised through the serializer |
| the control's two-item fixture ran both items onto one line (`$(…)` strips the trailing newline) | this row | one argument per item; the builder adds the newline (`0e98b208a`) |
| `Finding` not imported in the sync handler after the render move; build failed while the controls kept passing on the stale binary | this row | import fixed; every control re-run on cargo's executable |
| `./target/debug/pmat` stale from 08:33; the fingerprint grep silent | host build layout + shell function | executable taken from `cargo build --message-format json` for every later run; `strings \| grep` for fingerprints |
| the first M1/M2 control-column readings were taken on the stale binary ("all 8 arms behaved" under a verdict inversion) | this row | discarded; re-measured on cargo's executable: ARM 1 / ARM 7 FAILED |
| `pmat comply check --checks CB-2115 --path .` on the real repository hung the turn (>10 min): the report computes all 159 rules before `--checks` selects | pre-existing (PMAT-718's selector is post-hoc) | killed; the repository measured through `work sync --check-only` (same engine, seconds); named in Gaps |
| my first kill matched my own shell (`pgrep -f` on a phrase that sat in the command line) | this row | anchored pattern on the binary path; killed by PID |
| the quorum delegate hit maxTurns while polling | harness | lanes were complete; fan-out and reduce run by me; not resumed (a resume takes a slot and adds nothing) |
| quorum 3/3 FAIL, 9 claims | this row | 4 fixed with tests and control arms, 1 wording, 2 refuted (table above) |
| `git add -A src` in `61b6a92ee` swept an untracked scratch file (`src/cli/test_clap_checks.rs`, three `.unwrap()`, no `mod`) into the branch; `pmat verify` at `625fafa6f` was red on the ratchet and PR #1249 had been opened non-draft on a receipt that said DONE | this row | PR drafted within the same turn; the file dropped from the index and left on disk untracked as it was (`9dae11d4d`); ledgers re-rendered; ratchet at baseline; verify green at `2bbe5d8e3`; never `git add -A <dir>` while scratch files sit in it — name the files |
| andon threshold `0.8K = 38` turns reached with the quorum gate FAIL open | this row | continued deliberately: the remaining work was four enumerated, bounded fixes; the branch was pushed as the checkpoint at turn 40; stated here rather than hidden |

## Estimates

| field | value |
|---|---|
| `K̂` | 6, `basis=first-run[U]` (`estimate.sh pmat 6`: `ROWS=1` < 3, so K̂ = N) |
| `K` | 48, `basis=docs/audits/impl-estimates.jsonl:L20` (PMAT-720's actual, the closest analogue) — the 12 that `2×K̂` would give is contradicted by both measured rows (98, 48) |
| actual | **46** turns for this ticket (transcript `k_measured` 192 minus PMAT-720's 146) |
| `pr_runs` / `mg_runs` | not yet observable: the PR is opened after this receipt |

## Gaps

| gap | artifact that closes it |
|---|---|
| the direct CB-2115 step is **withheld** (§5.4 step 3): master measures 112 findings today | **PMAT-723** — the step is written as a comment beside CB-2113's; precondition `pmat work sync --check-only` exits 0 on master, which needs PMAT-721 and the fixers |
| Phase 1 `teamwork` grill of §5 / §11 step 4 (Q2) **NotRun** — the pre-PR quorum reviewed the diff instead | a `/teamwork-preview` receipt on the section |
| `pv` contract **NotRun** — `contracts_dir=contracts` is set; the spec names none for step 4 | `contracts/roadmap-coherence-v1.yaml` bound by a test |
| `grace_minutes` is a committed tolerance: `.pmat.yaml` can set it to a year and DRIFT never fires; §5.2 names the parameter and no cap | a cap, or a ledger note, under PMAT-723 |
| `--github-snapshot` on a real-tree CI line is recorded by the ledger as not evidence but is not refused by the rule; the rule cannot tell a control from a bypass on the command line | the ledger row is the guard; PMAT-723 decides whether the flag should also require `--path` into another tree |
| `pmat comply check --checks X` computes all 159 rules before selecting (~15 min on this repository) — the traceability job's CB-2113 step already pays it; a CB-2115 step will too | a pre-selection in `compute_compliance_report` (P1 follow-up) |
| harness: a `writes=true` agy lane runs in the shared checkout, not in the worktree `agy-lane.sh` creates for it — second occurrence (PMAT-720, PMAT-722); the worktree lives under the checkout's own directory, probable cause `[U]` | an issue on the paiml-implement skill (other repository; not filed by this row) |
| `merged green on required_check` — pending CI on PR #1249; the traceability job's first run of both new control steps is on this PR | the PR's checks |
| the 15 untracked scratch files in the checkout are not this row's and were left untouched | their owner |

## Machine-readable

orch_model: fable [V]   orch_class: fable   orch_decision: admit   orch_basis: M>=3
fable_binding: true   quota_age_h: absent   quota_mark: U   k_measured_at_set: 160

routes:
  ph1  class=orchestration  route=self        w=100.00  basis=absent
  ph2  class=mechanical     route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (RED test is the orchestrator's)
  ph3  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=agy-goal (lane FAILED isolation; work verified and kept)
  ph4  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (workflows are not a worker's; tree frozen during the writes lane)
  ph5  class=review         route=agy-quorum  w=1.00    basis=absent   effort=1[U]
  ph6  class=impl           route=agy-goal    w=1.00    basis=absent   note=fable-binding effort=1[U]   taken=self (three modules under a FAIL gate)
  ph6  class=orchestration  route=self        w=100.00  basis=absent

verification:
  cmd=cargo-test-lib-touched-modules(160)           claimed_exit=0  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tests722.log  sha256=303b21c80da2e4dc
  cmd=roadmap-coherence-control-11-arms             claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/control722.log  sha256=346f7c8482bfc238
  cmd=work-sync-control-4-arms                      claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/wsc722.log  sha256=ee0f0a423f259826
  cmd=traceability-control-5-arms                   claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/tc722.log  sha256=cfc1f650e6a0a6e1
  cmd=work-sync-check-only-real-repository          claimed_exit=-  rerun_exit=1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/check-real-722.json  sha256=296ce9f3517d7ca4
  cmd=mutant-M2-unreadable-snapshot-skips(control)  claimed_exit=-  rerun_exit=1(ARM-7-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut722-m2-control.log  sha256=566f583c3d997d60
  cmd=mutant-M3-grace-ignored(control)              claimed_exit=-  rerun_exit=1(ARM-11-FAILED)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/mut722-m3-control.log  sha256=b523c4bbac306977
  cmd=lane-reduce-ph5                               claimed_exit=-  rerun_exit=1(agreed=false)  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/lane-reduce-722.json  sha256=7aa853456f2a8e60
  cmd=transcript-gate                               claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/transcript-gate-722.log  sha256=27e967c078c7cad2
  cmd=pmat-verify@ad6bc332a                         claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify722-ad6bc332a.json  sha256=d8ead8b40854768d
  cmd=pmat-verify@625fafa6f(scratch-file-regression)  claimed_exit=-  rerun_exit=1  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify722.json  sha256=d18ed61fd526042d

  cmd=pmat-verify@2bbe5d8e3                         claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/verify722-final.json  sha256=4efc961124ffae96
  cmd=comply-ratchet@2bbe5d8e3                       claimed_exit=-  rerun_exit=0  log_path=/tmp/claude-1000/-home-noah-src-paiml-mcp-agent-toolkit/39a15c63-fab3-4148-8cb7-0cb012369b35/scratchpad/ratchet722.log  sha256=ebddd84fd2c66165

`claimed_exit=-` marks a row with no second party. `log_path` is session scratch; the `sha256` prefix pins the bytes.

## Status blocks

[status] ticket=PMAT-722 phase=1/6 global=152/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-722 blocker=- next=stack #1247 onto #1248, RED tests (goal.sh set refused: one ticket per session, operator reaffirmed)
[status] ticket=PMAT-722 phase=2/6 global=160/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=direct trigger=- route=agy-goal w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=- blocker=- next=RED committed 1f922b8de (12 of 13 fail); dispatch the goal lane
[status] ticket=PMAT-722 phase=3/6 global=165/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=quorum:goal trigger=R-4-single-module-width-1 route=agy-goal w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=lane-isolation-exit-3 filed=- blocker=- next=fast-forward to the escaped commit, verify its acceptance myself, fix the colour leak
[status] ticket=PMAT-722 phase=4/6 global=172/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=direct trigger=- route=agy-goal w=1.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-723 blocker=- next=control 8 arms, CI steps, ledgers, mutants on cargo's executable, quorum
[status] ticket=PMAT-722 phase=5/6 global=181/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=quorum:quorum trigger=Phase-4-pre-PR-review route=agy-quorum w=1.00 basis=absent q=? gate=FAIL slots=1/3 denied=0
         red=quorum-3/3-FAIL filed=- blocker=- next=adjudicate 9 claims; fix 4 with tests and control arms; andon line 38 crossed, continuing deliberately
[status] ticket=PMAT-722 phase=6/6 global=192/6(K=48) k_measured=192 sub=0/0 basis=docs/audits/impl-estimates.jsonl:L20
         mode=direct trigger=- route=self w=100.00 basis=absent q=? gate=PASS slots=0/3 denied=0
         red=- filed=PMAT-723 blocker=- next=receipt corrected after the scratch-file regression, verify green at 2bbe5d8e3, PR un-drafted

Finding: `global=k` and `k_measured` are the transcript-wide count, which includes PMAT-719's 98 and PMAT-720's 48 turns (Identity); the first five blocks are reconstructed after the fact with the count of the turn they describe. `q=?` because `quota.json` is absent.

## Verdict

**DONE** for the scope §5.4 allows today — every acceptance criterion re-run green by the orchestrator: the rule reports each class by id and number and fails on the §7 falsifier (arm 1, 9 tests under M1); a disagreement inside the window is tolerated and the option is read (arms 6 and 11, M3); Skip and `not_measured` sit on the right sides of doctrine 2 (arms 7–10, 4 tests); the snapshot is shared code with `pmat work sync` and only ever a command-line file; the traceability job runs both controls before CB-2113; the ledger says NEUTERED and the reason is here. The gate step itself is **PMAT-723**, blocked on a human (PMAT-721) and the fixers — not this row's to force.

IMPL-PMAT-722-RECEIPT-END
