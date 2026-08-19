# Antigravity Global Rules for PMAT

Welcome to the `paiml-mcp-agent-toolkit` (PMAT) codebase! PMAT is a zero-config AI context generation and code quality toolkit.

## Agent Guidelines

- Use the PMAT CLI via the configured MCP Server for quality checks and analysis.
- Follow the rules defined in `.agents/rules/` for all operations.
- Maintain test coverage above 95% (as detailed in the rules).
- Adhere to the autonomous verify loop protocol for self-correction.

When making code changes, ensure that PMAT quality gates (via the `pmat-quality-feedback.sh` hook) pass. If it fails, fix the code according to the feedback!

## Contract-first, and how it is enforced

PMAT is contract-first: a rule's behaviour is declared in `contracts/*.yaml` and
bound to the code by `#[provable_contracts_macros::contract(...)]`, so the
contract is checked by the build rather than by review. Write or update the
contract before the code it governs.

Run `pmat verify` before every commit. It runs the gate set CI enforces —
format, complexity, satd, clippy, tests — fail-fast, so green here means green
in CI. The pre-commit hook and `pmat quality-gate` both skip clippy and tests;
`pmat verify` does not.

`pmat comply check` audits the 157 registered rules. Two of them are worth
knowing before you touch a number in a config file:

- **CB-2101 (threshold coherence)** — every scalar threshold in
  `.pmat-metrics.toml` must bind to a metric in `.pmat-ratchet.toml`. A limit
  that is breached at HEAD, or so far from the measurement that nothing could
  ever reach it, fails. An unbound threshold fails too: a number enforced by
  nothing reads as enforcement and enforces nothing.
- **CB-2102 (ratchet)** — baselines may only improve. The gate does not read the
  number; it re-runs the `command` recorded beside it. A metric that cannot be
  measured FAILS, because "we could not measure it" must never be recorded as
  "it did not regress".

That last rule generalises, and it is the one to internalise: **a measurement
with no failing condition is indistinguishable from a correct one.** Prefer the
compiler over a text search whenever the question is "does this construct appear
in shipped code" — pmat is a linter, so its own source contains as data every
pattern it hunts, and grepping it for a defect marker returns its own detectors.
Before trusting any new gate, apply the defect it targets and confirm it turns
red, then restore and confirm it turns green again.
