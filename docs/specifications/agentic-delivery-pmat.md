# Agentic Delivery — pmat, paiml-implement and agy against the whiteboard

**Status:** specification, ticket PMAT-650 · **Audited at:** master `cd6f796d6` (2026-09-03), pmat 3.35.0-unpublished, `~/src/paiml-implement` `c8e2ced` (bundle v1.1.0, spec AUTO-IMPL-SKILL-001 v1.0.1), agy 1.1.25 · **Source:** the 2026-09-03 whiteboard "Agentic Delivery Architecture" (Claude Code + Fable orchestrator · quorum-reviewed sub-agents · Google Antigravity execution pool · pmat quality gates · auto-merge to GitHub and crates.io).

## 0. Provenance

Every status in this document was **measured** in one session by the orchestrator with the command beside it; nothing is quoted from a design document. The plan behind it (`~/.claude/plans/immutable-orbiting-moore.md`) was reviewed by one agy `/teamwork-preview` lane (conversation `0a2b5862-3ea1-4533-9b9a-758f33be4b00`, 245 s, two child conversations). The lane returned `do-not-implement-as-written` with six findings; five are applied here (§6 quorum row, §8.1 control, §9 order, §7 columns, §8 trailer). The sixth — that `~/.claude/hooks/subagent-lock.sh` and `~/src/paiml-implement/agents/*.md` do not exist — was refuted by `test -e` on all four paths; a sandboxed lane does not see the host filesystem, which is itself a fact about lane grounding this document records in §6.4.

House rule, as in `docs/specifications/pmat-architecture-crux-audit.md`: a gate that cannot fail is theater; a status of PRESENT is a claim with a command that would have returned a different answer if it were false.

## 1. Executive summary

- The pipeline's two **"must"** arrows do not exist. Nothing produces a quorum verdict that gates a pull request (no such skill is installed; the delegate's quorum lane emits verdicts nobody consumes), and nothing dog-foods the **published** crate: the pre-publish probe (`crate-release-dogfood`) reads the packaged tarball, and no step ever installs the version crates.io serves.
- The incident that proves the second hole: **3.35.0 was merged on 2026-09-02 (#1108) and never tagged, released or published.** crates.io's newest version is 3.34.0 (2026-08-29); `git tag` stops at `v3.34.0`; docs.rs has no 3.35.0. Every gate was green throughout, because no gate asks "did the merged release PR produce a release".
- Of the five quantitative rules in the quality zone, `pmat quality-gate` enforces **one** (complexity). Lint lives only in `pmat verify`; churn is measured (`analyze churn`) and gated nowhere; lines-per-file exists only as a `pmat work` contract claim (`max_file_lines`, default 500); the ticket-in-commit check in the generated hook prints a warning and exits 0 (#1126).
- "Every sub-agent must comply" is not a contract today: the worker returns a receipt with an `acceptance` exit code and no gate result; the orchestrator re-runs the acceptance command but nothing obliges the worker to have run `pmat verify`.
- The orchestration zone's "1–3 sub-agents" is implemented as a **concurrency cap** (`CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS=3`), not as a **vote**. Fan-out to Antigravity works (`/teamwork-preview` runs headless, 2–12 min); the Goal and Grill-me modes do not exist in agy 1.1.25; the executor is not swappable; the pool is capped at 10, not 20.
- Ticket ↔ work linking has no machine-checkable record: branch names carry ticket ids by convention, PR bodies by habit, commits by nothing.

Twelve of the diagram's twenty-one capabilities are PRESENT or PARTIAL with evidence; nine are MISSING. §9 ranks the ten changes that close them, and §7 goes beyond the diagram: every enforcement is placed at the moment an agent acts, not only at CI.

## 2. Doctrine

1. **A gate must be able to fail**, and every gate here carries the command that shows it failing on a planted defect.
2. **Enforce at the point of action, then again at CI.** A hook that refuses the commit is worth more than a CI leg that fails an hour later; the CI leg exists so a bypassed hook is still caught.
3. **A sub-agent's claim is a claim.** Receipts carry the gate's own JSON; the orchestrator re-runs it; disagreement is a finding.
4. **Nothing merges without a quorum verdict; nothing ships without dog-fooding the published bytes.** The diagram's two "must" arrows, as gates.
5. **One orchestrator, a small quorum for judgment, a wide pool for throughput.** Claude turns are the scarce budget; agy lanes are the cheap one.
6. **Traceability is a trailer, not a habit.** `Pmat-Ticket: PMAT-NNN` on every commit; everything else is derived.

## 3. Zone 1 — Delivery pipeline

> Quorum Review Skill (3 reviewers must agree) → review verdict → Pull Request (gates merge) → Build Server (CI runs the gates) → GitHub Repo (auto-merge on green) → crates.io (publish, must pass Dog Food Skill). *Nothing merges without a quorum verdict; nothing ships without dog-fooding the published crate.*

### 3.1 Quorum Review Skill — MISSING

**Evidence.** `ls ~/.claude/skills` → `course-marketing-image-uploads crate-release-dogfood dogfood edx-publish generate-narration narrate-animation nextdns-toggle nightly-ux-crux paiml-implement post-course-asset-roundup quiz-audit-fix render-audit` — no review or quorum skill. What exists: `~/src/paiml-implement/agents/paiml-agy-delegate.md` runs a `quorum` lane (N `agy -p` runs, cap 10) whose output is validated against `agy/quorum-schema.json` (`verdict ∈ {PASS, FAIL, do-not-implement-as-written}`, `summary`, `findings`). Nothing consumes those verdicts to gate a PR; `gh pr merge --auto` is issued by the orchestrator on its own judgment.

**Acceptance test (AD-04).**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
# a diff with a planted contradiction: the test asserts the opposite of the ticket
git switch -c qr-fixture && printf '#[test]\nfn planted(){ assert_eq!(1+1, 3); }\n' >> src/lib.rs && git commit -qam 'planted' -m 'Pmat-Ticket: PMAT-0'
quorum-review --base master --ticket PMAT-0 --out /tmp/qr.json || true
jq -e '[.lanes[].verdict] | index("FAIL") != null' /tmp/qr.json || fail "no lane failed a planted contradiction"
# control: the same skill on a clean diff must reach 3 PASS
git checkout -q -- src/lib.rs; git commit -qam 'clean' -m 'Pmat-Ticket: PMAT-0' --allow-empty
quorum-review --base master --ticket PMAT-0 --out /tmp/qr2.json
jq -e '[.lanes[].verdict] | length == 3 and all(. == "PASS")' /tmp/qr2.json || fail "clean diff did not reach 3 PASS"
# the merge helper refuses without the artifact
rm /tmp/qr2.json; pmat-merge --auto 2>&1 | grep -q 'no quorum verdict' || fail "auto-merge proceeded without a verdict"
```
*Anti-vacuity:* the planted contradiction must be **found**, not just "any FAIL" — the finding's `file:line` must point at the planted test; the control must be 3 PASS, not "no FAIL".

### 3.2 Pull Request — gates merge — PRESENT

**Evidence.** `gh api repos/paiml/paiml-mcp-agent-toolkit/branches/master/protection` → `strict=true`, required contexts `ci / gate`, `feature-gate`, `docs build (docs.rs environment)`, `pmat score`, `provable ladder`; `required_approving_review_count=0`; `enforce_admins=false`. Strict means every merge puts every other PR BEHIND, which is why the cascade routine in §7 exists.

**Acceptance.** `gh pr merge <n> --merge` on a PR with a red required check → `GraphQL: … Required status check … is expected` (refused). Control: green PR merges. Verified 2026-09-03 on #1163/#1164.

### 3.3 Build Server — CI runs the gates — PARTIAL

**Evidence.** `.github/workflows/ci.yml` calls `paiml/.github/.github/workflows/sovereign-ci.yml@main` (lint, test, coverage, security, provenance), then `gate` aggregates `ci`, `windows-check`, `reusable-pin-drift`; `quality-gate.yml` runs `pmat score` and `provable ladder`; `feature-matrix.yml` runs the feature bundles and, since #1157, `reachability-ledger`. **Not run in CI:** `pmat verify` and `pmat quality-gate --checks all` — the gate set agents are told to pass before committing is not the gate set the build server runs.

**Acceptance test (part of AD-05).** A CI leg `pmat quality-gate --checks all --format json` on the merge commit; `.results.passed == true` required; control: a planted 501-line file fails it.

### 3.4 GitHub Repo — auto-merge on green — PRESENT

**Evidence.** `gh api repos/paiml/paiml-mcp-agent-toolkit --jq .allow_auto_merge` → `true`; `gh pr merge <n> --auto --merge` used for #1158…#1167. **Caveat measured 2026-09-03:** pushes to a PR branch created **no** workflow runs on three occasions (`actions/runs?head_sha=` → 0) while `workflow_dispatch` on the same head did (PMAT-646); auto-merge therefore waited on a check that would never appear until the runs were dispatched by hand.

### 3.5 crates.io publish — MANUAL, UNVERIFIED

**Evidence.** `Makefile:1363` `env -u CARGO_REGISTRY_TOKEN cargo publish --package pmat` behind an interactive prompt; `docs/release-process.md` still describes `server/Cargo.toml` and `cargo set-version`, a layout deleted in January. `git tag --sort=-creatordate | head -1` → `v3.34.0`; `gh release list` → `v3.34.0` latest; `curl https://crates.io/api/v1/crates/pmat` → `max_stable_version 3.34.0`; `Cargo.toml` → `3.35.0`; `#1108` merged 2026-09-02. **No job on `master` compares these four.**

**Acceptance test (AD-01).**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
v=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
latest_tag=$(git tag -l 'v*' --sort=-v:refname | head -1 | sed 's/^v//')
[ "$v" = "$latest_tag" ] && { echo "at a tag"; exit 0; }
git tag -l "v$v" | grep -q . || fail "release-check: Cargo.toml says $v but no tag v$v"
gh release view "v$v" >/dev/null 2>&1 || fail "release-check: no GitHub release v$v"
curl -s "https://crates.io/api/v1/crates/pmat/$v" -H 'User-Agent: pmat-release-check' | jq -e '.version.num == "'"$v"'"' >/dev/null || fail "release-check: $v is not on crates.io"
```
*Anti-vacuity:* the version is read from `Cargo.toml`, never a literal; **control:** a fixture with `Cargo.toml` at 9.9.9 must fail all three legs — a check that hardcodes `3.35.0` fails the control. On master at `cd6f796d6` this test fails at the tag leg naming 3.35.0.

### 3.6 must pass Dog Food Skill — PARTIAL

**Evidence.** `~/.claude/skills/crate-release-dogfood/probe.sh` probes the **packaged tarball** (P1 provenance … P7 registry) and its self-test fires 5 findings on `fixtures/defective-crate` (run 2026-09-03: `SELF-TEST PASSED: 5 probe(s) fired`). The fleet protocol's canonical runner is a shim (`~/.claude/skills/dogfood/pmat-dogfood-runner.sh`) over `<aprender>/.claude/skills/apr-dogfood`; `scripts/dogfood-use.sh` runs the built binary against independent oracles (13 checks). **None of them installs the version crates.io serves and runs it.** The diagram's arrow is from *crates.io* to the skill: the bytes a stranger receives.

**Acceptance test (AD-02).**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
v=$1; root=$(mktemp -d)
cargo install pmat --version "$v" --root "$root" --locked >/dev/null 2>&1 || fail "install $v from crates.io failed"
"$root/bin/pmat" --version | grep -q "pmat $v" || fail "installed binary does not report $v"
BIN="$root/bin/pmat" bash scripts/dogfood-use.sh || fail "dogfood-use against the installed $v failed"
bash ~/.claude/skills/crate-release-dogfood/probe.sh --published "$v" || fail "probe P5–P7 against the published $v failed"
```
*Anti-vacuity:* **control:** a fabricated version (`9.9.9`) must fail at the install leg with exit 1, and the receipt `docs/audits/release-<v>-dogfood.md` must name the installed binary's `commit:` line, which the local build cannot fake.

## 4. Zone 2 — Quality enforcement (pmat MCP)

> pmat MCP (quality server) → Sub-Agent Level (every agent must comply) → Enforce quantitative quality — hard thresholds, not opinions: Max complexity · Lint · Churn · Lines per file · Traceability. *Gates are evaluated by the Build Server and by every sub-agent before it reports back.*

### 4.1 pmat MCP quality server — PRESENT

**Evidence.** `tools/list` over stdio (protocol 2024-11-05) → 20 tools: 6 core analyzers, 3 forensic analyzers, `quality_gate`, `quality_check_content` (+ its one-release alias `quality_proxy`), `pdmt_deterministic_todos`, `git_operation`, `generate_context`, `scaffold_project`, 4 agent-context tools (`docs/mcp/TOOLS.md`, pinned by `manifest_matches_server`). `quality_gate` runs the same `run_gate_suite` the CLI runs (`src/cli/analysis_utilities/quality_gate_suite.rs`), so the two surfaces cannot disagree on findings.

### 4.2 Sub-Agent Level — every agent must comply — PARTIAL

**Evidence.** `~/src/paiml-implement/agents/paiml-impl-worker.md` §receipt: `{"ticket","phase","files_changed","commands":[{"cmd","exit"}],"tests_added","acceptance":{"cmd","exit"},"open_questions","partial"}` — no gate field; the worker "runs acceptance_cmd before returning"; `pmat verify` is named in the brief as `gate_cmd` but nothing requires it to have run. The orchestrator re-runs `acceptance_cmd` (SKILL.md Phase 2 step 3). `~/.claude/hooks/subagent-lock.sh` denies `Edit`/`Write` without an active ticket (§2b) and `git push`/`gh pr` while a worker holds a slot (§2c) — enforcement at the point of action, for two of the moments in §7.

**Acceptance test (AD-06).** A worker receipt without `gate` is treated as `partial=true` by the orchestrator (SKILL.md §6.2); a receipt whose `gate.ok` differs from the orchestrator's own `pmat verify --format json` on the same tree is recorded as a finding in the dispatch ledger. Control: a receipt with `gate.ok=true` that the rerun confirms is accepted.

### 4.3 Max complexity — PRESENT

**Evidence.** `pmat quality-gate --checks complexity` (`--max-complexity-p99`, default 50 — `GATE_DEFAULT_MAX_COMPLEXITY_P99`); `pmat verify` measures **changed files** at `--max-cyclomatic 30 --max-cognitive 25 --fail-on-violation` (`src/cli/verify.rs:456-458`). Two thresholds under one name; verify's is the one agents meet. Measured this week: verify stopped three tickets on pre-existing debt in touched files (CRUX-10 `proxy_operation` 15/34; CRUX-04 six functions 27–33; #1159 four functions 27–39), each refactored under the ceiling — the gate can fail and did.

### 4.4 Lint, zero warnings — PARTIAL

**Evidence.** `pmat verify` stage `clippy` runs `cargo clippy --all-targets -- -D warnings` (matches `ci / lint`, memory v3.26.0). `pmat quality-gate --checks` has no `lint` value (`dead-code, complexity, coverage, sections, provability, satd, entropy, security, duplicates, all`); the MCP `quality_gate` therefore cannot report a warning. The `analyze clippy` command is preview-only since 3.33.0.

**Acceptance test (AD-05).** `pmat quality-gate --checks lint --format json -p <crate with one clippy warning>` → `.results.lint_violations >= 1`, `.results.passed == false`; control: the same crate with the warning fixed → 0, passed.

### 4.5 Churn — MISSING as a gate

**Evidence.** `pmat analyze churn --days N` measures commits per file and a churn score; `quality-gate` has no churn check (see 4.4's list); `analyze churn --help` exposes no threshold or `--fail-on` flag. `.pmat-metrics.toml` has no churn key. Nothing can fail on churn.

**Acceptance test (AD-05).** `pmat quality-gate --checks churn --format json -p <repo>` with `pmat.toml [quality] max_churn_commits_90d = 5` → a file with 6 commits in 90 days is a `churn` violation naming the file and count; control: threshold 100 → 0 violations. Fixture: a temp git repo with one file committed 6 times (`git_seal` pattern from `scripts/dogfood-use.sh`).

### 4.6 Lines per file — PARTIAL

**Evidence.** `src/cli/handlers/work_contract_profile.rs:336` `max_file_lines … unwrap_or(500)` feeds `pmat work`'s contract claims (`universal_claims`, `rust_claims`); `pmat quality-gate` has no file-size check; nothing refuses a commit that grows a file past the cap. This repository's own `src/cli/commands/commands_enum/definition.rs` is 1,822 lines (#1118).

**Acceptance test (AD-05).** `pmat quality-gate --checks file-size --format json -p <crate with a 501-line file>` → `.results.file_size_violations == 1` naming the file and its line count; control: 500 lines → 0. The threshold is read from `pmat.toml [quality] max_file_lines` (the same key `pmat work` reads), default 500.

### 4.7 Traceability — link work to ticket + commit message — PARTIAL

**Evidence.** The generated pre-commit hook (`src/cli/handlers/hooks_command_handlers/hook_generation.rs:396-401`) prints `Warning: Commit message should contain task ID matching $PMAT_TASK_ID_PATTERN` and reaches `echo "✅ All quality gates passed!"` regardless (#1126, unimplemented at HEAD). `pmat work` offers `add list edit delete annotate start continue checkpoint complete delegate falsify cot ledger event claim triage status sync init validate migrate list-statuses score codebase-score` — no verb links a commit, branch or PR to a ticket. `subagent-lock.sh` §2b refuses an edit without `$STATE/active-ticket`, but only while the skill's state directory exists.

**Acceptance test (AD-03, AD-07).**
```sh
set -euo pipefail; fail(){ echo "FAIL: $*"; exit 1; }
pmat hooks install --strict >/dev/null
printf 'x\n' >> README.md
git commit -qam 'no trailer' && fail "a commit without Pmat-Ticket was accepted"
git commit -qam 'with trailer' -m 'Pmat-Ticket: PMAT-650' || fail "a commit with the trailer was refused"
git log -1 --format='%(trailers:key=Pmat-Ticket,valueonly)' | grep -qx 'PMAT-650' || fail "trailer not readable by git"
pmat work link PMAT-650 --commit HEAD && pmat work annotate PMAT-650 | grep -q "$(git rev-parse --short HEAD)" || fail "link not recorded"
pmat comply check --checks CB-TRACE --format json | jq -e '.checks[] | select(.name|test("ticket trailer")) | .status == "pass"' >/dev/null || fail "comply did not see the trailer"
```
*Anti-vacuity:* the first commit must be **refused** (non-zero, message naming the trailer); a hook that warns and exits 0 fails this test exactly as the shipped one does today.

## 5. Zone 3 — Orchestration (Claude Code + Fable)

> Claude Code (harness / CLI) → Fable (model) → Orchestrator (plans · delegates · merges results). Constraints: limit xhigh reasoning; single concurrency (one orchestrator). Quorum: 1–3 sub-agents max — Opus, Sonnet, Haiku; independent reviewers; the Quorum Review Skill consumes their verdicts to gate the PR. Fan out → Google Antigravity (AGX, agent-execution layer; swappable executor — a local agent, Kimi, any runtime) → 2–20 AGX sub-agents, parallel workers. Working modes: Goal · Teamwork · Grill me · Plan. *One orchestrator, a small quorum for judgment, a wide Antigravity pool for throughput.*

### 5.1 Orchestrator, xhigh, single concurrency — PARTIAL

**Evidence.** The orchestrator is the `paiml-implement` skill running in Claude Code with Fable (`SKILL.md` "You are Fable, the orchestrator"). Effort is a session setting (`/effort`), not asserted by any hook. `hooks/subagent-lock.sh` keys its lock by `session_id` ("two sessions on one host neither share a cap nor clear each other's slots") — so two orchestrator sessions in the same repository are **not** prevented; the per-user lock this session hit earlier (memory: "paiml-implement subagent lock is per USER") was the previous bundle.

**Acceptance test (AD-09).** With one session holding the repo lock, a second `paiml-implement` invocation in the same `repo_root` is refused at Phase 0 naming the first session id; control: a different repo proceeds.

### 5.2 Quorum of 1–3 models that must agree — MISSING (the cap is PRESENT)

**Evidence.** `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS=3` (`settings.fragment.json`) enforced by `subagent-lock.sh` §2a via atomic slot directories — a **concurrency limit**. The routing table (`SKILL.md` §11) assigns Haiku/Sonnet/Opus by task class. Neither collects independent verdicts from three models nor requires them to agree; the plan-review lane's finding on this row is applied here: a cap and a vote are different capabilities, and only the cap exists.

**Acceptance test.** = §3.1 (AD-04): three lanes, three verdicts, agreement required.

### 5.3 Fan out to Antigravity — PARTIAL

**Evidence.** `agents/paiml-agy-delegate.md`: `teamwork` lane = one `agy … -p="/teamwork-preview …"` run (measured 2026-09-03: 245 s, 2 child conversations, `status: SUCCESS`); `quorum` lane = N parallel `agy -p` runs, **cap 10**, each validated against `agy/quorum-schema.json`; lane-side hook `~/.gemini/config/hooks.json` denies `git push`/`gh pr` inside lanes. Three headless facts (AIS-006): `--dangerously-skip-permissions` is required or a lane needing a shell prints nothing and exits 0; the prompt must be attached (`-p="…"`); the default 5 m `--print-timeout` kills teamwork.

**Grounding caveat (measured).** The teamwork lane that reviewed this document's plan reported four existing files as absent and labelled that finding `measured`; its five other findings were self-labelled `asserted`. A lane's `file:line` claims are claims (Doctrine 3); §7's sub-agent-report row applies to agy lanes as to Claude workers.

### 5.4 Swappable executor — MISSING

**Evidence.** The delegate's calling form is `agy [--sandbox] --dangerously-skip-permissions --print-timeout 25m --output-format json --json-schema <schema> -p="<prompt>"`; no indirection.

**Acceptance test (AD-08).** `PAIML_EXECUTOR=kimi` with a `kimi` shim on PATH that echoes its argv → the delegate's receipt `commands[].cmd` starts with `kimi`; `PAIML_EXECUTOR=nope` → the delegate returns `partial=true` naming the unknown executor; control: unset → `agy`.

### 5.5 2–20 workers — PARTIAL

**Evidence.** "Cap N at 10" (`paiml-agy-delegate.md` step 4). **Acceptance test (AD-08).** `width=20` runs 20 lanes under the token budget guard; `width=21` is refused.

### 5.6 Working modes — Goal MISSING · Teamwork PRESENT · Grill-me MISSING · Plan PARTIAL

**Evidence.** `agy -p="/help" --dangerously-skip-permissions --print-timeout 2m` → `/agents /changelog /config /credits /effort /help /hooks /model /permissions /skills /usage` (print mode lists only commands that need no agent turn); `/teamwork-preview` runs; `/goal` and `/grillme` are absent (AIS-006, re-verified 2026-09-03); `agy --mode plan -p` works headless (memory: reference_agy_headless_plan_review).

**Acceptance test (AD-10).** Each mode is a named lane template in the delegate (`goal`, `teamwork`, `grillme`, `plan`) with its own prompt prefix and schema; the receipt's `lane` names the mode; `goal` and `grillme` are implemented as teamwork prompts until agy ships the commands, and the receipt says so (`"emulated": true`).

## 6. Capability ledger

| # | capability | zone | status | closes with |
|---|---|---|---|---|
| 1 | Quorum Review Skill, 3 must agree | 1 | MISSING | AD-04 |
| 2 | PR gates merge | 1 | PRESENT | — |
| 3 | Build server runs the gates | 1 | PARTIAL | AD-05 (CI leg) |
| 4 | Auto-merge on green | 1 | PRESENT | — (PMAT-646 caveat) |
| 5 | crates.io publish, verified | 1 | MANUAL | AD-01 |
| 6 | Dog-food the published crate | 1 | PARTIAL | AD-02 |
| 7 | pmat MCP quality server | 2 | PRESENT | — |
| 8 | Sub-agent level compliance | 2 | PARTIAL | AD-06 |
| 9 | Max complexity | 2 | PRESENT | — |
| 10 | Lint zero warnings (gate) | 2 | PARTIAL | AD-05 |
| 11 | Churn (gate) | 2 | MISSING | AD-05 |
| 12 | Lines per file (gate) | 2 | PARTIAL | AD-05 |
| 13 | Traceability | 2 | PARTIAL | AD-03, AD-07 |
| 14 | One orchestrator, xhigh | 3 | PARTIAL | AD-09 |
| 15 | Quorum as a vote | 3 | MISSING | AD-04 |
| 16 | Concurrency cap ≤ 3 | 3 | PRESENT | — |
| 17 | Fan out to Antigravity | 3 | PARTIAL | AD-08, AD-10 |
| 18 | Swappable executor | 3 | MISSING | AD-08 |
| 19 | 2–20 workers | 3 | PARTIAL | AD-08 |
| 20 | Goal / Grill-me modes | 3 | MISSING | AD-10 |
| 21 | Teamwork / Plan modes | 3 | PRESENT / PARTIAL | AD-10 |

## 7. Micro-enforcement matrix — beyond the diagram

The diagram places the gates at the build server and at "every sub-agent before it reports back". This section places one at **every moment an agent acts**, names the enforcer, what it refuses, and how a bypass is caught. Where the enforcer column says *(AD-nn)* the row is a target; the rest is measured today.

| moment | enforcer | rule | failure mode | bypass, and how it is caught |
|---|---|---|---|---|
| **edit** | `~/.claude/hooks/subagent-lock.sh` §2b (PreToolUse Edit/Write) | no edit in a `.pmat/` repo without `$STATE/active-ticket` | exit 2, `no active ticket. Run 'pmat work add'…` | hook is fail-open by harness design; the commit-time trailer check (below) and AD-07's comply check catch an edit that reached a commit without a ticket |
| **edit (paths)** | `pmat work claim` (ULTRA-002) | one agent owns a path at a time | claim refused when held | unclaimed edits collide at merge; the PR's `files_changed` vs claims is checked by the orchestrator |
| **sub-agent spawn** | `subagent-lock.sh` §2a | ≤ `CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS` (3) | exit 2 naming the cap | `transcript-gate.sh` (I-3) audits peak overlap after the fact |
| **sub-agent report** | orchestrator (SKILL.md Phase 2 step 3) + *(AD-06)* receipt `gate` field | the receipt carries `pmat verify --format json`; the orchestrator re-runs it and `acceptance_cmd` | a receipt without `gate` is `partial=true`; disagreement is a finding in the dispatch ledger | a worker cannot bypass: the orchestrator's rerun is the verdict |
| **agy lane report** | delegate receipt (`consensus`, `dissent`, `conversations`) | every lane verdict is a claim; `file:line` findings are re-read | a `do-not-implement-as-written` stops the phase | sandboxed lanes mis-see the filesystem (§5.3); the orchestrator re-runs `test -e` on every path a lane cites |
| **commit** | generated pre-commit hook (`pmat hooks install`) *(AD-03: `--strict` blocks)* + **commit-msg hook** *(AD-03)* | `pmat verify` fast stages (format, complexity on changed files, satd) pass; message carries `Pmat-Ticket: PMAT-NNN` | pre-commit exit 1 on a red stage; commit-msg exit 1 naming the trailer | `git commit --no-verify` skips both; caught at PR by AD-07's comply check over every commit on the branch and by the same stages re-run in CI |
| **push** | pre-push hook (`scripts/install-git-hooks.sh`) | pmat-book commits pushed first; ledgers current (`analyze unrun-tests --check-ledger`, `reachability --check-ledger`) | push refused | `--no-verify` on push; caught by the `reachability-ledger` CI job (#1157) and `the_committed_ledger_matches_the_tree` |
| **PR open** | orchestrator (SKILL.md Phase 4) + PR template | body carries ticket, receipt path, named mutation RED, both-sides acceptance | a PR without them is not armed for auto-merge | AD-04's merge helper refuses `--auto` without a quorum verdict artifact |
| **PR review** | *(AD-04)* Quorum Review Skill | three lanes, three PASS | merge helper refuses | branch protection with `required_approving_review_count=0` does not enforce it; AD-04 makes the helper the only path the bundle offers, and AD-01's post-merge check names any release merged without one |
| **merge** | branch protection (strict, 5 contexts) + *(AD-05)* `pmat quality-gate --checks all` CI leg | every required check green on the merge commit | GitHub refuses the merge | admin merge (`enforce_admins=false`); caught by nothing today — recorded, not fixed |
| **post-merge release** | *(AD-01)* `release-check` job on `master` | `Cargo.toml` version ⇒ tag + GitHub release + crates.io version | job fails, issue opened | a merge that bumps the version and publishes nothing is red within one CI run — the 3.35.0 case |
| **publish** | `crate-release-dogfood` probe (self-tested) + Lane A/B readings | packaged tarball builds, links, installs, README claims hold | probe exit 1, do not publish | `cargo publish --allow-dirty/--no-verify` is P1's finding |
| **post-publish** | *(AD-02)* `scripts/dogfood-published.sh <v>` | install from crates.io, `dogfood-use.sh` 13 checks, probe P5–P7, receipt | exit 1; the release is announced only with the receipt | the receipt names the installed `commit:` — a local build cannot forge it |

Two rows are enforced by nobody today and are written down rather than hidden: the **admin merge** (branch protection does not bind admins) and the **PR review** (zero required reviews). AD-01 makes the first visible after the fact; AD-04 makes the second the bundle's only path.

## 8. Ticket ↔ work linking model

The machine-checkable record is the **commit trailer**:

```
Pmat-Ticket: PMAT-650
```

read with `git log --format='%(trailers:key=Pmat-Ticket,valueonly)'`. It is git-native, survives rebases and squashes that keep the message, and needs no network to check. The branch name (`PMAT-650-agentic-delivery-spec`) and the PR body line are **derived** and secondary — a branch is deleted on merge (`delete_branch_on_merge=true`), a PR body is not git.

| record | who writes it | who reads it |
|---|---|---|
| `Pmat-Ticket:` trailer on every commit | the committer; AD-03's commit-msg hook refuses its absence | AD-07's comply check `every commit on this branch carries a trailer naming an in-progress ticket`; `pmat work link --commit` |
| branch `PMAT-NNN-slug` | `git switch -c` in SKILL.md Phase 1 | discovery, humans |
| PR body `Ticket PMAT-NNN` + receipt path | orchestrator, Phase 4 | AD-04 quorum lanes read the ticket to judge the diff against it |
| ticket → commits/PR (`pmat work link`, AD-07) | orchestrator after push / after merge | `pmat work annotate`, release notes, the receipt's identity block |
| receipt `docs/audits/impl-<ticket>-receipt.md` | orchestrator, Phase 4 | humans; AD-04 lanes; the post-merge release check names receipts missing for merged tickets (stretch) |

A trailer naming a ticket that is not `inprogress` fails the comply check: linking to a closed or planned ticket is the same defect as no link.

## 9. Backlog — EV order

Each item ships as one ticket and one PR under the paiml-implement discipline (RED-first acceptance with a control, `pmat verify`, `make gate-artifact`, pv contract, named mutation in the PR body, receipt in `docs/audits/`). The first two answer the incident in §1 and go first; the CRUX audit's remaining items (CRUX-07 onward) rank **after** these ten — see the cross-link in `pmat-architecture-crux-audit.md` §8.0.

### 9.1 AD-01 — post-merge release check (S)
**Problem.** A merged `release/x.y.z` PR that produces no tag, release or crate is invisible to every gate (3.35.0). **Proposal.** A job in `.github/workflows/quality-gate.yml` running on `master` (push and schedule) that executes the §3.5 script; on failure it opens or refreshes an issue titled `release-check: <v> merged but not published`. **Acceptance.** §3.5, with the 9.9.9 control. **Risk.** Runner availability (the fleet was offline for hours on 2026-09-03) — the job must not be a *required* PR check; it guards `master`. **Related.** #1122 (Docker publishes 2.10.0 from a 3.35.0 crate) is the same class for the Docker channel and is folded in as a fourth leg once `docker-publish.yml` is wired.

### 9.2 AD-02 — post-publish dog-food (S)
**Problem.** Nothing exercises the bytes crates.io serves. **Proposal.** `scripts/dogfood-published.sh <version>` + `make dogfood-published VERSION=` + a step in the release process; the receipt is committed as `docs/audits/release-<v>-dogfood.md`. **Acceptance.** §3.6. **Risk.** `cargo install` needs network and ~5 min; run it once per release, not in CI on every push.

*Implementation note (PMAT-652, 2026-09-03).* `scripts/dogfood-published.sh <version>` +
`make dogfood-published VERSION=`; contract `contracts/dogfood-published-v1.yaml`. Legs in the order
they are cheapest to fail: registry entry present and un-yanked (P7) → `cargo install --locked` into a
throwaway root (P5) → the installed binary reports the version and answers `--help` → the release gate
`scripts/dogfood-use.sh` runs with `BIN=` that binary → receipt `docs/audits/release-<v>-dogfood-published.md`
(the release-chain receipt `release-<v>-dogfood.md` links to it, so the hand-written chain and the
script output do not collide) pinned by the registry's `crate_size` and `created_at` (the crates.io build reports `commit: unknown`,
CRUX-21, so a commit line cannot be the pin). Controls measured: `9.9.9` → `FAIL … is not on
crates.io`, exit 1, no receipt; `v3` → refused before any network call. Real run on the published
3.36.0: see the receipt. The 3.36.0 release itself was dog-fooded by hand with the same legs before
this script existed; that chain receipt stays, and links to this script's output.

### 9.3 AD-03 — commit enforcement (S)
**Problem.** #1126: the generated hook's SATD and task-ID checks warn and exit 0; no commit-msg hook exists. **Proposal.** `pmat hooks install --strict` (and `[hooks] strict = true` in `pmat.toml`) makes both blocking, emits a `commit-msg` hook requiring `Pmat-Ticket: PMAT-\d+` (or `#\d+` for repositories without pmat work), and the bundle's `settings.fragment.json` turns strict on. **Acceptance.** §4.7 first three legs. **Control.** the shipped hook fails the test today (warn + exit 0). **Related.** #1126.

*Implementation note (PMAT-655, 2026-09-04).* `[hooks] strict = true` / `pmat hooks install --strict`
(also on `hooks init`); `[hooks] ticket_pattern` (default `PMAT-[0-9]+|#[0-9]+` — the shipped
`PMAT-[0-9]{4}` matched no real ticket id). The pre-commit hook exports `PMAT_HOOKS_STRICT` (`--strict` OR
`[hooks] strict`, resolved once for both hooks; automatic rewrites keep it); its SATD branch exits 1 under strict; its task-ID block is gone — a pre-commit hook receives
no message, so that check had read an empty `$1` since it was written. A generated `commit-msg` hook
(`.git/hooks/commit-msg`, auto-managed) reads the trailer with `git interpret-trailers --parse`,
falls back to the pattern over non-comment lines, refuses under strict naming `Pmat-Ticket:`, warns
otherwise; `hooks uninstall` removes it. Measured both ways: `scripts/commit-enforcement-audit.sh`
(four legs driven through `git commit`) fails at leg 1 on the 3.36.0 binary (no `--strict`) and
passes on this tree; named mutation: the strict branch exiting 0 fails
`a_commit_without_a_ticket_trailer_is_refused_in_strict_mode`. The paiml-implement bundle runs
`pmat hooks install --strict` at Phase 1 (bundle PR).

### 9.4 AD-04 — Quorum Review Skill (M)
**Problem.** §3.1/§5.2. **Proposal.** `~/.claude/skills/quorum-review`: three lanes review `git diff <base>...HEAD` against the ticket text and the receipt under `agy/quorum-schema.json`; lanes are agy by default, Claude models one at a time when agy is absent (the user's one-subagent rule); "must agree" = three PASS; the verdict file is posted as a PR comment and `pmat-merge --auto` (a thin wrapper the bundle installs) refuses without it. **Acceptance.** §3.1. **Risk.** Lane grounding (§5.3): every `file:line` a lane cites is re-read by the skill before the verdict counts.

*Implementation note (PMAT-656, 2026-09-04).* The skill lives in the paiml-implement bundle, not in pmat: `skills/quorum-review/quorum-review.sh` builds one prompt (the diff, `pmat work show <ticket>`, the receipt, the refutation doctrine) and runs it through `--width` agy lanes (`--sandbox`, `agy/quorum-schema.json`), then writes `docs/audits/quorum-<ticket>.json` with `agreed`, the judged `head`, every lane's verdict and findings, and keeps the raw lane outputs under `<artifact>.lanes/`. A lane's answer is recognised only when `verdict` is one of the enum strings and `findings` is a list — the first run mistook the schema object agy echoes back for a verdict. `skills/quorum-review/pmat-merge <pr> --auto …` refuses (exit 1, naming `docs/audits/quorum-<ticket>.json`) unless an artifact agrees for the PR's current head; a non-auto merge passes through. pmat carries the acceptance (`scripts/quorum-review-audit.sh`: five offline legs through a stub `gh`, plus `--clean`/`--planted` for the two live lane artifacts), the contract `contracts/quorum-review-v1.yaml` and the receipt `docs/audits/impl-PMAT-656-receipt.md`. Without `agy` the script stops and says to run the lanes as sequential Claude reviews; it never spawns Claude subagents itself.

### 9.5 AD-05 — `quality-gate --checks lint,churn,file-size` and a CI leg (M)
**Problem.** §4.4–4.6 and §3.3. **Proposal.** Three new checks in `src/cli/analysis_utilities/quality_gate_execute.rs` / `quality_gate_part2a.rs`, routed through `quality_gate_suite.rs` so MCP `quality_gate` gains them; thresholds from `pmat.toml [quality]` (`max_file_lines` — the key `work_contract_profile.rs` already reads, default 500; `max_churn_commits_90d`, default 20; lint = clippy `-D warnings` reusing the verify stage's runner); `checks_run`, `not_measured` and the differential gate's `ALLOWED_CONSTANTS` updated with reasons. A CI leg `pmat quality-gate --checks all` on the merge commit. **Acceptance.** §4.4–4.6 with their controls. **Risk.** clippy inside the gate costs a compile; `lint` is opt-in for `--checks all` on non-Rust trees (`not_applicable`).

### 9.6 AD-06 — worker receipt carries the gate (S)
**Problem.** §4.2. **Proposal.** `agents/paiml-impl-worker.md` §receipt gains `gate: {cmd, ok, stages_measured, not_measured}` from `pmat verify --format json`; `SKILL.md` §6.2 treats its absence as `partial=true`; `verify.sh` checks the agent file contains the field. **Acceptance.** §4.2.

*Implementation note (PMAT-660, 2026-09-04).* Bundle paiml-implement#7: worker rule 4b runs `gate_cmd`
once after acceptance and reports `gate {cmd, ok, stages_measured, not_measured}` (the fields
`pmat verify --format json` emits); a receipt without `gate` is `partial=true`.
`scripts/receipt-lint.sh <receipt> [--rerun <verify.json>]` marks a missing `gate` partial and exits 1
when the claim disagrees with the orchestrator's rerun on `ok`, `stages_measured` or `not_measured`; its
`--self-test` proves both directions, and the bundle's `verify.sh` runs it. SKILL.md Phase 2 step 3 now
re-runs the gate as well as `A_i`. The first receipt held to the rule is the PMAT-657 worker's
(`docs/audits/worker-receipt-PMAT-657.json`): complete, `gate.ok=false`, the first failing test named and the other five characterised in its note, and
the orchestrator's rerun on the same tree recorded in `docs/audits/impl-PMAT-660-receipt.md` — a finding: the worker
measured five stages on its dirty tree, the rerun four on the committed one (verify's complexity stage is
withdrawn on a clean tree), so the next bundle revision makes the worker run the gate on the tree it commits.

### 9.7 AD-07 — `pmat work link` + comply trailer check (M)
**Problem.** §4.7, §8. **Proposal.** `pmat work link <ticket> --commit <sha> | --pr <n>` records on the ticket; `pmat work annotate` shows the links; comply check `CB-TRACE` walks `git log <base>..HEAD` and fails on a commit without a trailer or with a trailer naming a ticket not in progress. **Acceptance.** §4.7 last two legs.

*Implementation note (PMAT-661, 2026-09-04).* `pmat work link <ticket> --commit <sha> | --pr <n>` records
`links` on the roadmap item (serde default, omitted when empty, so old roadmaps round-trip byte-for-byte);
`pmat work annotate` shows them. Comply check **CB-1340** (`check_ticket_trailer.rs`, registered with the
cb-13xx commit checks) reads `Pmat-Ticket` with git's trailer parser for every non-merge commit between the
default branch and `HEAD` and fails naming each sha whose trailer is missing, unknown, or names a ticket that is
not in progress; on the default branch or outside a repository it passes saying there is nothing to judge.
Acceptance `scripts/ticket-trailer-audit.sh` (six legs; RED on 3.36.0: the check is absent and `work link`
does not parse), contract `contracts/ticket-trailer-v1.yaml`, receipt `docs/audits/impl-PMAT-661-receipt.md`.

### 9.8 AD-08 — swappable executor, width 2–20 (S)
**Problem.** §5.4, §5.5. **Proposal.** `PAIML_EXECUTOR` (`agy` default) in the delegate, executor-specific calling forms in one table, width cap 20 behind the token budget guard. **Acceptance.** §5.4, §5.5.

### 9.9 AD-09 — single-orchestrator lock + effort assertion (S)
**Problem.** §5.1. **Proposal.** `subagent-lock.sh` takes a host-level lock keyed by `repo_root` at Phase 0 (`SessionStart`), released at `SessionEnd`; Phase 0 prints the effort setting and refuses below `xhigh` unless `--effort-override` is named in the receipt. **Acceptance.** §5.1.

### 9.10 AD-10 — Goal and Grill-me as lane templates (S)
**Problem.** §5.6. **Proposal.** Four named lane templates in the delegate; `goal` and `grillme` emulate through `/teamwork-preview` prompts with `"emulated": true` in the receipt until agy ships the commands; `plan` = `agy --mode plan`. **Acceptance.** §5.6.

*Implementation note (PMAT-664, 2026-09-04).* Bundle paiml-implement#10: `scripts/agy-lane.sh --mode
goal|teamwork|grillme|plan --prompt "<p>" [--writes] [--dry-run]` composes the agy call per mode — `teamwork`
prefixes `/teamwork-preview` and refuses a timeout under 20 minutes; `plan` passes `--mode plan`; `goal` and
`grillme` are prompt templates with their own schemas (`agy/goal-schema.json`: achieved | partial | blocked with
grounded evidence; `agy/grillme-schema.json`: questions marked answered-by-the-text or not, verdict including
`do-not-implement-as-written`) until agy ships them natively (AIS-006); lanes are sandboxed unless `--writes`.
`--self-test` proves the refusals, each mode's calling form and the sandbox rule for every mode (fourteen checks since paiml-implement#12) and the bundle's `verify.sh` runs it;
the delegate brief gains a `mode` field and names it in its receipt. pmat side: this note and
`docs/audits/impl-PMAT-664-receipt.md`.

## 10. Do-not-do

- Do not make AD-01 a **required** PR check: it guards `master` after merge and would block every PR whenever the fleet is offline.
- Do not implement the quorum as three Claude subagents in parallel: the user's rule is one subagent at a time; the lanes are agy's, or sequential.
- Do not fold `lint`, `churn` or `file-size` into an existing threshold's count: each is a named finding type with its own `not_measured` reason (the CRUX-02 rule).
- Do not put ticket ids in commit *subjects* as the record: subjects are rewritten on squash; trailers are the record.

## 11. Definition of done

- Every capability in §6 is PRESENT, or PARTIAL with the named AD item merged, or MISSING with a cited reason recorded here.
- Every acceptance test in §3–§5 runs from a clean clone with only `git jq cargo gh` (plus `agy` for the lane tests) and fails on its control.
- `pmat-architecture-crux-audit.md` §8.0 links here and ranks AD-01…AD-10 above CRUX-07.
- The first post-publish receipt (`docs/audits/release-3.36.0-dogfood.md`) exists and names the installed commit.

## 12. Verification ledger

| claim | command | measured |
|---|---|---|
| no quorum skill installed | `ls ~/.claude/skills` | 12 skills, none review/quorum |
| protection contexts | `gh api repos/paiml/paiml-mcp-agent-toolkit/branches/master/protection/required_status_checks --jq '.contexts[]'` | 5 contexts, `strict=true`, 0 reviews, admins not enforced |
| auto-merge | `gh api repos/paiml/paiml-mcp-agent-toolkit --jq .allow_auto_merge` | `true` |
| 3.35.0 unpublished | `git tag --sort=-creatordate \| head -1`; `gh release list`; `curl https://crates.io/api/v1/crates/pmat` | `v3.34.0`; `v3.34.0`; `max_stable_version 3.34.0`; `Cargo.toml 3.35.0` |
| probe self-test fires | `bash ~/.claude/skills/crate-release-dogfood/probe.sh --self-test …/fixtures/defective-crate` | `5 probe(s) fired` |
| quality-gate checks | `pmat quality-gate --help` | `dead-code, complexity, coverage, sections, provability, satd, entropy, security, duplicates, all` |
| verify thresholds | `src/cli/verify.rs:456-458` | `--max-cyclomatic 30 --max-cognitive 25` |
| file-size claim | `src/cli/handlers/work_contract_profile.rs:336` | `max_file_lines … unwrap_or(500)` |
| hook warns | `src/cli/handlers/hooks_command_handlers/hook_generation.rs:396-401` | `Warning: … task ID …` then success banner |
| work verbs | `pmat work --help` | no `link` |
| subagent cap | `~/.claude/hooks/subagent-lock.sh:10` | `MAX="${CLAUDE_CODE_MAX_CONCURRENT_SUBAGENTS:-3}"` |
| agy commands | `agy -p="/help" --dangerously-skip-permissions --print-timeout 2m` | 11 built-ins, no `/goal`, `/grillme` |
| teamwork lane runs | delegate receipt, conversation `0a2b5862…` | 245 s, SUCCESS, 2 child conversations |
| delegate width cap | `~/src/paiml-implement/agents/paiml-agy-delegate.md` step 4 | `Cap N at 10` |
