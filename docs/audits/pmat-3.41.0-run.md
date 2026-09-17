# pmat 3.41.0 — autonomous fix-and-ship run log

Orchestrator: Claude Fable 5.1, session `4002dc6f`. Operator away; quorum decides scope; stop-the-line conditions are logged here, never asked.
Every entry: tree · ticket · step · raw measurement · decision + basis. Corrections to prior claims are findings, listed in the final section.

## 2026-09-16T14:52Z — setup and D0 location

tree: primary checkout `~/src/paiml-mcp-agent-toolkit` is on `feat/roadmap-fragments` (d70a78f67, behind=6) and is never used; all work in `~/src/paiml-mcp-agent-toolkit.wt/<ticket>` worktrees off `origin/master` (441d198e7, version 3.40.2).

### D0 — the issue auto-closer is GitHub's own keyword parser, not custom code

Raw measurements:
- `gh api repos/paiml/aprender/issues/3091/events`: two `closed` events, `2026-09-15T17:49:05Z` and `2026-09-16T06:50:47Z`, actor `noahgift`, `commit_id` null, `performed_via_github_app` null.
- GraphQL `ClosedEvent.closer`: `PullRequest 3323` and `PullRequest 3340` respectively.
- GraphQL `closingIssuesReferences` on the merged PRs: #3340 → [3091]; #3322 → [3091, 3321]; #3323 → [3091, 3303]. GitHub itself lists 3091 as a closing reference of every one of them.
- The PR bodies: #3323 `no-close: #3091 is the Qwen 3.5 epic …`, #3322 `no-close: #3091 is the Qwen 3.5 epic …`, #3340 `no-close: #3091 stays open; …`. Their `Pmat-Ticket` trailers were PMAT-3303, PMAT-3321 and PMAT-3091 — the first close came from PRs whose ticket was NOT 3091, which refutes the "Pmat-Ticket → #N" mapping hypothesis in the brief.
- pmat: `pmat query --literal ".close_issue("` → 0 call sites; `src/services/github_client.rs:196 close_issue` has no caller. `work_sync`'s `Action` enum has `CloseItem` (roadmap side) and no GitHub-side close; `apply_to_roadmap` never calls gh. `pmat work complete --help` has no issue-closing option.
- paiml-implement: the only close path is `scripts/mutate.sh:66` (`gh issue close`), which refuses without a quorum artifact + citation (exit 2); no `mutations.jsonl` names 3091. No Claude transcript, agy transcript, shell history, cron or systemd timer on this host issued a close of 3091 (grepped `~/.claude/projects`, `~/.gemini`, `~/.zsh_history`, `crontab -l`, `systemctl --user list-timers`).
- aprender `.github/workflows`: only `GITHUB_TOKEN` secrets (actor would be `github-actions[bot]`); no close call.

Mechanism (five whys): the issue closed → because GitHub recorded PR #3323/#3340 as its closer → because GitHub's closing-keyword parser matched `close: #3091` inside the body line `no-close: #3091 …` (the `-` before `close` is a word boundary, so `no-close:` IS `close:` to the parser) → because the `no-close:` convention was chosen to tell humans the PR does not close the issue, using the one word GitHub is guaranteed to parse → because nothing lints PR/commit bodies for a GitHub-parseable closing keyword that the author meant as a negation.

Decision (basis: the measurements above): there is no custom close path to delete in pmat or paiml-implement — the brief's premise is corrected. The D0 fix is (a) the convention: a "keep open" line must not contain any GitHub closing keyword (`close|closes|closed|fix|fixes|fixed|resolve|resolves|resolved`) followed by `#N`; (b) a pmat gate refusing such a line in commit messages / PR bodies (the `commit-msg` hook `pmat hooks install --strict` writes, plus `make gate`); (c) `contracts/pmat-issue-closure-v1.yaml`: `pmat work complete` never changes GitHub issue state, with the mutation evidence the brief asked for, where the "no-close" PR uses a non-keyword marker (`keep-open: #N`). aprender#3091 is OPEN now (reopened 2026-09-16T08:01:59Z); the D0 ticket is filed in pmat after D1 lands (fragment mode), so `pmat work add` does not touch `roadmap.yaml`.

### Sessions launched
- D1 PMAT-1363 (PR #1364): worktree `.wt/PMAT-1363`, branch `PMAT-1363-roadmap-fragments` from the PR head, headless `claude -p --model opus`. The first launch wrote its logs inside the worktree (launcher bug: relative log dir resolved after `cd`); killed within 2 minutes, worktree reset (`git reset --hard`, `git clean -fd`), relaunched. The first session had already rebased onto master (HEAD 4c85a06fc, behind=0, ahead=2 — the `chore(roadmap): file PMAT-1363's own entry` commit became empty because master already carries the entry); the relaunch inherits that tree.
- D4 PMAT-1365 (PR #1368): worktree `.wt/PMAT-1365`, branch `PMAT-1365-declare-gate-land`, same launcher, same reset/relaunch; HEAD b77f9d307, behind=0, ahead=1.
- D0, D2, D3 wait for D1 to merge (fragment mode) so their `pmat work add` writes `entries/<id>.yaml`. D4 lands the declared gate with an extension point; D0 and D2 append their own gate lines.
- D5: `pv` is `crates/aprender-contracts-cli` in paiml/aprender (not this repo); tickets go there.

## 2026-09-16T15:20Z — D5 ticketed in the owning repo; D0 warning relayed

tree: run-log worktree at 441d198e7 (= origin/master, behind=0).

D5 measured against aprender origin/main: `unlock.rs:7` already round-trips via `serde_yaml::Value` (needs the guard test only); the L2 index→link fix is open as aprender#3351; the validate denominator and executed-test counting are unaddressed. Filed in paiml/aprender (pv's owning repo):
- aprender#3397 — `pv validate` reports `N obligations, K evaluated, F failed`, exits non-zero on K == 0; `--check-ids` fails on anonymous obligations / dangling kani references.
- aprender#3398 — `#[ignore]`d / empty / never-executed tests count zero; `proof-status` reports `N tests (K executed)`.
- aprender#3399 — `pv unlock` round-trip pinned by a test the typed path fails.
- aprender#3400 — the D0 root cause (`no-close: #N` is `close: #N` to GitHub), with the evidence above.
- Commented on aprender#3351: its body carries `no-close: #3347`, which will close #3347 on merge.
pmat's PMAT-1369 (issue #1369) is now tracked by those four; its lifecycle row goes into the D0 ticket's PR.

## 2026-09-16T15:58Z — stop-the-line: shared-.git worktrees cannot pass lane isolation concurrently

Raw measurement (D4 session, phase-1 grillme lane stderr):
```
agy-lane: LANE ISOLATION VIOLATED — a shared ref changed: a lane created or moved a branch or tag (refs/heads/PMAT-1363-roadmap-fragments,refs/heads/chore/3.41.0-run-log)
```
`agy-lane.sh` hashes every ref of the repository (`for-each-ref`, line 957) before and after a lane and exits 3 on any change other than the lane's own branch. `git worktree add` trees share one `.git`, so a commit by the D1 session and the orchestrator's run-log commit both landed inside the D4 lane's window. With three tickets live this is structural, not a race to wait out.

Decision (basis: the assertion's own scope): each ticket tree is a standalone clone at the same path (`~/src/paiml-mcp-agent-toolkit.wt/<ticket>`, `git clone --branch <branch>`, origin re-pointed at GitHub, private refs), not a `git worktree`. The brief's "own worktree" intent — never the primary checkout, one tree per ticket — is kept; the mechanism changed because the mechanism was the defect. Both sessions killed and relaunched on the clones (D1 at 64042b343, D4 at b77f9d307, both behind=0). Two orchestrator mistakes on the way, both mine: a `kill` given both pids as one argument killed nothing, and `pkill -f` self-matched the orchestrator shell (exit 144) twice.

## 2026-09-16T17:10Z — sessions in flight

tree: run-log at 64fc628e7 (origin/master 441d198e7 + the log commit; behind=0).
- D4 PMAT-1365: pushed 1d8c64f40 to `PMAT-1365-declare-gate` ("make gate runs one table that maps every required check, and prints what it cannot run"); `gh pr checks 1368`: pass=34 fail=7 pending=2 at 17:10Z; the session is reading `.pmat-ratchet.toml` to root-cause the reds.
- D1 PMAT-1363: plan quorum passed; running `cargo test --lib roadmap_fragments` and the `roadmap`/`work_`/`ticket` filters; no new commit yet beyond 64042b343.
- Orchestrator: no ref moves in either clone since the relaunch; briefs and the spec sections for D0/D2/D3 are written and wait for D1's merge.

## 2026-09-16T18:25Z — stop-the-line: both sessions killed by the account session limit

Raw: both `claude -p` sessions ended at 16:20Z/16:21Z with result "You've hit your session limit · resets 8:20pm (Europe/Madrid)"; the orchestrator itself paused until the reset. D1 left 13 uncommitted files and one deleted file (`src/cli/test_clap_checks.rs`) in its clone; D4 left a clean tree at an unpushed commit 3425e9ed3. Decision: relaunch both at 18:22Z with resume notes naming that exact state; the D1 session is told to judge the deletion before keeping it. ~2 h of wall clock lost; no work lost beyond the sessions' own context.

## 2026-09-17T06:40Z — new orchestrator session; all three slots relaunched

Orchestrator: Claude Fable 5.1, session `a0a8b4ba` (the operator re-issued the brief; session `4002dc6f` is gone).

tree: run-log clone HEAD=64fc628e7 origin/master=441d198e7 behind=0. Primary checkout (d70a78f67, behind=6) still unused.

Raw measurements:
- `pgrep -af '^claude -p'` at 06:31Z: none. The third D1/D4 sessions' transcripts end at 2026-09-16T18:23:01Z and 18:22:48Z — ~90 s after launch, mid tool call; `uptime -s` = 2026-09-16 22:08:46 local (20:08Z). The reboot wiped `/tmp`, taking the previous launcher, its logs and the unlaunched D0/D2/D3 briefs with it. Why the sessions ended at 18:23Z is not recoverable from what is left; nothing was lost in the trees.
- D1 clone: HEAD=64042b343 behind=0 ahead=3, 13 dirty paths (identical to the 18:25Z entry). D4 clone: HEAD=3425e9ed3 behind=0 ahead=4, clean, 3 unpushed.
- D3: on 441d198e7 all 16 `repo=paiml-mcp-agent-toolkit` rows of `impl-estimates.jsonl` have `unit:null` (RED reproduced). Roadmap entry PMAT-1366 already exists (`labels: []`), so D3 needs no `pmat work add` and does not have to wait for fragment mode.
- D2: `pmat query --literal "fs::write(roadmap_path"` → 2 files, not 1: `ticket_validate_migrate.rs:526` (roadmap.yaml, unlocked — the defect) and `roadmap_handler_parsing.rs:180` (`apply_roadmap_changes`, which writes `- [ ]` checkbox lines — a markdown roadmap, to be confirmed by the D2 session). The brief's gate "returns 0 outside roadmap_service_io.rs" therefore has to be scoped by serialisation site, as the brief's own rule says, or it is red forever on a file that never holds YAML.

Decisions:
- Launcher, briefs and session logs now live under `~/src/paiml-mcp-agent-toolkit.wt/.run/` (not a repo, not `/tmp`) — basis: the reboot loss above. The launcher refuses a 4th live session. First launch failed on my own guard: `pgrep | wc -l` under `pipefail` exits 1 when nothing matches (the trap already recorded in memory); fixed with `|| true`.
- 06:32Z relaunched D1 PMAT-1363 (pid 2508652) and D4 PMAT-1365 (pid 2508663) with a fourth-session resume note; 06:34Z launched D3 PMAT-1366 (pid 2516703) in a new standalone clone on `PMAT-1366-estimate-ledger-unit` from 441d198e7. 3/3 slots live. D0 and D2 wait for a slot and for D1 (fragment mode) + D4 (`make gate` extension point).

## 2026-09-17T07:22Z — sessions in flight

tree: run-log HEAD=c4e119b4c origin/master=441d198e7 behind=0.
- D1 PMAT-1363: LIVE, HEAD 223b973c7 ("save() and every model writer emit fragments; pmat roadmap aggregate"), behind=0, 4 dirty paths, not yet pushed.
- D4 PMAT-1365: LIVE, HEAD 08eebd2e5, behind=0, clean, pushed; `gh pr checks 1368` at 07:21Z: pass=16 pending=26 skipping=4 fail=0. The session found that open issue #1381 had no roadmap row, which reds CB-2115 on every PR bound for master, and registered it on its branch (48cd21d03) — so D4 is now the PR that unblocks the others' traceability check.
- D3 PMAT-1366: LIVE, HEAD b994265b7, behind=0. It backfilled `unit` on ledger lines 1–18 from each row's own text or receipt and found a second defect on the way: rows 17–30 were keyed `repo=pmat`, not the ledger key `paiml-mcp-agent-toolkit` (the orchestrator's "16 rows" count was of the correctly-keyed rows only). Its writer's `--mode` flag collided with pmat's global `--mode` and panicked; renamed `--exec-mode`.
- D5: installed `pv 0.65.2` prints `0 error(s), 0 warning(s) / Contract is valid.` for `contracts/benchmarking-v1.yaml` with no denominator — the ground rule "N obligations, K evaluated, 0 failed" cannot be read off this pv's `validate` output at all until aprender#3397 lands. Each ticket session has to derive N and K another way and say how; decision on running aprender sessions for #3397–#3399 is taken when D0 and D2 no longer need the slots.

## 2026-09-17T07:50Z — D4 session ended PARTIAL(andon); #1381 row split out; CB-200 is a release blocker

tree: run-log behind=0 against origin/master 441d198e7.

Raw (D4 PMAT-1365 session receipt, HEAD 97037518a, behind=0): `PARTIAL(andon)` at k_measured=104 ≥ 0.8K=96. PR #1368 is back in draft, auto-merge not armed. `make gate` exists (44 rows: 28 run locally, 16 CI-only, each named); discovery on the branch reports `gate_cmd="make gate"`, `gate_cmd_fallback=false`. CI on 08eebd2e5: 42 pass, 0 fail. Two 3/3 PASS delegate quorums; the `quorum-review.sh` round was 2 PASS / 1 FAIL — the FAIL is scope: the roadmap row for issue #1381 is not PMAT-1365's. `make gate` is red locally on one leg: `the_committed_baseline_is_the_measured_count` (CB-200, 1742 below-A vs 1688 banked; identical on a clean master clone; #1266) — invisible to CI because CI checkouts have no index.

Corrections the D4 session made to this orchestrator's brief (findings): no rebase was needed; the PR's reds were its own ratchet/ledger drift plus CB-2115 on orphan #1381, not pre-#1360 staleness; 11 checks failed, not 7; `mutation-diff` is NOT a required check and `provable ladder` IS (branch protection: `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder`, plus the org ruleset's `gate`); most `feature-gate`/`pmat score` legs run locally — only the docs.rs build is wholly CI-only; `$XDG_RUNTIME_DIR` is tmpfs and does not survive a reboot either.

Decisions:
- The lane's scope FAIL is right (memory: a PR whose work is in no ticket's criteria needs its own ticket). 07:45Z launched a lifecycle session on the standing ticket PMAT-1336 (clone `.wt/PMAT-1336`, branch `PMAT-1336-register-1381`, pid 3020870) to land the #1381 row alone, byte-identical to the operator's unmerged `origin/PMAT-1381-row` (242755717), which is left untouched. Basis: until that row is on master, CB-2115 reds `gate → traceability` on every PR, D1/D3/D4 included. D4 resumes after it merges: rebase drops the row from #1368's diff and the objection with it.
- STOP-THE-LINE (logged, not asked): release precondition "make gate green" cannot be met on master today — CB-200 reads 1742 vs a banked 1688 and no bypass is acceptable (nor is raising the baseline; PMAT-636/#1266 says never). #1266 joins the fix queue as D6, after D0/D2. The release does not proceed past that gate until it is green by measurement.

## 2026-09-17T08:23Z — sessions in flight

tree: run-log behind=0 against origin/master 441d198e7.
- PMAT-1336 (the #1381 row): LIVE, HEAD d22870740, behind=0; PR: none.
- D1 PMAT-1363: LIVE, HEAD eef6881f8, behind=0, clean at 08:12Z. PR #1364 `gh pr checks` earlier at 07:38Z: 31 pass / 14 fail (on the pre-session head).
- D3 PMAT-1366: LIVE, HEAD 0cbff9129, behind=0; opened PR #1382 ("the estimate ledger gets one gated, append-only writer — `pmat work estimate record`").
- D4 PMAT-1365: no session; PR #1368 draft, waits for the #1381 row.
- Slots: 3/3.

## 2026-09-17T09:02Z — D3 MERGED (#1382, e89a827f7); orchestrator re-verification; one process finding

tree: run-log rebased, HEAD behind=0 against origin/master e89a827f7.

Raw (orchestrator's own rerun on master e89a827f7):
- `jq 'select(.repo=="paiml-mcp-agent-toolkit")|.unit' docs/audits/impl-estimates.jsonl | sort | uniq -c` → 26 `"turn"`, 5 `"unknown"`, 0 null.
- `estimate.sh paiml-mcp-agent-toolkit 4` → exit 0, `K_HAT=35 BASIS=docs/audits/impl-estimates.jsonl:L22-L31 ROWS=14 MEDIAN=35 EXCLUDED=9 UNMEASURED=8` (was: exit 2 ENV). Each exclusion is printed with its reason (7 range-phase, 2 unit=unknown).
- Session receipt: writer = `pmat work estimate record|check` (`src/cli/handlers/work_estimate_ledger.rs`), refuses a row without `unit` and writes nothing, `O_APPEND` single `write_all`, `.gitattributes merge=union`; contract `contracts/estimate-ledger-v1.yaml` 21 obligations / 21 evaluated / 0 failed; 7 mutants planted, 7 killed; required checks all SUCCESS; reader-side follow-up filed as paiml/paiml-implement#216.

Corrections to the orchestrator's brief (findings): the ledger held 30 rows, 14 of them keyed `repo=pmat` for this same repository; 18 rows lacked `unit`, not 16; only 10 of the 16 correctly-keyed rows were measured; `estimate.sh` already pooled by unit (paiml-implement PMAT-066) — the missing piece was the writer and a check in this repo. The quorum widened the one sanctioned rewrite to re-key L17–L30.

PROCESS FINDING: PR #1382 merged at 08:55:10Z with `docs/audits/quorum-PMAT-1366.json` `agreed=false` (FAIL/PASS/PASS). The dissent's only blocking claim — "no closing keyword" — was factually wrong (the PR body carried `Closes #1366`; the issue closed on merge), but the rule is three PASS, and a wrong FAIL is answered by another round, not by the author overruling it. Not reverted (the substance is verified above); every later brief now says: merge only through the quorum-gated helper, never around a non-agreed artifact.

Side effect: #1382 also carried the PMAT-1381 row to master, which makes the PMAT-1336 lifecycle PR #1383 redundant for that row (its session is still live and will meet the conflict) and unblocks D4 at once. 08:57Z relaunched D4 PMAT-1365 (pid 614362) with a fifth-session resume note: rebase, mark ready, re-run quorum with the receipt, arm the merge. Slots 3/3: PMAT-1336, PMAT-1363, PMAT-1365.

## 2026-09-17T09:34Z — three PRs in CI

tree: run-log behind=0 against origin/master e89a827f7.
- D1 PMAT-1363: LIVE, HEAD df6c351b2, pushed; PR #1364 checks 21 pass / 21 pending / 0 fail (was 14 fail at 07:38Z, 3 fail at 08:56Z).
- D4 PMAT-1365: LIVE, rebased on e89a827f7, HEAD 1f6207559; PR #1368 out of draft, 7 pass / 35 pending.
- PMAT-1336 lifecycle: LIVE, HEAD dcad45735; PR #1383 7 pass / 35 pending (rebased after #1382 carried the #1381 row).
- Slots 3/3. D0 and D2 briefs are ready under `.run/briefs/`; they launch as D1 and D4 merge.

## 2026-09-17T09:58Z — D1 MERGED (#1364, 8915fe3e6); D2 launched

tree: run-log rebased, behind=0 against origin/master 8915fe3e6.

Raw (D1 session receipt, verdict DONE): `RoadmapService::save`/`upsert_item`/`remove_item`/`replace_item_raw` write `entries/<id>.yaml` iff `docs/roadmaps/entries/` exists (the upstream gate's `[ -d docs/roadmaps/entries ]`, `~/src/.github` sovereign-ci.yml f02067e). `pmat roadmap aggregate`: aprender's 14 selftest rows ported as `case_01..case_14`; on aprender origin/main 754c48225 three runs byte-identical to each other, to the committed file and to the Python aggregator (sha256 e1a984ee…); 22/22 differential scenarios byte-equal; `--check` exits 1 naming the row after a hand edit. `work add` refuses six malformed ids (exit 1, nothing written). Lock: `<git common dir>/pmat/roadmap-id.lock` — NOT keyed on roadmap.yaml's path, so fragment writes were already covered; what was missing was proof (mutant M5, exclusive→shared, survived until the tests held a shared lock too). 10 mutants red. Contract `contracts/roadmap-fragments-v1.yaml`: 8 obligations / 8 falsification tests. Quorums: plan 3/3; review 2 FAIL → both fixed under mutation → re-review 3/3 agreed; post-rebase scope 3/3. CI 45 pass / 0 fail. #1370: quorum decided LEAVES (3/3).

Corrections to the orchestrator's/operator's brief (findings): `RoadmapServiceIo::save()` does not exist — it is `RoadmapService::save()` in `roadmap_service_io.rs`; aprender's file has 899 entries + 11 fragments, not 879; there is no `pmat work show` (it is `pmat work status <id>`); the lock-scope worry (D1 item 4) was unfounded by measurement; `pv validate` cannot print a denominator — `pv status` counts obligations (CI pins pv 0.63.0, local is 0.65.2); of aprender's five RMFR obligations, RMFR-OB-003 (`split()`) was not ported because pmat has no split command.

Session-reported hazards, acted on:
- Shared cargo target dir: the interactive `cargo` zsh function derives `CARGO_TARGET_DIR` from the origin remote's basename, so every clone shares `/mnt/nvme-raid0/targets/paiml-mcp-agent-toolkit`; one session's build overwrote another's `pmat`. Decision: not editing the operator's shell profile; every new brief mandates `CARGO_TARGET_DIR=/mnt/nvme-raid0/targets/pmat-<ticket> command cargo …`. The orchestrator's own verification build uses `/mnt/nvme-raid0/targets/pmat-orch`.
- Lifecycle cascade: each merge closes its issue and leaves its own row `planned` (a ticket cannot complete itself, CB-2113), which reds CB-2115 on every open PR until a lifecycle commit lands. #1364 had to complete PMAT-1366; now PMAT-1363's row is the orphan. The live PMAT-1336 session (PR #1383, currently "complete PMAT-1366") and D4 will both meet it.
- pmat's own repo is NOT in fragment mode (no `docs/roadmaps/entries/` on master) — opting it in is not in the operator's brief and changes what CB-2113/2115 read, so it is not done here; the previous orchestrator's plan to wait for "fragment mode" before filing D0/D2 is dropped (correction).
- `merge=union` on the estimates ledger duplicated every row during D1's rebase onto #1382; the session restored master's copy and re-appended with `pmat work estimate record`. Logged for D3's follow-up: union merge is only safe when neither side rewrote existing lines, and #1382 did (the one sanctioned backfill).

09:55Z launched D2 (clone `.wt/D2`, branch `fix/roadmap-unlocked-write-bypass` from 8915fe3e6, pid 3479979); the session files its own ticket (Phase 1). Its brief carries the D1 gap verbatim ("pmat work migrate still rewrites roadmap.yaml without the lock and without fragment support") and the merge rule from the D3 process finding. Slots 3/3: PMAT-1336, PMAT-1365, D2.

## 2026-09-17T10:12Z — D1 re-verified by the orchestrator on master 8915fe3e6

tree: pmat run-log HEAD behind=0 against origin/master 8915fe3e6; binary `/mnt/nvme-raid0/targets/pmat-orch/debug/pmat` (path taken from `cargo build --message-format json`, built from that tree); aprender origin/main a672677f4 exported with `git archive` (899 rows, 11 fragments).

Raw:
```
run1 exit=0 sha=e1a984eeee7990df bytes=628259
run2 exit=0 sha=e1a984eeee7990df bytes=628259
run3 exit=0 sha=e1a984eeee7990df bytes=628259
committed sha=e1a984eeee7990df
aggregate(aggregate(x)) sha=e1a984eeee7990df
--check: ok  docs/roadmaps/roadmap.yaml == aggregate(11 fragment(s)), idempotent   exit=0
--check after editing fragment-backed row PMAT-3205 in roadmap.yaml: FAIL … first differing row: PMAT-3205   exit=1
--check after editing a base-only row (no fragment): ok, exit=0
```
Decision: D1's contract holds by my own measurement (three runs byte-identical, equal to the committed file, idempotent; the control goes red). The last line is by design, not a hole: roadmap.yaml is its own base, so a row with no fragment has nothing to be compared against — same semantics as aprender's aggregator.

Finding (not blocking, goes to #1370's follow-up): on a READ-ONLY copy every run exits 2 — `FAIL cannot take the roadmap lock … (Failed to open lock file: "docs/roadmaps/roadmap.yaml.lock") — this box cannot judge`. A print-only `aggregate` and `--check` need a writable lock file, and outside a git repository that file is created beside the roadmap (`roadmap.yaml.lock` was left in the tree). Honest (exit 2, not a false ok), but a read-only CI mount cannot run the parity check. My first three runs were on the read-only copy and measured exactly that; the numbers above are from a writable copy.

## 2026-09-17T10:37Z — sessions in flight

tree: run-log behind=0 against origin/master 8915fe3e6.
- PR #1383:      32 pass      10 pending       4 skipping — chore(PMAT-1336): register PMAT-1385 and complete PMAT-1363 — master's two CB-2115 findings
- PR #1368:       1 fail      23 pass      18 pending       4 skipping — build(make): declare `gate` — pmat had no gate of its own, so discovery guessed a weaker one
- D2 filed its own ticket: PMAT-1385 (branch `PMAT-1385-roadmap-unlocked-write-bypass`), LIVE, no PR yet. D4 rebased onto the D1 merge (HEAD be34454e7). PMAT-1336 HEAD 7f46ba209.
- Drafted the D6 brief (`.run/briefs/PMAT-636.tmpl`, CB-200 / #1266): re-measure with one binary on both trees and publish the set difference before fixing; never raise the baseline; make a required CI job able to see the count. Queue after the next free slot: D0, then D6.
- Correction: the previous entry's heading said 10:12Z; its commit landed at ~10:11Z — headings are now generated with `date -u`, not typed.
