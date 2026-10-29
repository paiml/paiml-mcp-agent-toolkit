# GH-1557 implementation receipt

Scope: `Cargo.lock` only, copied from dependabot PR #1498 (tokio 1.53.3 -> 1.54.1, hyper 1.11.1 -> 1.12.0, hyper-util 0.1.20 -> 0.1.21, tokio-test 0.4.5 -> 0.4.6, tokio-util 0.7.19 -> 0.7.20). Transitive lock edges that came with them: want 0.3.1 -> 0.3.2, httparse added, base64 0.22.1 -> 0.23.1 (hyper-util 0.1.21). No source, gate or config file changes.

Measured on this branch (2026-10-29T19:35:29Z):

- cargo deny check advisories: advisories ok
- cargo audit (tail):     Scanning Cargo.lock for vulnerabilities (854 crate dependencies)
warning: 12 allowed warnings found
- cargo check --lib --tests: Finished (no errors)

The dependabot commit has no Pmat-Ticket trailer, so CB-2113 cannot pass on #1498; this branch exists for that reason.
