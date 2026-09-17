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

## 2026-09-17T11:07Z — D4 reviewed and ready but blocked by the lifecycle cascade; the #1305 flake hit the lifecycle PR; D7 launched

tree: run-log behind=0 against origin/master 8915fe3e6.

Raw:
- D4 PMAT-1365 fifth session: `PARTIAL(blocker)`, PR #1368 open, not draft, NOT armed. Quorum 3/3 PASS twice (judged heads 1f6207559, f6f0aea1d; artifact eba9388cd). `make gate` on be34454e7: 27/28 legs pass; the one red is `cb-2113-cb-2115` — ORPHAN-ROADMAP PMAT-1363 (#1363 closed 09:51:56Z by the D1 merge, row still `planned`) and ORPHAN-GITHUB #1385 (D2's new issue, opened 09:57:46Z, no row on master). Both are master's state, not the branch's. CI on eba9388cd: 30 pass, `traceability` fail. Extension point for D0/D2: ONE row in the `legs_table` heredoc of `scripts/gate.sh` below `# ── EXTENSION POINT`, format `kind | contexts | leg | source | note | command`.
- Notable: at be34454e7 the CB-200 lib test PASSED inside `make gate` (it was the one red at bfe4e7acd). The session did not investigate. That is the index-freshness flip #1266 describes — the same tree reads red or green — and is part of D6's brief.
- Lifecycle PR #1383 (now "register PMAT-1385 and complete PMAT-1363") went red on required `ci / test`, run 35209474539 job 105168799345, 10:49:42Z, on a ROADMAP-ONLY diff: `a_crate_that_does_not_compile_is_reported_as_not_measured … FAILED — not_measured must be set for an uncompilable crate; outcome was: violations=[] not_applicable=None` (21731 passed, 1 failed). That is the known flake #1305 / #1284. The PMAT-1336 session re-ran the job (pending at 11:07Z).
- Corrections D4 made to my resume note (findings): "red on CB-200 only" was wrong at both heads it measured; "#1382 resolved it upstream" was half true — #1382 landed the #1381 row and in the same merge orphaned PMAT-1366's row; the installed `pmat` on PATH (3.40.2) has no `work estimate` — only a tree-built binary does; `merge=union` on the estimates ledger re-added 31 rows on rebase (second session to be bitten; goes to D3's follow-up paiml-implement#216 context and to the release notes as a known hazard until fixed).

Decisions:
- The cascade is now the critical path: D4 cannot go green until #1383 merges, #1383 needs a re-run of a flaky required check, and a PR that fixes the flake would itself need `traceability` green — so the re-run is the only non-circular way forward and is NOT treated as a bypass (the check still has to pass on its own). But a flake is red until root-caused: 11:06Z launched D7 PMAT-1305 (clone `.wt/PMAT-1305`, pid 2608636) to fix the analyzer at the cause. Basis: three recorded occurrences, the latest on a diff that touches no Rust.
- Queue: when the PMAT-1336 session ends → a short D4 finishing session (merge master, re-quorum if the diff changed, arm). Then D6 (CB-200) and D0 as slots free. Slots 3/3: D2 (PMAT-1385), PMAT-1336, PMAT-1305.

## 2026-09-17T11:28Z — lifecycle PR #1383 MERGED (7c2aa59b8); master red again within the hour; D4 finishing session launched

tree: run-log rebased, behind=0 against origin/master 7c2aa59b8.

Raw:
- #1383 merged 11:25:11Z after the flaky `ci / test` job was re-run by its session (45 pass, 0 fail). It ended up registering PMAT-1385 (D2's issue) and completing PMAT-1363 — not the #1381 row it was launched for, which #1382 had already landed with different bytes.
- Orchestrator measurement on 7c2aa59b8: open issue #1386 (opened 10:50:15Z by a sibling session: "`pmat query --regex … --path <dir> --files-with-matches` ignores --path: 47 files returned, 0 inside the dir") has 0 roadmap rows → CB-2115 ORPHAN-GITHUB → `traceability` red on every open PR again.
- Corrections from the PMAT-1336 session to my brief (findings): the operator's row uses block-style `labels:` (same content, different bytes than I wrote); NO sanctioned writer can reproduce a byte-identical row — `work sync` mints `GH-<n>` ids with wall-clock nanosecond timestamps and `work add --github-issue` stamps run time, not the issue's `createdAt` — so "byte-identical across PRs" was an unsatisfiable instruction and I have stopped giving it; locally, CB-2113 judges against the clone's `origin/HEAD`, which in clones of the primary checkout pointed at `feat/roadmap-fragments` (new clones now run `git remote set-head origin master`).

Mechanism (why the run keeps stalling here): CB-2115 is a live bijection, so its verdict on a PR is a function of the world, not of the diff. Every issue a session files and every merge that closes an issue reds every open PR until a row lands on master, and landing a row costs one full CI cycle (~40 min) — longer than the interval between orphans with three sessions live. Open PR #1341 (PMAT-1309, stale since 2026-09-12, 2 failing checks) would tolerate YOUNG orphans inside a grace window, but by design never the closed-issue leg, which is the one each merge trips. It is not in D0–D5 and is not picked up here.

Decisions:
- No more stand-alone lifecycle sessions while three tickets are live. Each ticket PR carries master's current CB-2115 findings as ONE separate commit through a sanctioned writer, with the quorum asked to rule on scope and shown the precedent (#1364: SCOPE ACCEPT 3/3). Basis: measured cycle time above. Sessions are told not to file new issues in this repo unless the same PR carries the row.
- 11:28Z launched the D4 finishing session (pid 2768639): merge master, carry the #1386 row, re-quorum, arm through `pmat-merge`; one named re-run allowed for the #1305 flake only. Slots 3/3: D2 (PMAT-1385), D7 (PMAT-1305), D4.

## 2026-09-17T11:50Z — CORRECTION: carrying lifecycle rows inside ticket PRs does not pass quorum; stand-alone lifecycle PRs are back

tree: run-log behind=0 against origin/master 7c2aa59b8.

Raw (D4 finishing session, ~30 turns, `PARTIAL(blocker)`): merged master cleanly (behind=0, HEAD 0522fb62c pushed, not armed). Added the #1386 row with `pmat work add --github-issue 1386` in its own commit → CB-2115 115/115 green locally. Quorum on ea3edddc2: FAIL/PASS/PASS — lane 1 (gemini-3.1-pro-high) FAILed the row as "an unrelated roadmap item" with the receipt section and the #1364 precedent in front of it. Per the brief the row was reverted (7fbce2a6d) and not argued a third time. `make gate` on 9dc60d624: 27/28; the one red is `lib-tests` 21734/21735 — the #1305 flake, locally this time, not re-run.

Correction to my own 11:28Z decision (finding): "each ticket PR carries master's CB-2115 rows, the quorum will accept it given the precedent" is refuted by measurement — the same lane has now failed it three times on #1368 (once without, twice with the explanation), while #1364's lanes accepted it. Lane verdicts on scope are not stable across PRs; a plan that depends on them is not a plan. Stand-alone lifecycle PRs under PMAT-1336 are the only shape that has passed every time (#1375, #1377, #1380, #1383), so that is the rule again, batched: each one folds in every finding live at arm time. Cost accepted: ~40 min CI per lifecycle PR plus ~40 min for the ticket PR behind it, per merge, because branch protection is strict.

New finding from the D4 session, in D4's own code (not yet fixed, goes into the next D4 session): `scripts/gate.sh` legs `cb-2113-cb-2115` and `pmat-score` call `./target/debug/pmat` and ignore `CARGO_TARGET_DIR`; under the build isolation my briefs mandate they ran a stale binary from 09:30Z, so those two legs' PASS was not evidence. A gate that runs whatever binary happens to be lying in `./target` is the "never hand-write a binary path" trap from memory; the fix is to take the path from `cargo build --message-format json`.

11:50Z launched lifecycle session 3 (clone `.wt/PMAT-1336`, branch `PMAT-1336-lifecycle-3`, pid 1435689): register #1386 plus whatever else the bijection reports at arm time. Slots 3/3: D2, D7, PMAT-1336. D4 waits for it (needs: merge master, fix the gate.sh binary path RED→GREEN, one more quorum round, arm).

## 2026-09-17T12:16Z — sessions in flight

tree: run-log behind=0 against origin/master 7c2aa59b8.
- PR #1387 (chore(PMAT-1336): register PMAT-1386 — master's one CB-2115 finding):      27 pass      15 pending       4 skipping 
- PR #1388 (fix(PMAT-1305): dead-code analyzer builds into a target dir only its workspace root uses —):       2 fail      28 pass      13 pending       4 skipping 
- PR #1368 (build(make): declare `gate` — pmat had no gate of its own, so discovery guessed a weaker o):       2 fail      41 pass       1 pending       5 skipping 
- D7 PMAT-1305 opened #1388 after ~65 min: "dead-code analyzer builds into a target dir only its workspace r…" — i.e. the mechanism it found is a SHARED cargo target dir, the same class as the cross-clone hazard logged at 09:58Z. Receipt pending.
- D2 PMAT-1385: LIVE, HEAD dbf8774ec, no PR yet. Lifecycle-3: PR #1387 open. Slots 3/3.

## 2026-09-17T12:37Z — lifecycle PR #1387 MERGED (7fa1be27d); D4 seventh session launched

tree: run-log rebased, behind=0 against origin/master 7fa1be27d.

Raw: #1387 armed through `pmat-merge` on a 3/3 PASS artifact (judged 81fe249fa), 45 pass / 0 fail, merged 12:3xZ; session reports CB-2115 on master = 115 items / 115 issues / 0 findings. Cycle time launch→merge: 46 min. Note: `kind-gate.sh` and `model-gate.sh` both exit 2 on `kind:lifecycle` — the skill has no such kind; the session recorded it rather than working around it (goes to paiml-implement, not fixed here).

Decision: 12:36Z launched D4 session seven (pid 3325886): merge master, fix its own gate.sh stale-binary finding RED→GREEN, full `make gate`, one quorum round, arm via `pmat-merge`; told NOT to carry lifecycle rows and to stop if a new orphan appears. Slots 3/3: D2 (PMAT-1385), D7 (PMAT-1305, PR #1388 in CI), D4.

## 2026-09-17T13:12Z — sessions in flight

tree: run-log behind=0 against origin/master 7fa1be27d.
- PR #1388:      26 pass      16 pending       4 skipping 
- PR #1368:      15 pass      27 pending       4 skipping 
- D2 PMAT-1385: LIVE at 287746ab0, last transcript event 12:55Z (a long-running command or a delegate is in flight; pid alive). D7 at 34d347c69, D4 at c5f254f70. Slots 3/3. Queue: D0, D6.

## 2026-09-17T13:31Z — D7 MERGED (#1388, ce945d81e): the #1305 "flake" was a shared cargo target dir in CI; lifecycle-4 launched

tree: run-log rebased, behind=0 against origin/master ce945d81e.

Raw (D7 session receipt, verdict DONE, quorum agreed=true 3/3 on ae8efee2f, merged 13:27:53Z, 45 pass / 0 fail):
- Mechanism: cargo fingerprints a workspace member by its path relative to the workspace root; the dead-code analyzer inherited `CARGO_TARGET_DIR=/workspace/target`, which `ci / test` and `ci / coverage` mount per PR, so coverage's COMPILABLE `fx` fixture made test's UNCOMPILABLE `fx` fixture look fresh and `cargo check` exited 0 — a clean full measurement of a crate that cannot compile. Not a test flake: a product defect (a gate reporting on what it could not see).
- Fix: the analyzer builds into `<target_directory>/pmat-dead-code/<name>-<fnv1a64(canonical workspace root)>`.
- Reproduction (harness `scripts/repro-pmat-1305-shared-target-race.sh`, N=40 per mode): before 17/80 aligned, 17/80 free, 0/40 single; after 0/80, 0/80, 0/40; under the mutant 19/80. Planted test red 3/3 before, green after, red under the mutant. Contract `contracts/dead-code-target-isolation-v1.yaml`: 5 obligations + 6 falsification tests, all evaluated, 0 failed.
- All four hypotheses in my brief were REFUTED by measurement (cache hit across fixtures, shared `.pmat` dir, `set_current_dir` race, TMPDIR); the cause was not on the list. Findings, not failures.
- #1284 closed as duplicate through `mutate.sh close` on a 3/3 scope vote. Rows now awaiting a lifecycle commit: PMAT-1305 and PMAT-708 (#1284).
- Session findings worth keeping: F-5 a `quorum-review.sh` review lane ran in the SHARED checkout and applied/reverted the mutant patch there (lane-confinement breach, same class as memory's escape #5 — belongs to paiml-implement/quorum-review, not fixed here); F-6 `pmat-merge` hashes against the literal local base ref while `quorum-review.sh` uses `origin/<base>` — a stale local `master` disarms a good PR (new clones now fast-forward local master; told to every later session); F-9 some lib test writes a 15 GB `target/` into the repo root despite an inherited `CARGO_TARGET_DIR` (disk hazard with several clones live — `df` checked below).

Same class, three times today: one target dir shared by things that believed they were alone — the operator's `cargo` shell function across clones (09:58Z), `scripts/gate.sh`'s hard-coded `./target/debug/pmat` (11:50Z), and now CI's per-PR mount across jobs.

Decision: #1388's merge orphans PMAT-1305 (and PMAT-708) under CB-2115, and PR #1368 — armed for auto-merge since ~13:25Z, 40 pass / 4 pending — is strict-protected, so it must take master and will go red on `traceability` until those rows are terminal. 13:30Z launched lifecycle-4 (pid 3360656) to complete both and anything else live. Slots 3/3: D2, D4, lifecycle-4. disk: 320G free (82% used).

## 2026-09-17T14:05Z — sessions in flight

tree: run-log behind=0 against origin/master ce945d81e.
- PR #1390 (chore(PMAT-1336): complete PMAT-1305 and PMAT-708 — master's two CB-21… armed=true):      25 pass      17 pending       4 skipping 
- PR #1368 (build(make): declare `gate` — pmat had no gate of its own, so discover… armed=true):       3 pass       3 pending       3 skipping 
- PR #1389 (fix(PMAT-1385): pmat work migrate writes under the repository lock; ev… armed=false):       1 fail      27 pass      14 pending       4 skipping 
- Lifecycle-4 PR #1390 armed; D4 PR #1368 armed (waits on #1390 for `traceability`); D2 opened PR #1389. Slots 3/3. Queue: D0, D6.

## 2026-09-17T14:06Z — D4 armed on a 3/3 verdict; D6 launched

tree: run-log behind=0 against origin/master ce945d81e.

Raw (D4 session seven, 68 turns): PR #1368 armed via `pmat-merge` on head b13eb5dc6 with a committed 3/3 PASS verdict (three rounds, all 3/3, because master moved twice). Stale-binary fix RED 48737ee01 → GREEN e7812f9a9; the finding was WIDER than the session's own earlier report — 10 legs ran `./target/debug/pmat`, not 2. `scripts/gate.sh` now resolves `$PMAT_BIN` once from `cargo build --locked --bin pmat --message-format json`, and a leg FAILS if cargo reports no binary (no fallback to `./target`). `make gate` on e94b2c6cc: 27 PASS / 1 FAIL (`lib-tests`: #1305 — since fixed on master — and CB-200). Blocked only by `traceability`: ORPHAN-ROADMAP PMAT-1305 and PMAT-708, which lifecycle-4 (PR #1390, armed, 25 pass / 17 pending) completes.

Decisions:
- When #1390 merges, strict protection leaves #1368 one merge behind. Taking master into an armed branch is orchestration (a pure cascade: the reviewed diff is unchanged, which `pmat-merge` checks by hash), so I will do it myself with `gh pr update-branch 1368` rather than spend a session; if the helper disarms, a session re-runs the round.
- 14:06Z launched D6 PMAT-636 / #1266 (clone `.wt/PMAT-636`, pid 1468871): same-binary cold-index measurement on both trees and the set difference first; never raise the baseline; make a required CI job able to see the count. Slots 3/3: D2 (PR #1389), lifecycle-4 (PR #1390), D6. Queue: D0 (needs #1368's extension point on master).

## 2026-09-17T14:24Z — lifecycle-4 MERGED (#1390, 85b798e0d); #1368 cascaded by the orchestrator

tree: run-log rebased, behind=0 against origin/master 85b798e0d.

Raw: #1390 merged 14:23:18Z (launch→merge 53 min). `gh pr update-branch 1368` → "PR branch updated", new head 85cac5210, `autoMergeRequest` still set (armed=true), mergeStateStatus BLOCKED while CI runs. The cascade adds only master's roadmap/audit changes; the reviewed diff is unchanged. If a rendered ledger drifts on this head, CI's non-required ledger checks will say so and a session re-renders.

## 2026-09-17T14:32Z — D0 launched, with the ticket filed issue-last

tree: run-log behind=0 against origin/master 85b798e0d.

Raw: #1368 (head 85cac5210) `traceability` = pass at 14:31:20Z; still armed. 14:31Z launched D0 (clone `.wt/D0`, branch `feat/issue-closure-contract` from 85b798e0d, pid 2735158).

Decision: I held the free slot for 6 minutes until #1368's `traceability` job had passed, because a new issue opened in that window would have red it again. D0's brief inverts the usual order for the same reason — roadmap row first, GitHub issue created only when the PR is otherwise green and reviewed, just before its final CI run — so the orphan window CB-2115 opens on the sibling PRs (#1389, D6) is one CI cycle instead of the whole implementation. Basis: three cascades measured today (#1381, #1385/#1386, #1305/#1284), each costing 45–55 min. Slots 3/3: D2 (PR #1389), D6 (PMAT-636), D0.

## 2026-09-17T15:05Z — D4 MERGED (#1368, ef2a0b947) and re-verified; orchestrator-run lifecycle PR #1392

tree: run-log rebased, behind=0 against origin/master ef2a0b947.

Raw (my own rerun on master ef2a0b947): `discover.sh` exit 0 → `{"gate_cmd":"make gate","gate_cmd_fallback":"false","required_check":"ci / gate,docs build (docs.rs environment),feature-gate,pmat score,provable ladder"}` (was `cargo test --workspace`, fallback true). `bash scripts/gate.sh --list` → "44 rows, 28 run here, 16 CI-only; every required context has a row"; the CI-only rows are printed by name (coverage, provenance, roadmap-fragment-parity, tests-dont-write, windows-check, docs-rs-build, bundles, individual, feature-tests, package-size, binary-size, differential-corpus, flag-efficacy, cli-doc-sync-falsifier, comply-ladder, mutation-diff). My `gh pr update-branch` cascade held: the PR stayed armed and merged at 14:57:09Z with no further session.

Correction to the operator's brief (from the D4 sessions, confirmed above): `mutation-diff` is not a required check; `provable ladder` is; `feature-gate` and `pmat score` mostly run locally.

Lifecycle, done by the orchestrator itself (decision + basis): the merge left `PMAT-1365` planned → `pmat work sync --check-only`: "open items 113 · open issues 112 … ORPHAN-ROADMAP PMAT-1365: #1365 is closed". All three slots were busy (D2, D6, D0) and D2's PR #1389 cannot go green without this row, so waiting for a slot would have idled D2 for a CI cycle. I ran the sanctioned writer (`pmat work sync --direction github-to-yaml`: one `close-item`, a 2-line diff, then 112/112 coherent), appended the receipt section, pushed `PMAT-1336-lifecycle-5`, opened PR #1392, and started `quorum-review.sh --author-model claude-fable-5-1` (three gemini lanes; none in the author's family). It is armed only through `pmat-merge` on a 3/3 artifact, like every other PR today. This is not a fourth session: no Claude session was spawned; slots stay 3/3.

## 2026-09-17T15:12Z — #1392 armed on 3/3; D0 session one hit andon before writing code; relaunched with a measured budget

tree: run-log behind=0 against origin/master ef2a0b947.

Raw:
- Lifecycle PR #1392: `quorum-review.sh` → "AGREED: lane 1=PASS, lane 2=PASS, lane 3=PASS" (gemini-3.1-pro-high, gemini-3.8-flash-high, gemini-3.7-flash-high; author claude-fable-5-1, judged head 6d4132666). Artifact committed (cb0872a99); `pmat-merge 1392 --auto --merge` → "quorum verdict … agrees … arming auto-merge". armed=true at 15:12Z.
- D0 session one: `PARTIAL(andon)` at k_measured=55 ≥ 0.8K=56 (K̂=35 from the now-working ledger, K=70) with no code written. It filed ticket PMAT-900001 (`pmat work add --id`; a synthetic id because `work add` refuses without `--github-issue` or `--id` (#1240) and CB-2112 ties the id's number to the issue's), opened draft PR #1391, and ran a width-3 plan grill (FAIL/FAIL/PASS, each FAIL resolved with a fix or evidence, D1–D10 in its receipt).
- Its corrections to my brief (findings): "no custom closer exists" holds for CALL SITES, but pmat tells users to close by hand — `pmat work complete` prints "Next: gh issue close N" and `prompts/github-ticket.yaml` teaches a `fixes #N` subject and a manual close; there are THREE commit-msg hook writers, not one; a gate cannot be purely `pmat query`-based (it indexes function bodies only — hook templates in consts, scripts, YAML and prompts are invisible to it); `pmat work edit` cannot write notes; `pmat work complete` commits with `--no-verify`, so no commit-msg lint ever sees that commit; `traceability` is required only through the org ruleset's top-level `gate` job; `pmat query --exclude-tests` still returns functions from `tests.rs` files.

Decision: the D3 fix did what it was for — the estimate is now a number — and the number is wrong for this ticket: K̂=35 is a median over single-deliverable tickets. Relaunched D0 at 15:12Z (pid 419660) with `--budget-turns 220`, basis = today's measured sessions on this repo (107, 179, 231, 104+67+68), stated in the brief; it starts at the code phases from its own receipt, and #1368 being on master means its gate row can land in the same PR. Slots 3/3: D2, D6, D0.

## 2026-09-17T15:41Z — STOP-THE-LINE (logged): the host hard-crashed at 15:30Z, killing all three sessions and the CI runners; relaunched

tree: run-log behind=0 against origin/master ef2a0b947.

Raw: `uptime` at 15:40Z = "up 8 min"; `last -x reboot` shows boots at Sep 16 20:26, 20:45, 22:08 and Sep 17 17:31 local — four in 24 h; `journalctl -b -1` ends at 17:30:04 local mid-activity with no shutdown sequence (an unclean crash or power loss, not a reboot anyone issued); no OOM line survived. All three `claude -p` sessions (D0, D2, D6) died with 0-byte logs. The self-hosted runners live on this same machine: `ci / test` and `ci / coverage` on #1392 and #1389 failed with "The self-hosted runner lost communication with the server" (runner intel-clean-room-14, 15:18:41Z→15:39:43Z, no steps, no log blob) — infrastructure, not a verdict; the runs were still in progress so a re-run was refused for now. The last line before the crash was `fleet-runner-liveness: total=28 online=27 … UNHEALTHY`.

What survived: every clone, commit and isolated target dir. D0: HEAD 75a9a28bd, 42 commits unpushed, 2 files uncommitted. D2: 46a49e0f5, clean, pushed, PR #1389 (real reds besides the runner loss: `ci / lint`, `traceability`). D6: dd56a0cb5, 1 unpushed commit, measurement artefacts on disk under `/mnt/nvme-raid0/targets/pmat-636*`.

Decision + basis: cause unknown; load is a suspect (three sessions building and running full suites beside 21 clean-room runners on one box; yesterday's three crashes fall in the previous orchestrator's run as well). Relaunched all three at 15:41Z (pids 35673, 35685, 35696) with a crash-resume note and a load rule — one heavy job at a time per session, `CARGO_BUILD_JOBS=8`, never a full-suite pair — and "push early". This does not fix the machine; it is reported to the operator as the top risk to the release, because clean-room and `cargo publish` run on this host too.

## 2026-09-17T16:12Z — lifecycle PR #1392 MERGED; three sessions live after the crash

tree: run-log rebased, behind=0 against origin/master f25d7f1cc.

Raw: at 15:44:11Z `gh run rerun 35238657820 --failed` (the one re-run, for the runner-lost `ci / test` + `ci / coverage` jobs; `ci / gate` and `gate` were red only as their aggregates). Result 45 pass / 0 fail; auto-merge fired; master = f25d7f1cc. PMAT-1365's row is terminal; master's bijection was coherent at arm time.
- D0 HEAD 7b1fc68f1 (pushing as it goes), D2 HEAD 77bdbf388, D6 HEAD 35da3cc6d — all LIVE, host up 40 min, load average ≤ 10 since the relaunch (48 cores).
