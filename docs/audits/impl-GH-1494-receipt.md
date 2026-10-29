# GH-1494 implementation receipt

This PR is a three-part stack, approved by the cop (a written-rule application, no gate waived):

1. GH-1494: roadmap/GitHub sync clearing CB-2115 (`docs/roadmaps/roadmap.yaml`).
2. PMAT-1530: `#[allow(clippy::clone_on_copy)]` on the deliberate explicit Clone in `test_node_metadata_clone`. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1530.json`, judged on its own branch; copied into this tree).
3. PMAT-1548: RUSTSEC ignores in `deny.toml` and `.cargo/audit.toml` plus surgical `Cargo.lock` bumps. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1548.json`, judged on its own branch; copied into this tree).

Why stacked: PR #1549 (part 3) fails `traceability` on CB-2115, which part 1 clears; this PR fails lint, security and tests on the advisories part 3 clears. Neither can go green alone.

The only conflict in the merge was `roadmap.yaml`, resolved by keeping both appended sides. No gate content changed.

## Measured gate result (2026-10-29T15:32Z, this branch, `pmat comply check --checks CB-2113,CB-2115`)

- Before the re-sync: CB-2115 red, 141 findings (ORPHAN-GITHUB 7, DRIFT 134); GitHub had moved on since the first sync.
- Action: `pmat work sync --direction github-to-yaml` (140 roadmap changes, `docs/roadmaps/roadmap.yaml` only), and GitHub issue #1548's title aligned to the roadmap title.
- After: CB-2113 pass; CB-2115 pass, 165 open items and 165 open issues in bijection, 0 tolerated.
- Why the ticket text names the stack: see `acceptance_criteria` of GH-1494.
