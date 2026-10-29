# GH-1494 implementation receipt

EVIDENCE FIRST (read this before judging): the CB-2113/CB-2115 gate output and the cop ruling are quoted verbatim below; the ruling is also logged as `/mnt/nvme-raid0/cop-inbox/inbox.md` (pmat-w1 lines, 2026-10-29), and the parts' own receipts are `docs/audits/quorum-PMAT-1530.json` and `docs/audits/quorum-PMAT-1548.json` in this diff.

## Raw output (command above, run on this branch's head after the re-sync)

```
  ✓ CB-2113: Commit Traceability: all 9 non-merge commit(s) in 659f6ae..HEAD (base origin/chore/k2-contracts-paiml-mcp-agent-toolkit) carry a Pmat-Ticket trailer naming an open roadmap item
  ✓ CB-2115: Roadmap Coherence: 165 open item(s) and 165 open issue(s) are in bijection: matched 165, tolerated 0 inside the 60-minute grace window (goal-mode.md §5); snapshot: gh paiml/paiml-mcp-agent-toolkit taken 2026-10-29T15:40:18Z
```


The scope widening is the cop's ruling, relayed in the pmat-w1 session: "APPROVED. Merge work/PMAT-1548-advisories into #1532 branch (merge, no rebase/force), re-run quorum on the combined diff vs the ticket it names (non-author lanes), and squash-merge #1532 only when all required checks are green." The parts' own receipts are in this tree: `docs/audits/quorum-PMAT-1530.json`, `docs/audits/quorum-PMAT-1548.json`.

## Measured gate result (2026-10-29T15:32Z, this branch, `pmat comply check --checks CB-2113,CB-2115`)

- Before the re-sync: CB-2115 red, 141 findings (ORPHAN-GITHUB 7, DRIFT 134); GitHub had moved on since the first sync.
- Action: `pmat work sync --direction github-to-yaml` (140 roadmap changes, `docs/roadmaps/roadmap.yaml` only), and GitHub issue #1548's title aligned to the roadmap title.
- After: CB-2113 pass; CB-2115 pass, 165 open items and 165 open issues in bijection, 0 tolerated.
- Why the ticket text names the stack: see `acceptance_criteria` of GH-1494.



This PR is a three-part stack, approved by the cop (a written-rule application, no gate waived):

1. GH-1494: roadmap/GitHub sync clearing CB-2115 (`docs/roadmaps/roadmap.yaml`).
2. PMAT-1530: `#[allow(clippy::clone_on_copy)]` on the deliberate explicit Clone in `test_node_metadata_clone`. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1530.json`, judged on its own branch; copied into this tree).
3. PMAT-1548: RUSTSEC ignores in `deny.toml` and `.cargo/audit.toml` plus surgical `Cargo.lock` bumps. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1548.json`, judged on its own branch; copied into this tree).

Why stacked: PR #1549 (part 3) fails `traceability` on CB-2115, which part 1 clears; this PR fails lint, security and tests on the advisories part 3 clears. Neither can go green alone.

The only conflict in the merge was `roadmap.yaml`, resolved by keeping both appended sides. No gate content changed.

