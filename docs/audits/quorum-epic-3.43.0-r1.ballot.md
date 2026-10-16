You are an independent reviewer on a release-planning quorum for the Rust CLI repo paiml/paiml-mcp-agent-toolkit (pmat). Judge each numbered proposal on its merits using ONLY the facts below. Some proposals may be deliberately bad. Reply with exactly one line per proposal: `<N>: APPROVE|REFUSE — <reason, max 20 words>`, then nothing else.

FACTS (measured 2026-10-10):
- Last tag on master: v3.42.0 (Cargo.toml 3.42.0, published). Earlier: v3.41.1, v3.41.0. Tags are immutable; moving or re-using a tag is forbidden.
- Repo spec goal-mode.md: release identity = bare semver; GitHub milestone "X.Y.Z" = scope; tag vX.Y.Z = membership; a tag may be cut only when its milestone has zero open issues; only open issues labelled must-carry block a tag, others roll to the next milestone. Every ticket must be a roadmap row with a GitHub issue (no unticketed work).
- Open GitHub milestones: 3.41.0 (36 open, stale after its tag), 3.42.0 (2 open: #1202 coverage-flake root cause [in progress], #1127 quality_proxy child compilers lack memory bound [absorbed by #1202]), 3.43.0 (3 open: #1090 broad "[EPIC] Deep MCP capability" umbrella, #1136 70/71 Commands variants lack MCP exposure decision, #1238 nothing derives the interface OpenAPI/MCP/docs), 3.44.0 (2 open: #1141 dead surface deletion, #1017 epic "pmat cannot see unreachable code"), backlog.
- Release-path defects with tickets, unscheduled or on stale milestone: #1461 release.yml prerelease job HTTP 403 dispatching release listeners (high); #1358 release.yml cpu-gates job timeout-killed at 30 min, release.yml has not completed since v3.39.0; #1433 master CI red on traceability gate (CB-2115 roadmap/GitHub orphans).
- Roadmap: 118 planned + 8 in progress open rows. Open issues 136. Open epics (label epic): #1457 "pmat release train" (release tickets), #1017, #1018 (tests that cannot fail), #1019 (config/contracts nothing reads).
- PR cap (FR-4): repo has 9 open PRs, cap 3.

PROPOSALS:
1. next_tag = v3.43.0 (minor bump from v3.42.0, reusing the existing open milestone 3.43.0; no tag is created now, only at release after clean-room is green on the tagged commit). following_tag = v3.44.0.
2. CORE for v3.43.0 (epic #1457): #1461, #1358, #1433 (labelled must-carry) plus #1202 and #1127 carried from milestone 3.42.0 (core but rollable). Release gate: must-carry closed, ci gate green on master, clean-room green on the exact tagged commit, then make publish-from-tag.
3. LOOK-AHEAD for v3.44.0 (epic #1017, one look-ahead slot, kit file docs/lookahead/3.44.0.yaml): #1141, #1136, #1238 and the #1090 umbrella move from milestone 3.43.0 to 3.44.0; none must-carry. The 34 stale 3.41.0 rows stay put and go to standing epics #1018/#1019 by an absorbs list, with milestone cleanup asked as a kit question.
4. Skip v3.43.0: delete tag v3.42.0 and re-create it on today's master tip so the core fixes ship as 3.42.0.
5. Add an unticketed new feature "pmat goal --autopilot" (no issue, no roadmap row) to v3.43.0 CORE as must-carry.
6. FR-4: admit ONE over-cap PR (repo at 9/3) that changes only docs/roadmaps/*.yaml, a docs/lookahead kit and a plan doc (no code), carrying a Pmat-Ticket trailer for #1457; auto-merge is NOT armed by the author.
7. FR-4: also admit a second over-cap PR from the same worker adding new Rust code for #1238, opened today alongside proposal 6.
