# Implementation receipt — GH-1562

Cop ruling (pmat-cop, 2026-10-29): carry dependabot #1550's lock change onto a fresh work/ branch with a ticket trailer, quorum with Opus lanes, arm squash.

## Measured
- `git diff origin/master --stat`: Cargo.lock (+ roadmap row GH-1562 and this receipt); no source or gate content touched.
- `cargo deny check advisories`: `advisories ok`
- `cargo audit`: no new vulnerabilities (12 allowed warnings, pre-existing ignores)
- `cargo check --lib --tests`: `Finished dev profile` (exit 0)

## Scope
Cargo.lock only, 11 patch bumps identical to #1550 (dependabot/cargo/patch-updates-3df945c169).
