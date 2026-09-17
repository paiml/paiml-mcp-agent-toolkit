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
