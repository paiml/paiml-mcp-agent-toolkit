# impl receipt — PMAT-650 (agentic-delivery specification)

| field | value |
|---|---|
| ticket | PMAT-650 |
| deliverable | `docs/specifications/agentic-delivery-pmat.md` — 21 capabilities of the 2026-09-03 whiteboard measured against pmat, `~/src/paiml-implement` (c8e2ced) and agy 1.1.25; acceptance test + control per capability; micro-enforcement matrix; `Pmat-Ticket:` linking model; backlog AD-01…AD-10 |
| branch | docs/agentic-delivery-spec (PR #1168, superseded — its commit 364432d5d rides the 3.36.0 release PR #1171 as 2ada8b95d) |
| plan review | one agy `/teamwork-preview` lane (conversation 0a2b5862-3ea1-4533-9b9a-758f33be4b00, 245 s): `do-not-implement-as-written`, six findings; five applied (quorum-as-vote row, AD-01 control, EV order, matrix columns, commit trailer); one refuted by `test -e` (a sandboxed lane reported four existing files as absent) |
| cross-link | `pmat-architecture-crux-audit.md` priority banner + §8.0 rank AD-01…AD-10 above CRUX-07… |
| path sweep | every cited repo path exists except two the spec itself names as future artifacts (`scripts/dogfood-published.sh`, `docs/audits/release-3.36.0-dogfood.md`) |
| subagents | 1 delegate (agy lane), 0 workers |

## Verdict
DONE when #1171 merges. Follow-ups: AD-02 (`scripts/dogfood-published.sh`, PMAT-652 — deferred by the user; the 3.36.0 post-publish dog-food is run by hand and its receipt is the first instance), AD-03…AD-10 as their own tickets.
