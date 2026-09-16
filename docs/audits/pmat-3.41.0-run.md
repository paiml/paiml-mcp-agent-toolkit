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
