# Implementation receipt — GH-1562

Cop ruling (pmat-cop, 2026-10-29): carry dependabot #1550's lock change onto a fresh work/ branch with a ticket trailer, quorum with Opus lanes, arm squash.

## Measured
- `cargo deny check advisories`: `advisories ok`
- `cargo audit`: no new vulnerabilities (12 allowed warnings, pre-existing)
- `cargo check --lib --tests`: `Finished dev profile` (exit 0)

## Scope — exactly three paths
1. `Cargo.lock`: identical to dependabot/cargo/patch-updates-3df945c169 (#1550). Beyond the 11 named patch bumps, octocrab 0.54.2 -> 0.54.5 switches jsonwebtoken 10.4.0's crypto backend from RustCrypto to aws-lc-rs; rsa, crypto-bigint, p256, p384 leave the graph. This is inherited from #1550, not chosen here.
2. `docs/roadmaps/roadmap.yaml`: rows GH-1562 (this ticket) and GH-1559. GH-1559 is NOT wanted by this ticket: `pmat work sync --direction github-to-yaml` mints a row for every live issue, and CB-2115 (roadmap/GitHub bijection against live GitHub) reds this PR without it. Removing it would break the gate.
3. This receipt.
No source or gate content touched.

## Follow-up, not in scope
The ignores for RUSTSEC-2023-0071 and RUSTSEC-2026-0387 in `deny.toml` / `.cargo/audit.toml` may now be stale (rsa and crypto-bigint 0.5.5 are out of the lock). Removing them is gate content; left untouched per cop directive.
