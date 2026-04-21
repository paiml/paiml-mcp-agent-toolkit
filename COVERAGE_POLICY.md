# Coverage Policy

> **Status**: Phase 0 honesty baseline (v3.15.1)
> **Related spec**: [`docs/specifications/improve-coverage-80-95.md`](docs/specifications/improve-coverage-80-95.md)
> **Last updated**: 2026-04-21

## 1. Two Numbers, One Project

PMAT has two coverage numbers. Both are real; they measure different slices.

| Scope | Target | Lines measured | % | Gate? |
|-------|--------|----------------|---|-------|
| **Narrow** (`make coverage`) | ≥95% | 20,394 | 96.03% | ✅ Blocking |
| **Broad** (`make coverage-broad`) | ≥95% | 324,456 | 73.14% | ⚠️ Informational |

The narrow number gates CI today. The broad number is the honest project-wide baseline; closing the gap is the `improve-coverage-80-95.md` roadmap.

## 2. Canonical Commands

```bash
make coverage          # Narrow slice, gated at COV_THRESHOLD=95%, ~5 min
make coverage-broad    # Honest project baseline, informational, ~8 min
make coverage-quick    # Fast inner loop, no threshold, ~2-3 min
make coverage-html     # Render last run as HTML
```

**Never use `cargo tarpaulin`.** CB-127 policy: stack-wide `cargo-llvm-cov` only. Tarpaulin hangs on PMAT and has cache bugs.

**Never use `cargo nextest` for coverage.** CB-127-A: nextest writes one profraw per test (≈15k files) and the merge step becomes the bottleneck. Use `cargo llvm-cov test` (one profraw per binary).

**`PROPTEST_CASES` during coverage runs**: `2` for `make coverage` (gate), `3` for `make coverage-quick`. Proptest default (256) multiplies wall-clock by ~50× without adding coverage — the boundary cases are already hit in the first few iterations.

## 3. What the Narrow Gate Excludes (and Why)

`COVERAGE_EXCLUDE` (Makefile line 47) drops the following from `make coverage`. Each entry is listed with the reason it is currently excluded and the plan for re-including it under `improve-coverage-80-95.md`.

### 3.1 Unconditional excludes (stay out — LLVM can't or shouldn't instrument)

| Pattern | Reason | Status |
|---------|--------|--------|
| `_tests?\.rs` | Test files measure other code; self-coverage is meaningless | Permanent |
| `/(tests\|benches\|examples\|fixtures)/` | Test/bench/example/fixture directories | Permanent |
| `main.rs` | Binary entrypoint — integration tests spawn subprocesses (out of instrumentation scope) | Permanent |
| `test_performance_suite.rs` | Benchmark scaffold, not production | Permanent |

### 3.2 Selection-bias excludes (must be re-included per the roadmap)

These directories hold live production code that currently dodges the gate. The broad number in §1 is the result of un-excluding them. Each has a Phase and an exit condition.

| Pattern | Lines | Current broad cov | Re-include in | Exit condition |
|---------|-------|-------------------|---------------|----------------|
| `/cli/` | ~91k | ~65% | Phase 1 (v3.16.0) | CLI integration tests un-skipped; dispatch collapsed via macro |
| `/handlers/` | ~3.2k | ~36% | Phase 2 (v3.17.0) | Parity tests for every tool (D101/D102/D103 pattern) |
| `/services/` | ~75k | ~80% | Phase 2 (v3.17.0) | TDD on hot paths ranked by `--rank-by impact` |
| `/tdg/` | ~13k | ~78% | Phase 2 (v3.17.0) | Co-equal to services/ |
| `/roadmap/` | | ~32% | Phase 1 (v3.16.0) | Delete dead code; contract-gate live surface |
| `/scaffold/` | | ~0% | Phase 1 (v3.16.0) | `insta` snapshot tests per generator |
| `/workflow/` | | | Phase 3 (v3.18.0) | Contract-driven + mutation-forced |
| `/contracts/` | | | Phase 3 (v3.18.0) | Contract-driven + mutation-forced |
| `/red_team/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/qdd/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/unified_quality/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/state/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/protocol/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/docs_enforcement/` | | | Phase 2 (v3.17.0) | TDD on hot paths |
| `/provable-contracts/` | | | Phase 3 (v3.18.0) | Contract-driven — covered by the contract harness itself |
| `explain.rs` | | | Phase 2 (v3.17.0) | TDD |

### 3.3 Network-dependent excludes (re-include in contract tests, not unit tests)

| Pattern | Reason |
|---------|--------|
| `/mcp[^/]*/` | MCP transports require live stdio/HTTP servers. Covered by MCP integration tests (see `tests/mcp_*`), not unit coverage. Plan: cross-protocol integration suite in Phase 3. |

### 3.4 Skipped at runtime (not regex-excluded)

`make coverage` also passes `--skip libsql --skip cli_integration_tests` to the test runner:

- `libsql` — optional SQL backend, network/disk-dependent, covered by dedicated integration suite.
- `cli_integration_tests` — these test the CLI dispatch surface (`/cli/` above). Un-skipping these is the highest-ROI Phase 1 action per `improve-coverage-80-95.md` §3.1.

## 4. Source-Level `#[coverage(off)]`

PMAT uses module-level `#![cfg_attr(coverage_nightly, coverage(off))]` in ~250 files. Total attribute occurrences: 2,831 across 1,967 files; function-level `coverage(off)` drops roughly 8,097 functions from measurement.

**The current `make coverage` target sets `--no-cfg-coverage --no-cfg-coverage-nightly`**, which disables the `cfg(coverage_nightly)` predicate. That means the narrow gate *does* measure these modules despite the attribute — the exclusion comes from the regex in §3, not the attribute. `make coverage-broad` drops the regex so the attribute also has no effect.

Rationale categories (to be audited in Phase 3):

1. **Justified** — test scaffolding, unreachable polyfills, LLVM-can't-instrument (WASM shim, any future GPU shaders). These stay.
2. **Cargo-culted** — `#![cfg_attr(coverage_nightly, coverage(off))]` copy-pasted into every new module. Most of these should be removed.
3. **Masking low coverage** — attribute applied to hide a file that legitimately needs tests. These are the Phase 3 targets.

The Phase 3 audit in `improve-coverage-80-95.md` §5 will categorize every remaining attribute into these three buckets and justify or remove each.

## 5. CI Integration

**Current (v3.15.x)**: Sovereign CI (`.github/workflows/ci.yml` → `paiml/.github/sovereign-ci.yml`) runs `make coverage` as a gated job. The broad number is not yet reported.

**Phase 0.1 follow-up** (pending upstream `paiml/.github` change): sovereign-ci adds a non-gating `coverage-broad` step that reports the honest number for visibility. Blocked by upstream workflow edit; tracked separately.

## 6. Philosophy (Toyota Way — Jidoka)

**The narrow number is not a lie — it's a partial measurement.** Hiding code from a coverage gate is a form of selection bias. The path to honesty is three-phased:

- **Phase 0 (this doc)** — make the selection bias visible and justified per entry.
- **Phases 1–2** — delete dead code, un-skip integration tests, write tests for modules currently hidden by regex. Lower the delta between narrow and broad to ≤2%.
- **Phase 3** — audit every `coverage(off)` attribute; remove unjustified ones; couple with mutation testing so 100% line coverage cannot survive weak assertions.

The deliverable is *one honestly-measured number ≥95%*, not two numbers with a plausible story about the gap.

## 7. References

- [`docs/specifications/improve-coverage-80-95.md`](docs/specifications/improve-coverage-80-95.md) — the roadmap
- [trueno `COVERAGE_POLICY.md`](https://github.com/paiml/trueno/blob/main/COVERAGE_POLICY.md) — pattern template
- `Makefile` — `coverage`, `coverage-broad`, `coverage-quick` targets
- [CB-127 (stack-wide tarpaulin ban)](docs/specifications/quick-test-build-O(1)-checking.md)
- rust-lang/rust#84605 — `coverage_attribute` stabilization (still open on nightly 2026-04-18)
