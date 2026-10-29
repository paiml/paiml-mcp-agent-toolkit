# GH-1494 implementation receipt

This PR is a three-part stack, approved by the cop (a written-rule application, no gate waived):

1. GH-1494: roadmap/GitHub sync clearing CB-2115 (`docs/roadmaps/roadmap.yaml`).
2. PMAT-1530: `#[allow(clippy::clone_on_copy)]` on the deliberate explicit Clone in `test_node_metadata_clone`. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1530.json`, judged on its own branch).
3. PMAT-1548: RUSTSEC ignores in `deny.toml` and `.cargo/audit.toml` plus surgical `Cargo.lock` bumps. Quorum AGREED, 3 PASS (`docs/audits/quorum-PMAT-1548.json`, judged on its own branch).

Why stacked: PR #1549 (part 3) fails `traceability` on CB-2115, which part 1 clears; this PR fails lint, security and tests on the advisories part 3 clears. Neither can go green alone.

The only conflict in the merge was `roadmap.yaml`, resolved by keeping both appended sides. No gate content changed.
